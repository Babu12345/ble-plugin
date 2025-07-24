//! USB device class for the esp-idf hal interace
//! Taking example for https://github.com/esp-rs/esp-hal/blob/main/examples/src/bin/usb_serial.rs for the final product of
//! how this will be called and referenced in code.
#![allow(static_mut_refs)]
use std::cmp::min;
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::Scope;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

use esp_idf_svc::hal::task::block_on;

use esp_idf_sys::cherry_device::{
    CDC_ACM_DESCRIPTOR_LEN, USB_2_0, USB_CONFIG_BUS_POWERED, USB_DESCRIPTOR_TYPE_DEVICE_QUALIFIER,
    USB_DEVICE_CLASS_MISC, usb_descriptor, usbd_add_endpoint, usbd_add_interface,
    usbd_cdc_acm_init_intf, usbd_cdc_acm_set_dtr, usbd_desc_register, usbd_endpoint,
    usbd_ep_start_read, usbd_ep_start_write, usbd_event_type_USBD_EVENT_CLR_REMOTE_WAKEUP,
    usbd_event_type_USBD_EVENT_CONFIGURED, usbd_event_type_USBD_EVENT_CONNECTED,
    usbd_event_type_USBD_EVENT_DISCONNECTED, usbd_event_type_USBD_EVENT_RESET,
    usbd_event_type_USBD_EVENT_RESUME, usbd_event_type_USBD_EVENT_SET_REMOTE_WAKEUP,
    usbd_event_type_USBD_EVENT_SUSPEND, usbd_get_ep_mps, usbd_initialize, usbd_interface,
};

use crate::utils::{
    CDC_MAX_MPS, cdc_acm_descriptor_init, config_descriptor_init, device_descriptor_init,
};
use crate::{AlignedBuffer, concat_n_arrays, mk_static};

use std::ptr;
use std::sync::LazyLock;
const CDC_IN_EP: u8 = 0x81;
const CDC_OUT_EP: u8 = 0x02;
const CDC_INT_EP: u8 = 0x83; // 0x85
const USB_CONFIG_SIZE: u32 = 9 + CDC_ACM_DESCRIPTOR_LEN;
const USBD_VID: u16 = 0xFFFF;
const USBD_PID: u16 = 0xFFFF;
const USBD_MAX_POWER: u32 = 100; // 2mA * 100 = 100 mA
const SIZE: usize = CDC_MAX_MPS as usize;

static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);

static mut READ_BUFFER: AlignedBuffer<64> = AlignedBuffer::new();
static mut INPUT: [u8; SIZE] = [0; SIZE];

/// Sending and receiving type
pub type TSendAndReceive = [u8; 64];
static SIGNAL: Signal<CriticalSectionRawMutex, TSendAndReceive> = Signal::new();

/// Class error type
#[derive(Debug)]
pub enum Error {
    /// Custom error type
    CustomError(&'static str),
}

/// Result type with the custom error
pub type Result<T> = core::result::Result<T, Error>;

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L19
static DEVICE_DESCRIPTOR: LazyLock<[u8; 18]> = LazyLock::new(|| {
    device_descriptor_init(
        USB_2_0,
        USB_DEVICE_CLASS_MISC, // USB_DEVICE_CLASS_CDC
        0x02,
        0x01,
        USBD_VID as u32,
        USBD_PID as u32,
        0x0100,
        0x01,
    )
});

// https://claude.ai/chat/b333a37f-351f-4bd3-b4af-ed1c3888b205
static CONFIG_DESCRIPTOR: LazyLock<[u8; 75]> = LazyLock::new(|| {
    concat_n_arrays!(
        config_descriptor_init(
            USB_CONFIG_SIZE,
            0x02,
            0x01,
            USB_CONFIG_BUS_POWERED,
            USBD_MAX_POWER,
        ),
        cdc_acm_descriptor_init(
            0x00,
            CDC_INT_EP as u32,
            CDC_OUT_EP as u32,
            CDC_IN_EP as u32,
            CDC_MAX_MPS,
            0x02,
        )
    )
});

static DEVICE_QUALITY_DESCRIPTOR: [u8; 10] = [
    0x0a,                                       // bLength
    USB_DESCRIPTOR_TYPE_DEVICE_QUALIFIER as u8, // bDescriptorType (Device Qualifier)
    0x00,
    0x02, // bcdUSB
    0x00, // bDeviceClass
    0x00, // bDeviceSubClass
    0x00, // bDeviceProtocol
    0x40, // bMaxPacketSize0
    0x00, // bNumConfigurations
    0x00, // bReserved
];

static STRING_MANUFACTURER: &[u8] = b"Wanyeki Technologies LLC\0";
static STRING_PRODUCT: &[u8] = b"BLEPlugin\0";
static STRING_SERIAL: &[u8] = b"1999\0";
static STRING_LANGID: &[u8] = b"\x09\x04\0";

// https://github.com/orangecms/RV-Debugger-BL702/blob/05739699b50a9235f8906bd80b4b8f7dd0c37e62/components/usb_stack/common/usb_def.h#L473
#[unsafe(no_mangle)]
unsafe extern "C" fn device_descriptor_callback(_speed: u8) -> *const u8 {
    DEVICE_DESCRIPTOR.as_ptr() as *const u8
}

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L72
#[unsafe(no_mangle)]
unsafe extern "C" fn device_quality_descriptor_callback(_speed: u8) -> *const u8 {
    DEVICE_QUALITY_DESCRIPTOR.as_ptr()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn config_descriptor_callback(_speed: u8) -> *const u8 {
    CONFIG_DESCRIPTOR.as_ptr()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn string_descriptor_callback(_speed: u8, index: u8) -> *const u8 {
    match index {
        0 => STRING_LANGID.as_ptr() as *const u8,
        1 => STRING_MANUFACTURER.as_ptr() as *const u8,
        2 => STRING_PRODUCT.as_ptr() as *const u8,
        3 => STRING_SERIAL.as_ptr() as *const u8,
        _ => ptr::null(),
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn usbd_cdc_acm_bulk_out(busid: u8, ep: u8, nbytes: u32) {
    unsafe {
        INPUT = [0; SIZE];
        (&mut INPUT[..nbytes as usize]).copy_from_slice(&READ_BUFFER.get_data()[..nbytes as usize]);
    }

    SIGNAL.signal(unsafe { INPUT });
    unsafe { usbd_ep_start_read(busid, ep, READ_BUFFER.as_mut_ptr(), SIZE as u32) };
}

#[unsafe(no_mangle)]
unsafe extern "C" fn usbd_cdc_acm_bulk_in(busid: u8, ep: u8, nbytes: u32) {
    let ep_mps = unsafe { usbd_get_ep_mps(busid, ep) as u32 };
    match (nbytes % ep_mps) == 0 && nbytes > 0 {
        true => {
            unsafe { usbd_ep_start_write(busid, ep, ptr::null(), 0) };
        }
        false => {}
    };
}

#[unsafe(no_mangle)]
unsafe extern "C" fn usbd_event_handler(busid: u8, event: u8) {
    #[allow(non_upper_case_globals, non_snake_case)]
    match event as u32 {
        usbd_event_type_USBD_EVENT_RESET
        | usbd_event_type_USBD_EVENT_CONNECTED
        | usbd_event_type_USBD_EVENT_DISCONNECTED
        | usbd_event_type_USBD_EVENT_RESUME
        | usbd_event_type_USBD_EVENT_SUSPEND
        | usbd_event_type_USBD_EVENT_SET_REMOTE_WAKEUP
        | usbd_event_type_USBD_EVENT_CLR_REMOTE_WAKEUP => {}
        usbd_event_type_USBD_EVENT_CONFIGURED => {
            let _res = unsafe {
                usbd_ep_start_read(
                    busid,
                    CDC_OUT_EP as u8,
                    READ_BUFFER.as_mut_ptr(),
                    SIZE as u32,
                )
            };
        }
        _ => {}
    }
}

/// Main CDC ACM device structure
#[derive(Debug)]
pub struct CdcAcmDevice<STATE> {
    descriptor: &'static usb_descriptor,
    cdc_out_ep: &'static mut usbd_endpoint,
    cdc_in_ep: &'static mut usbd_endpoint,
    intf0: &'static mut usbd_interface,
    intf1: &'static mut usbd_interface,
    busid: Option<u8>,
    _state: PhantomData<STATE>,
}

/// Pre device configuration
pub struct PREINIT;

/// Post device configuration
pub struct POSTINIT;

/// https://github.com/CherryUSB/cherryusb_esp32/tree/main/examples/device
impl CdcAcmDevice<PREINIT> {
    /// Initiates a new cdc device
    pub fn new() -> Self {
        let descriptor = mk_static!(
            usb_descriptor,
            usb_descriptor {
                device_descriptor_callback: Some(device_descriptor_callback),
                config_descriptor_callback: Some(config_descriptor_callback),
                device_quality_descriptor_callback: Some(device_quality_descriptor_callback),
                string_descriptor_callback: Some(string_descriptor_callback),
                ..Default::default()
            }
        );
        let intf0 = mk_static!(usbd_interface, usbd_interface::default());
        let intf1 = mk_static!(usbd_interface, usbd_interface::default());

        let cdc_out_ep = mk_static!(
            usbd_endpoint,
            usbd_endpoint {
                ep_addr: CDC_OUT_EP as u8,
                ep_cb: Some(usbd_cdc_acm_bulk_out),
            }
        );

        let cdc_in_ep = mk_static!(
            usbd_endpoint,
            usbd_endpoint {
                ep_addr: CDC_IN_EP as u8,
                ep_cb: Some(usbd_cdc_acm_bulk_in),
            }
        );

        Self {
            cdc_out_ep,
            cdc_in_ep,
            intf0,
            intf1,
            descriptor,
            busid: None,
            _state: PhantomData::<PREINIT>,
        }
    }

    /// initialize the device
    pub fn init(self, busid: u8, reg_base: u32) -> Result<CdcAcmDevice<POSTINIT>> {
        match IS_INITIALIZED.load(std::sync::atomic::Ordering::Relaxed) {
            true => {
                return Err(Error::CustomError("Already initialized"));
            }
            false => {}
        }
        unsafe {
            usbd_desc_register(busid, self.descriptor);
            usbd_add_interface(busid, usbd_cdc_acm_init_intf(busid, self.intf0)); // 0
            usbd_add_interface(busid, usbd_cdc_acm_init_intf(busid, self.intf1)); // 1
            usbd_add_endpoint(busid, self.cdc_out_ep);
            usbd_add_endpoint(busid, self.cdc_in_ep);

            match usbd_initialize(busid, reg_base as usize, Some(usbd_event_handler)) {
                x if x < 0 => {
                    return Err(Error::CustomError("Failed to initialize the usb device"));
                }
                _ => IS_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed),
            }
        }
        ::log::info!("Usb device initialized");

        Ok(CdcAcmDevice {
            cdc_out_ep: self.cdc_out_ep,
            cdc_in_ep: self.cdc_in_ep,
            intf0: self.intf0,
            intf1: self.intf1,
            descriptor: self.descriptor,
            busid: Some(busid),
            _state: PhantomData::<POSTINIT>,
        })
    }
}

impl CdcAcmDevice<POSTINIT> {
    /// Input and output to process data to and from the usb peripheral
    pub fn processors<'a, 'b>(
        self,
        scope: &'a Scope<'a, 'b>,
        channel_buffer_size: usize,
    ) -> Result<(SyncSender<TSendAndReceive>, Receiver<TSendAndReceive>)> {
        let to_usb: (SyncSender<TSendAndReceive>, Receiver<TSendAndReceive>) =
            sync_channel(channel_buffer_size);
        let from_usb = sync_channel(channel_buffer_size);

        let busid = self.busid.unwrap();
        // Writing to the usb endpoint
        scope.spawn(move || {
            loop {
                match to_usb.1.recv() {
                    Ok(mut data) => {
                        match unsafe {
                            usbd_ep_start_write(
                                busid,
                                CDC_IN_EP as u8,
                                data.as_mut_ptr(),
                                min(data.len() as u32, SIZE as u32),
                            )
                        } {
                            x if x < 0 => ::log::error!("Failed to send via usb device"),
                            _ => {}
                        }
                    }
                    Err(e) => ::log::error!("Unable to recieve data: {e}"),
                }
            }
        });

        // Reading from the usb endpoint
        scope.spawn(move || {
            loop {
                let data = block_on(SIGNAL.wait());

                match from_usb.0.try_send(data) {
                    Ok(_) => {}
                    Err(e) => {
                        ::log::error!("Unable to send data: {e}");
                    }
                };
            }
        });

        Ok((to_usb.0, from_usb.1))
    }

    /// Set the dtr of the usb cdc device
    pub fn set_dtr(self, intf: u8, dtr: bool) -> Self {
        let busid = self.busid.unwrap();
        unsafe {
            usbd_cdc_acm_set_dtr(busid, intf, dtr);
        }

        self
    }
}
