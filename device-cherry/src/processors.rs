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
    CDC_ACM_DESCRIPTOR_LEN, USB_2_0, USB_CONFIG_BUS_POWERED, USB_DESCRIPTOR_TYPE_DEVICE_QUALIFIER,
    USB_DEVICE_CLASS_MISC, usb_descriptor, usbd_add_endpoint, usbd_add_interface,
    usbd_cdc_acm_init_intf, usbd_cdc_acm_set_dtr, usbd_desc_register, usbd_endpoint,
    usbd_ep_start_read, usbd_ep_start_write, usbd_event_type_USBD_EVENT_CLR_REMOTE_WAKEUP,
    usbd_event_type_USBD_EVENT_CONFIGURED, usbd_event_type_USBD_EVENT_CONNECTED,
    usbd_event_type_USBD_EVENT_DISCONNECTED, usbd_event_type_USBD_EVENT_RESET,
    usbd_event_type_USBD_EVENT_RESUME, usbd_event_type_USBD_EVENT_SET_REMOTE_WAKEUP,
    usbd_event_type_USBD_EVENT_SUSPEND, usbd_get_ep_mps, usbd_initialize, usbd_interface,
};
use protocol::DEFAULT_PACKET_SIZE;
use protocol::plugin::plugin::{PluginReceiver, PluginSender};
use std::collections::VecDeque;
use throttle::Throttle;

use crate::utils::{
    CDC_MAX_MPS, cdc_acm_descriptor_init, config_descriptor_init, device_descriptor_init,
};
use crate::{AlignedBuffer, concat_n_arrays};
use crate::{Error, Result};
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
static WRITE_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

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
    // Bounds check to prevent buffer overrun panic
    let data_len = core::cmp::min(nbytes as usize, SIZE);
    let buffer_data = active_buffer.get_data();
    if data_len <= buffer_data.len() {
        let data_slice = buffer_data[..data_len].match_size(0x00);
        SIGNAL.signal(data_slice);
    } else {
        // Log error but don't panic
        ::log::error!("Buffer overrun attempt: data_len={}, buffer_len={}", data_len, buffer_data.len());
    }
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

    // Minimal delay for high-speed scenarios - reduced from 10us to 1us
    std::thread::sleep(Duration::from_micros(1));

    // Start read with error checking and retry logic - increased retries for high-speed
    let mut retry_count = 0;
    const MAX_RETRIES: i32 = 10;

    loop {
        let result =
            unsafe { usbd_ep_start_read(busid, ep, next_buffer.as_mut_ptr(), SIZE as u32) };
        if result >= 0 {
            // Success
            break;
        }

        retry_count += 1;
        if retry_count >= MAX_RETRIES {
            // Log failure but don't panic - continue operation
            ::log::warn!("Failed to restart USB read after {} retries", MAX_RETRIES);
            break;
        }

        // More aggressive backoff for high-speed scenarios
        let delay_us = match retry_count {
            1..=3 => 10,  // Very short delay for first few retries
            4..=6 => 50,  // Medium delay
            _ => 100,     // Longer delay for persistent issues
        };
        std::thread::sleep(Duration::from_micros(delay_us));
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn usbd_cdc_acm_bulk_in(busid: u8, ep: u8, nbytes: u32) {
    // Clear write-in-progress flag to allow next write
    WRITE_IN_PROGRESS.store(false, Ordering::Release);

    let ep_mps = unsafe { usbd_get_ep_mps(busid, ep) as u32 };
    // Send Zero Length Packet if needed (when data is multiple of max packet size)
    if (nbytes % ep_mps) == 0 && nbytes > 0 {
        // Don't send ZLP immediately - let the next write handle it
        // This prevents endpoint busy errors
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
            // Reset write state on configuration
            WRITE_IN_PROGRESS.store(false, Ordering::Release);
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

impl CdcAcmDevice<POSTINIT> {
    /// Input and output to process data to and from the usb peripheral
    pub fn processors<'a, 'b>(
        self,
        scope: &'a Scope<'a, 'b>,
        channel_buffer_size: usize,
        throttle_info: (Duration, usize),
    ) -> Result<(PluginSender<SIZE>, PluginReceiver<SIZE>)> {
        let to_usb: (SyncSender<TSendAndReceive>, Receiver<TSendAndReceive>) =
            sync_channel(channel_buffer_size);
        let from_usb = sync_channel(channel_buffer_size);

        let busid = self.busid.ok_or(Error::BusidUndefined)?;
        // Writing to USB endpoint with proper flow control
        scope.spawn(move || {
            let mut write_queue: VecDeque<TSendAndReceive> = VecDeque::with_capacity(16);
            let mut consecutive_errors = 0u32;

            loop {
                // Try to get data from channel
                match to_usb.1.try_recv() {
                    Ok(data) => {
                        // Prevent unbounded queue growth during high-speed bursts
                        if write_queue.len() < 16 {
                            write_queue.push_back(data);
                            // Drain channel to queue (up to capacity)
                            while write_queue.len() < 16 {
                                match to_usb.1.try_recv() {
                                    Ok(data) => write_queue.push_back(data),
                                    Err(_) => break,
                                }
                            }
                        } else {
                            // Queue full - drop the oldest packet to prevent memory overflow
                            write_queue.pop_front();
                            write_queue.push_back(data);
                            ::log::debug!("Write queue full, dropping oldest packet");
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        if write_queue.is_empty() {
                            // Block waiting for data only if queue is empty
                            match to_usb.1.recv_timeout(Duration::from_millis(1)) {
                                Ok(data) => write_queue.push_back(data),
                                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                                Err(_) => break, // Channel disconnected
                            }
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                }

                // Process write queue with flow control
                while let Some(mut data) = write_queue.pop_front() {
                    // Wait for previous write to complete
                    let mut wait_cycles = 0;
                    while WRITE_IN_PROGRESS.load(Ordering::Acquire) {
                        wait_cycles += 1;
                        if wait_cycles > 1000 {
                            ::log::warn!("USB write endpoint busy for too long");
                            std::thread::sleep(Duration::from_micros(100));
                            wait_cycles = 0;
                        }
                        std::thread::yield_now();
                    }

                    // Set write-in-progress flag
                    WRITE_IN_PROGRESS.store(true, Ordering::Release);

                    let len = min(data.len() as u32, SIZE as u32);
                    let result = unsafe {
                        usbd_ep_start_write(busid, CDC_IN_EP as u8, data.as_mut_ptr(), len)
                    };

                    if result < 0 {
                        WRITE_IN_PROGRESS.store(false, Ordering::Release);
                        consecutive_errors += 1;

                        // Handle specific I/O error codes
                        let should_retry = match result {
                            -5 => {
                                // EIO (Input/Output error)
                                ::log::warn!("USB I/O error detected, increasing retry delay");
                                true
                            }
                            -16 => {
                                // EBUSY (Device or resource busy)
                                ::log::debug!("USB endpoint busy, will retry");
                                true
                            }
                            _ => {
                                ::log::warn!("USB write failed with error: {}", result);
                                consecutive_errors <= 5 // Only retry for limited attempts on other errors
                            }
                        };

                        if should_retry && consecutive_errors <= 5 {
                            // Re-queue the data for retry with exponential backoff
                            write_queue.push_front(data);
                            let delay_ms = std::cmp::min(50, 5 * consecutive_errors);
                            std::thread::sleep(Duration::from_millis(delay_ms as u64));
                        } else {
                            ::log::error!(
                                "USB write failed after {} attempts, error: {}",
                                consecutive_errors,
                                result
                            );
                            consecutive_errors = 0;
                            // Drop the packet to prevent infinite retry
                        }
                    } else {
                        consecutive_errors = 0;
                        // Slightly longer delay between successful writes to prevent I/O errors
                        std::thread::sleep(Duration::from_micros(50));
                    }
                }
            }

            ::log::info!("USB write thread exiting");
        });

        // Reading from USB endpoint - optimized for high-speed burst handling
        scope.spawn(move || {
            let mut throttle = Throttle::new(throttle_info.0, throttle_info.1);
            let mut packet_count = 0u64;
            let mut dropped_packets = 0u64;

            loop {
                let data = block_on(SIGNAL.wait());
                packet_count += 1;

                // More aggressive throttle bypass for high-speed scenarios
                if packet_count > 100 {  // Increased from 10 to 100
                    if let Err(_) = throttle.accept() {
                        dropped_packets += 1;
                        // Log periodically but don't spam
                        if dropped_packets % 1000 == 0 {
                            ::log::debug!("Throttle dropped {} packets out of {}", dropped_packets, packet_count);
                        }
                        continue;
                    }
                }

                // Try non-blocking send with better error handling
                match from_usb.0.try_send(data) {
                    Ok(_) => {
                        // Reset dropped packet counter on successful sends
                        if dropped_packets > 0 && packet_count % 1000 == 0 {
                            ::log::debug!("Channel processing {} packets, {} dropped", packet_count, dropped_packets);
                        }
                    }
                    Err(std::sync::mpsc::TrySendError::Full(_)) => {
                        dropped_packets += 1;
                        // Channel full - drop packet to maintain USB responsiveness
                        // Log channel overflow periodically
                        if dropped_packets % 1000 == 0 {
                            ::log::warn!("Channel overflow: dropped {} packets", dropped_packets);
                        }
                    }
                    Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                        ::log::info!("USB read thread exiting - channel disconnected");
                        break;
                    }
                };
            }
        });

        Ok((PluginSender::new(to_usb.0), PluginReceiver::new(from_usb.1)))
    }

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
