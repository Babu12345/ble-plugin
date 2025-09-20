//! USB device class for the esp-idf hal interace
//! Taking example for https://github.com/esp-rs/esp-hal/blob/main/examples/src/bin/usb_serial.rs for the final product of
//! how this will be called and referenced in code.
use std::cmp::min;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::Scope;
use std::time::Duration;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

use esp_idf_svc::hal::task::block_on;

use esp_idf_sys::cherry_device::{
    CDC_ACM_DESCRIPTOR_LEN, USB_2_0, USB_CONFIG_BUS_POWERED, USB_DEVICE_CLASS_MISC, usb_descriptor,
    usbd_add_endpoint, usbd_add_interface, usbd_cdc_acm_init_intf, usbd_cdc_acm_set_dtr,
    usbd_desc_register, usbd_endpoint, usbd_ep_start_read, usbd_ep_start_write,
    usbd_event_type_USBD_EVENT_CLR_REMOTE_WAKEUP, usbd_event_type_USBD_EVENT_CONFIGURED,
    usbd_event_type_USBD_EVENT_CONNECTED, usbd_event_type_USBD_EVENT_DISCONNECTED,
    usbd_event_type_USBD_EVENT_RESET, usbd_event_type_USBD_EVENT_RESUME,
    usbd_event_type_USBD_EVENT_SET_REMOTE_WAKEUP, usbd_event_type_USBD_EVENT_SUSPEND,
    usbd_get_ep_mps, usbd_initialize, usbd_interface,
};
use lib_utils::types::AlignedBuffer;
use protocol::DEFAULT_PACKET_SIZE;
use protocol::devices::host::HostProcessor;
use protocol::devices::plugin::PluginProcessor;
use protocol::devices::{ReadThrottleInfo, WriteThrottleInfo};
use protocol::host::{HostReceiver, HostSender};
use protocol::plugin::plugin::{PluginReceiver, PluginSender};
use throttle::Throttle;

use crate::utils::{
    CDC_BULK_MPS, cdc_acm_descriptor_init, config_descriptor_init, device_descriptor_init,
};
use crate::{Error, Result};
use crate::{concat_n_arrays, device_quality_descriptor_init, other_speed_config_descriptor};
use lib_utils::{MatchSliceLengths, mk_static};

use std::ptr;
use std::sync::LazyLock;
const CDC_IN_EP: u8 = 0x81;
const CDC_OUT_EP: u8 = 0x02;
const CDC_INT_EP: u8 = 0x83; // 0x85
const USB_CONFIG_SIZE: u32 = 9 + CDC_ACM_DESCRIPTOR_LEN;
const USBD_VID: u16 = 0xFFFF;
const USBD_PID: u16 = 0xFFFF;
const USBD_MAX_POWER: u32 = 100; // 100 mA
const SIZE: usize = DEFAULT_PACKET_SIZE;

static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);

// Double buffering for improved throughput
static mut READ_BUFFER_A: AlignedBuffer<DEFAULT_PACKET_SIZE> = AlignedBuffer::new();
static mut READ_BUFFER_B: AlignedBuffer<DEFAULT_PACKET_SIZE> = AlignedBuffer::new();
static ACTIVE_BUFFER: AtomicUsize = AtomicUsize::new(0);

/// Sending and receiving type
pub type TSendAndReceive = [u8; SIZE];
static SIGNAL: Signal<CriticalSectionRawMutex, TSendAndReceive> = Signal::new();

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
            CDC_BULK_MPS,
            0x02,
        )
    )
});

static DEVICE_QUALITY_DESCRIPTOR: LazyLock<[u8; 10]> = LazyLock::new(|| {
    device_quality_descriptor_init(USB_2_0, USB_DEVICE_CLASS_MISC, 0x02, 0x01, 0x01)
});

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L44
static DEVICE_OTHER_SPEED_CONFIG_DESCRIPTOR: LazyLock<[u8; 10]> = LazyLock::new(|| {
    concat_n_arrays!(
        other_speed_config_descriptor(
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
            CDC_BULK_MPS,
            0x02,
        )
    )
});

static STRING_MANUFACTURER: &[u8] = b"Wanyeki Technologies LLC\0";
static STRING_PRODUCT: &[u8] = b"BLE Plugin\0";
static STRING_SERIAL: &[u8] = b"1999\0";
static STRING_LANGID: &[u8] = b"\x09\x04\0";

// https://github.com/orangecms/RV-Debugger-BL702/blob/05739699b50a9235f8906bd80b4b8f7dd0c37e62/components/usb_stack/common/usb_def.h#L473
#[unsafe(no_mangle)]
unsafe extern "C" fn device_descriptor_callback(_speed: u8) -> *const u8 {
    DEVICE_DESCRIPTOR.as_ptr()
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
unsafe extern "C" fn other_speed_config_descriptor_callback(_speed: u8) -> *const u8 {
    DEVICE_OTHER_SPEED_CONFIG_DESCRIPTOR.as_ptr()
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
    #![allow(static_mut_refs)]

    // Critical: Must restart USB read to maintain host communication
    // Optimized for high-speed data processing

    // Bounds check with early return
    if nbytes as usize > SIZE {
        // Still must restart reading immediately
        unsafe { restart_usb_read_immediate(busid, ep) };
        return;
    }

    // Handle zero-length packets efficiently
    if nbytes == 0 {
        unsafe { restart_usb_read_immediate(busid, ep) };
        return;
    }

    // Get current buffer atomically
    let current = ACTIVE_BUFFER.load(Ordering::Acquire);
    let active_buffer = unsafe {
        if current == 0 {
            &READ_BUFFER_A
        } else {
            &READ_BUFFER_B
        }
    };

    // Switch buffer immediately to prepare for next read
    let new_buffer = 1 - current;
    ACTIVE_BUFFER.store(new_buffer, Ordering::Release);

    // Restart read with explicit buffer selection to avoid race
    unsafe { restart_usb_read_with_buffer(busid, ep, new_buffer) };

    // Process the received data after ensuring continuity
    // Make sure that data length is within bounds
    let data_len = core::cmp::min(nbytes as usize, SIZE);
    let data_slice = active_buffer.get_data()[..data_len].match_size(0x00);
    SIGNAL.signal(data_slice);
}

// Race-free USB read restart with explicit buffer selection
unsafe fn restart_usb_read_with_buffer(busid: u8, ep: u8, buffer_index: usize) {
    #![allow(static_mut_refs)]
    let next_buffer = unsafe {
        if buffer_index == 0 {
            &mut READ_BUFFER_A
        } else {
            &mut READ_BUFFER_B
        }
    };

    // Try immediate restart with minimal error handling for speed
    let result = unsafe { usbd_ep_start_read(busid, ep, next_buffer.as_mut_ptr(), SIZE as u32) };
    if result < 0 {
        // Fall back to delayed restart only if immediate fails
        unsafe { restart_usb_read_with_delay(busid, ep) };
    }
}

// Immediate USB read restart for high-speed scenarios (for other callers)
unsafe fn restart_usb_read_immediate(busid: u8, ep: u8) {
    let current = ACTIVE_BUFFER.load(Ordering::Acquire);
    unsafe { restart_usb_read_with_buffer(busid, ep, current) };
}

// USB read restart with small delay to prevent I/O errors
unsafe fn restart_usb_read_with_delay(busid: u8, ep: u8) {
    #![allow(static_mut_refs)]
    let current = ACTIVE_BUFFER.load(Ordering::Acquire);
    let next_buffer = unsafe {
        if current == 0 {
            &mut READ_BUFFER_A
        } else {
            &mut READ_BUFFER_B
        }
    };

    // Start read with error checking and retry logic - increased retries for high-speed
    let mut retry_count = 0;
    const MAX_RETRIES: i32 = 20; // Increased from 10 for better resilience

    loop {
        // No initial delay for maximum throughput
        let result =
            unsafe { usbd_ep_start_read(busid, ep, next_buffer.as_mut_ptr(), SIZE as u32) };
        if result >= 0 {
            // Success
            break;
        }

        retry_count += 1;
        if retry_count >= MAX_RETRIES {
            // Log failure but don't panic - continue operation
            break;
        }

        // Ultra-aggressive retry strategy for high throughput
        let delay_us = match retry_count {
            1..=5 => 1,    // Minimal delay for first retries
            6..=10 => 5,   // Very short delay
            11..=15 => 10, // Short delay
            _ => 20,       // Still short for persistent issues
        };
        std::thread::sleep(Duration::from_micros(delay_us));
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn usbd_cdc_acm_bulk_in(busid: u8, ep: u8, nbytes: u32) {
    let ep_mps = unsafe { usbd_get_ep_mps(busid, ep) as u32 };
    // Send Zero Length Packet if needed (when data is multiple of max packet size)
    if (nbytes % ep_mps) == 0 && nbytes > 0 {
        // Don't send ZLP immediately - let the next write handle it
        // This prevents endpoint busy errors
        // Also prevents flooding the host with unnecessary packets that need processing and
        // can cause confusion on the host side
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn usbd_event_handler(busid: u8, event: u8) {
    #[allow(non_upper_case_globals, non_snake_case, static_mut_refs)]
    match event as u32 {
        usbd_event_type_USBD_EVENT_RESET
        | usbd_event_type_USBD_EVENT_CONNECTED
        | usbd_event_type_USBD_EVENT_DISCONNECTED
        | usbd_event_type_USBD_EVENT_RESUME
        | usbd_event_type_USBD_EVENT_SUSPEND
        | usbd_event_type_USBD_EVENT_SET_REMOTE_WAKEUP
        | usbd_event_type_USBD_EVENT_CLR_REMOTE_WAKEUP => {}
        usbd_event_type_USBD_EVENT_CONFIGURED => {
            // Start with buffer A
            ACTIVE_BUFFER.store(0, Ordering::Release);
            unsafe {
                restart_usb_read_immediate(busid, CDC_OUT_EP as u8);
            }
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

/// Main CDC ACM host that's a usb device
#[derive(Debug)]
pub struct CdcAcmDeviceHost<STATE> {
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
/// USB Device that implements the PluginProcessor
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
                other_speed_descriptor_callback: Some(other_speed_config_descriptor_callback),
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
                return Err(Error::DeviceAlreadyInitialized);
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
                    return Err(Error::InitializationFailure);
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

impl PluginProcessor<SIZE, crate::errors::Error> for CdcAcmDevice<POSTINIT> {
    /// Input and output to process data to and from the usb peripheral
    fn processors<'a, 'b>(
        self,
        scope: &'a Scope<'a, 'b>,
        channel_buffer_size: usize,
        read_throttle_info: ReadThrottleInfo,
        write_throttle_info: WriteThrottleInfo,
    ) -> Result<(PluginSender<SIZE>, PluginReceiver<SIZE>)> {
        let busid = self.busid.ok_or(Error::BusidUndefined)?;

        let (sender, receiver) = processor_common(
            scope,
            channel_buffer_size,
            read_throttle_info,
            write_throttle_info,
            busid,
        )?;

        Ok((PluginSender::new(sender), PluginReceiver::new(receiver)))
    }
}

impl CdcAcmDevice<POSTINIT> {
    /// Set the dtr of the usb cdc device
    pub fn set_dtr(self, intf: u8, dtr: bool) -> Self {
        let busid = self.busid.unwrap();
        unsafe {
            usbd_cdc_acm_set_dtr(busid, intf, dtr);
        }

        self
    }

    /// Sleep for a specified duration
    pub fn sleep(self, duration: Duration) -> Self {
        std::thread::sleep(duration);
        self
    }
}

/// USB Device that implements the HostProcessor
impl CdcAcmDeviceHost<PREINIT> {
    /// Initiates a new cdc device host
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
    pub fn init(self, busid: u8, reg_base: u32) -> Result<CdcAcmDeviceHost<POSTINIT>> {
        match IS_INITIALIZED.load(std::sync::atomic::Ordering::Relaxed) {
            true => {
                return Err(Error::DeviceAlreadyInitialized);
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
                    return Err(Error::InitializationFailure);
                }
                _ => IS_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed),
            }
        }
        ::log::info!("Usb device initialized");

        Ok(CdcAcmDeviceHost {
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

impl HostProcessor<SIZE, crate::errors::Error> for CdcAcmDeviceHost<POSTINIT> {
    /// Input and output to process data to and from the usb peripheral
    fn processors<'a, 'b>(
        self,
        scope: &'a Scope<'a, 'b>,
        channel_buffer_size: usize,
        read_throttle_info: ReadThrottleInfo,
        write_throttle_info: WriteThrottleInfo,
    ) -> Result<(HostSender<SIZE>, HostReceiver<SIZE>)> {
        let busid = self.busid.ok_or(Error::BusidUndefined)?;

        let (sender, receiver) = processor_common(
            scope,
            channel_buffer_size,
            read_throttle_info,
            write_throttle_info,
            busid,
        )?;

        Ok((HostSender::new(sender), HostReceiver::new(receiver)))
    }
}

impl CdcAcmDeviceHost<POSTINIT> {
    /// Set the dtr of the usb cdc device
    pub fn set_dtr(self, intf: u8, dtr: bool) -> Self {
        let busid = self.busid.unwrap();
        unsafe {
            usbd_cdc_acm_set_dtr(busid, intf, dtr);
        }

        self
    }

    /// Sleep for a specified duration
    pub fn sleep(self, duration: Duration) -> Self {
        std::thread::sleep(duration);
        self
    }
}

fn processor_common<'a, 'b>(
    scope: &'a Scope<'a, 'b>,
    channel_buffer_size: usize,
    read_throttle_info: ReadThrottleInfo,
    write_throttle_info: WriteThrottleInfo,
    busid: u8,
) -> Result<(SyncSender<TSendAndReceive>, Receiver<TSendAndReceive>)> {
    let to_usb: (SyncSender<TSendAndReceive>, Receiver<TSendAndReceive>) =
        sync_channel(channel_buffer_size);
    let from_usb = sync_channel(channel_buffer_size);

    // Writing to USB endpoint with proper flow control
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
                        x if x < 0 => ::log::error!("Failed to send via usb device: {data:?}"),
                        _ => {}
                    }
                }
                Err(e) => ::log::error!("Unable to recieve data: {e}"),
            }
            std::thread::sleep(write_throttle_info.delay);
        }
    });

    // Reading from USB endpoint - optimized for high-speed burst handling
    scope.spawn(move || {
        let ReadThrottleInfo {
            timeout,
            threshold_for_timeout,
        } = read_throttle_info;
        let mut throttle = Throttle::new(timeout, threshold_for_timeout);
        let mut packet_count = 0u64;

        // Sliding window for dropped packet tracking
        const WINDOW_SIZE: usize = 10000; // Track last 10k packets
        let mut window_total = 0u64;
        let mut window_dropped = 0u64;
        let mut total_dropped = 0u64; // Keep total for overall stats

        loop {
            let data = block_on(SIGNAL.wait());
            packet_count += 1;
            window_total += 1;

            // Reset window when it reaches the size limit
            if window_total >= WINDOW_SIZE as u64 {
                // Log window statistics before reset
                if window_dropped > 0 {
                    let drop_rate = (window_dropped as f64 / window_total as f64) * 100.0;
                    ::log::info!(
                        "Window stats: {:.2}% drop rate ({}/{} packets dropped)",
                        drop_rate,
                        window_dropped,
                        window_total
                    );
                }
                // Reset window counters
                window_total = 0;
                window_dropped = 0;
            }

            // More aggressive throttle bypass for high-speed scenarios
            if packet_count > 100 {
                // Increased from 10 to 100
                if let Err(_) = throttle.accept() {
                    window_dropped += 1;
                    total_dropped += 1;
                    // Log periodically but don't spam
                    if total_dropped % 1000 == 0 {
                        let recent_rate = if window_total > 0 {
                            (window_dropped as f64 / window_total as f64) * 100.0
                        } else {
                            0.0
                        };
                        ::log::debug!(
                            "Throttle: recent drop rate {:.2}%, total dropped {}",
                            recent_rate,
                            total_dropped
                        );
                    }
                    continue;
                }
            }

            // Try non-blocking send with better error handling
            match from_usb.0.try_send(data) {
                Ok(_) => {
                    // Log successful processing periodically with window stats
                    if packet_count % 5000 == 0 && window_total > 0 {
                        let recent_rate = (window_dropped as f64 / window_total as f64) * 100.0;
                        ::log::debug!(
                            "Processing: {} total packets, recent drop rate {:.2}%",
                            packet_count,
                            recent_rate
                        );
                    }
                }
                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                    window_dropped += 1;
                    total_dropped += 1;
                    // Channel full - drop packet to maintain USB responsiveness
                    // Log channel overflow periodically
                    if total_dropped % 1000 == 0 {
                        let recent_rate = if window_total > 0 {
                            (window_dropped as f64 / window_total as f64) * 100.0
                        } else {
                            0.0
                        };
                        ::log::warn!(
                            "Channel overflow: recent drop rate {:.2}%, total dropped {}",
                            recent_rate,
                            total_dropped
                        );
                    }
                }
                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                    ::log::info!("USB read thread exiting - channel disconnected");
                    break;
                }
            };
        }
    });

    Ok((to_usb.0, from_usb.1))
}
