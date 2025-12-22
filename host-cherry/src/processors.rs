// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

use std::{
    sync::{
        Condvar, Mutex as StdMutex, RwLock,
        atomic::Ordering,
        mpsc::{Receiver, SyncSender},
    },
    time::Duration,
};

use esp_idf_sys::cherry_host::{
    usbh_cdc_acm, usbh_cdc_acm_bulk_in_transfer, usbh_cdc_acm_bulk_out_transfer,
    usbh_cdc_acm_set_line_state, usbh_deinitialize, usbh_initialize,
};
use lib_utils::types::AlignedBuffer;

use protocol::{DEFAULT_PACKET_SIZE, devices::WriteThrottleInfo};
use protocol::{
    devices::{ReadThrottleInfo, host::HostProcessor, plugin::PluginProcessor},
    host::{HostReceiver, HostSender},
    plugin::plugin::{PluginReceiver, PluginSender},
};

use std::{
    marker::PhantomData,
    sync::{atomic::AtomicBool, mpsc::sync_channel},
    thread::Scope,
};

use crate::utils::{TSenderAndReceiver, ThreadSafeCDCWrapper};

// Threshold for triggering USB stack re-initialization
// If we've been waiting this many attempts, reset the USB stack
const REINIT_THRESHOLD: u32 = 10;

// Store USB initialization parameters for re-initialization
static USB_INIT_PARAMS: StdMutex<Option<(u8, u32)>> = StdMutex::new(None);

/// Re-initialize the USB host stack
/// This clears any stuck state in the USB controller
fn reinitialize_usb_stack() -> bool {
    log::warn!("Re-initializing USB stack to recover from stuck state...");

    let params = USB_INIT_PARAMS.lock().ok().and_then(|p| *p);
    let Some((busid, reg_base)) = params else {
        log::error!("USB init parameters not available for re-initialization");
        return false;
    };

    unsafe {
        // Deinitialize the USB stack
        let ret = usbh_deinitialize(busid);
        if ret < 0 {
            log::error!("USB deinitialize failed: {}", ret);
            return false;
        }

        // Small delay to allow hardware to settle
        std::thread::sleep(Duration::from_millis(100));

        // Re-initialize the USB stack
        let ret = usbh_initialize(busid, reg_base as usize);
        if ret < 0 {
            log::error!("USB re-initialize failed: {}", ret);
            return false;
        }

        log::info!("USB stack re-initialized successfully");
        true
    }
}

static CDC_LOCKER: RwLock<Option<ThreadSafeCDCWrapper>> = RwLock::new(None);
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
// Wait for device with longer timeout to allow USB enumeration
// USB enumeration can take 3-5 seconds, especially after rapid restarts
static ENUMERATION_WAIT_TIMEOUT: Duration = Duration::from_millis(100);

// Synchronization primitive for CDC device readiness
struct CdcReadySignal {
    ready: StdMutex<bool>,
    condvar: Condvar,
    configured: AtomicBool,
    generation: AtomicBool, // Toggle on each connect/disconnect to force new waits
}

impl CdcReadySignal {
    const fn new() -> Self {
        Self {
            ready: StdMutex::new(false),
            condvar: Condvar::new(),
            configured: AtomicBool::new(false),
            generation: AtomicBool::new(false),
        }
    }

    fn signal(&self) {
        let Ok(mut ready) = self.ready.lock() else {
            log::error!("Failed to acquire lock in signal()");
            return;
        };
        *ready = true;
        // Mark as not configured when device reconnects
        self.configured.store(false, Ordering::Release);
        // Toggle generation to invalidate stale waits
        self.generation.fetch_xor(true, Ordering::Release);
        self.condvar.notify_all();
    }

    fn wait_ready(&self, timeout: Duration) -> bool {
        let Ok(ready) = self.ready.lock() else {
            log::error!("Failed to acquire lock in wait_ready()");
            return false;
        };

        // If already ready, return immediately
        if *ready {
            return true;
        }

        // Otherwise wait for signal with timeout
        let Ok(result) = self.condvar.wait_timeout(ready, timeout) else {
            log::error!("Condvar wait_timeout failed in wait_ready()");
            return false;
        };
        *result.0
    }

    fn reset(&self) {
        let Ok(mut ready) = self.ready.lock() else {
            log::error!("Failed to acquire lock in reset()");
            return;
        };
        *ready = false;
        self.configured.store(false, Ordering::Release);
        // Toggle generation on disconnect too
        self.generation.fetch_xor(true, Ordering::Release);
    }

    fn mark_configured(&self) -> bool {
        // Returns true if we were the first to configure
        self.configured
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
}

static CDC_READY_SIGNAL: CdcReadySignal = CdcReadySignal::new();

/// Strong reference to the cdc runner defined in C
/// This callback is invoked asynchronously by the USB stack when the CDC device is enumerated.
#[unsafe(no_mangle)]
extern "C" fn usbh_cdc_acm_run(cdc_acm_class: *mut usbh_cdc_acm) {
    // Ensure proper memory ordering: Release guarantees that the pointer write
    // is visible to all threads that subsequently Acquire the lock
    std::sync::atomic::fence(Ordering::Release);

    let Ok(mut locker) = CDC_LOCKER.write() else {
        log::error!("Failed to acquire write lock in usbh_cdc_acm_run()");
        return;
    };
    *locker = Some(ThreadSafeCDCWrapper(cdc_acm_class));
    drop(locker);

    // Signal waiting threads that CDC device is available
    CDC_READY_SIGNAL.signal();

    log::info!("CDC ACM device enumerated and ready");
}

/// Strong reference to the cdc acm stopper defined in C
#[unsafe(no_mangle)]
#[allow(unused_variables)]
extern "C" fn usbh_cdc_acm_stop(cdc_acm_class: *mut usbh_cdc_acm) {
    log::info!("CDC ACM device disconnected");
    let Ok(mut locker) = CDC_LOCKER.write() else {
        log::error!("Failed to acquire write lock in usbh_cdc_acm_stop()");
        return;
    };
    *locker = None;
    drop(locker);
    CDC_READY_SIGNAL.reset();
}

/// Initialize the usb host and send out receivers and senders to process and send information to the connected usb device via the cdc acm driver class.

/// Pre device configuration
pub struct PREINIT;

/// Post device configuration
pub struct POSTINIT;

/// Host device that implement the HostProcessor
pub struct CdcAcmHost<STATE> {
    _state: PhantomData<STATE>,
}

/// https://github.com/CherryUSB/cherryusb_esp32/tree/main/examples/device
impl CdcAcmHost<PREINIT> {
    /// Create a new instance of the host device
    pub fn new() -> Self {
        Self {
            _state: PhantomData::<PREINIT>,
        }
    }
    /// Initialize the device
    pub fn init(self, busid: u8, reg_base: u32) -> Result<CdcAcmHost<POSTINIT>, ()> {
        // Use Acquire ordering to ensure we see any previous initialization
        match IS_INITIALIZED.load(Ordering::Acquire) {
            true => {
                log::error!("USB host already initialized");
                return Err(());
            }
            false => {}
        }

        // Store parameters for potential re-initialization
        if let Ok(mut params) = USB_INIT_PARAMS.lock() {
            *params = Some((busid, reg_base));
        }

        // Reset the ready signal in case of previous failed initialization
        CDC_READY_SIGNAL.reset();

        match unsafe { usbh_initialize(busid, reg_base as usize) } {
            x if x < 0 => {
                log::error!("USB host initialization failed with code: {}", x);
                return Err(());
            }
            _ => {
                // Use Release ordering to ensure initialization is visible to other threads
                IS_INITIALIZED.store(true, Ordering::Release);

                // Memory fence to ensure all hardware register writes complete
                // before we proceed with device operations
                std::sync::atomic::fence(Ordering::SeqCst);

                log::info!("USB host initialized successfully");
            }
        }

        Ok(CdcAcmHost {
            _state: PhantomData::<POSTINIT>,
        })
    }
}

impl CdcAcmHost<POSTINIT> {
    /// Sleep for a specified duration
    pub fn sleep(self, duration: Duration) -> Self {
        std::thread::sleep(duration);
        self
    }
}

impl HostProcessor<DEFAULT_PACKET_SIZE, ()> for CdcAcmHost<POSTINIT> {
    fn processors<'a, 'b>(
        self,
        scope: &'a Scope<'a, 'b>,
        channel_buffer_size: usize,
        _read_throttle_info: ReadThrottleInfo,
        write_throttle_info: WriteThrottleInfo,
    ) -> Result<
        (
            HostSender<DEFAULT_PACKET_SIZE>,
            HostReceiver<DEFAULT_PACKET_SIZE>,
        ),
        (),
    > {
        log::info!("Spawning processor threads (will wait for CDC device in background)");

        let to_usb = sync_channel(channel_buffer_size);
        let from_usb = sync_channel(channel_buffer_size);

        scope.spawn(move || unsafe { send_usb_data(to_usb.1, write_throttle_info) });
        scope.spawn(move || unsafe { receive_usb_data(from_usb.0) });

        Ok((HostSender::new(to_usb.0), HostReceiver::new(from_usb.1)))
    }
}

/// Host device that implement the PluginProcessor
pub struct CdcAcmHostDevice<STATE> {
    _state: PhantomData<STATE>,
}

/// https://github.com/CherryUSB/cherryusb_esp32/tree/main/examples/device
impl CdcAcmHostDevice<PREINIT> {
    /// Create a new instance of the host device
    pub fn new() -> Self {
        Self {
            _state: PhantomData::<PREINIT>,
        }
    }
    /// Initialize the device
    pub fn init(self, busid: u8, reg_base: u32) -> Result<CdcAcmHostDevice<POSTINIT>, ()> {
        // Use Acquire ordering to ensure we see any previous initialization
        match IS_INITIALIZED.load(Ordering::Acquire) {
            true => {
                log::error!("USB host already initialized");
                return Err(());
            }
            false => {}
        }

        // Store parameters for potential re-initialization
        if let Ok(mut params) = USB_INIT_PARAMS.lock() {
            *params = Some((busid, reg_base));
        }

        // Reset the ready signal in case of previous failed initialization
        CDC_READY_SIGNAL.reset();

        match unsafe { usbh_initialize(busid, reg_base as usize) } {
            x if x < 0 => {
                log::error!("USB host initialization failed with code: {}", x);
                return Err(());
            }
            _ => {
                // Use Release ordering to ensure initialization is visible to other threads
                IS_INITIALIZED.store(true, Ordering::Release);

                // Memory fence to ensure all hardware register writes complete
                // before we proceed with device operations
                std::sync::atomic::fence(Ordering::SeqCst);

                log::info!("USB host initialized successfully");
            }
        }

        Ok(CdcAcmHostDevice {
            _state: PhantomData::<POSTINIT>,
        })
    }
}

impl CdcAcmHostDevice<POSTINIT> {
    /// Sleep for a specified duration
    pub fn sleep(self, duration: Duration) -> Self {
        std::thread::sleep(duration);
        self
    }
}

impl PluginProcessor<DEFAULT_PACKET_SIZE, ()> for CdcAcmHostDevice<POSTINIT> {
    fn processors<'a, 'b>(
        self,
        scope: &'a Scope<'a, 'b>,
        channel_buffer_size: usize,
        _read_throttle_info: ReadThrottleInfo,
        write_throttle_info: WriteThrottleInfo,
    ) -> Result<
        (
            PluginSender<DEFAULT_PACKET_SIZE>,
            PluginReceiver<DEFAULT_PACKET_SIZE>,
        ),
        (),
    > {
        log::info!("Spawning processor threads (will wait for CDC device in background)");

        let to_usb = sync_channel(channel_buffer_size);
        let from_usb = sync_channel(channel_buffer_size);

        scope.spawn(move || unsafe { send_usb_data(to_usb.1, write_throttle_info) });
        scope.spawn(move || unsafe { receive_usb_data(from_usb.0) });

        Ok((PluginSender::new(to_usb.0), PluginReceiver::new(from_usb.1)))
    }
}

/// Verify that the CDC device is ready for communication
fn verify_cdc_device_ready() -> bool {
    let Ok(locker) = CDC_LOCKER.read() else {
        log::error!("Failed to acquire read lock in verify_cdc_device_ready()");
        return false;
    };
    match locker.as_ref() {
        Some(_) => {
            log::info!("CDC device pointer verified");
            true
        }
        None => {
            log::error!("CDC device pointer is null after enumeration signal");
            false
        }
    }
}

/// Configure the CDC line state after device verification
fn configure_cdc_line_state() {
    let Ok(locker) = CDC_LOCKER.read() else {
        log::error!("Failed to acquire read lock in configure_cdc_line_state()");
        return;
    };
    if let Some(wrapper) = locker.as_ref() {
        unsafe {
            usbh_cdc_acm_set_line_state(wrapper.0, true, false);
        }
        log::info!("CDC line state configured (DTR=true, RTS=false)");
    }
}

unsafe fn receive_usb_data(sender: SyncSender<TSenderAndReceiver>) {
    let mut aligned_buffer = AlignedBuffer::<{ size_of::<TSenderAndReceiver>() }>::new();
    let mut reconnect_attempts = 0;

    loop {
        // Device should already be ready, but handle disconnection gracefully
        let cdc_acm_class: *mut usbh_cdc_acm = {
            let Ok(locker) = CDC_LOCKER.read() else {
                log::error!("Failed to acquire read lock in receive_usb_data()");
                std::thread::sleep(Duration::from_millis(10));
                continue;
            };
            match locker.as_ref() {
                Some(wrapper) => wrapper,
                None => {
                    drop(locker);
                    reconnect_attempts += 1;
                    if reconnect_attempts == 1 {
                        log::warn!("CDC device not connected, waiting for device...");
                    }

                    // If we've been waiting too long, re-initialize the USB stack
                    // This clears any stuck state in the USB controller
                    if reconnect_attempts == REINIT_THRESHOLD {
                        log::warn!(
                            "Device not reconnecting after {} attempts, re-initializing USB stack",
                            REINIT_THRESHOLD
                        );
                        if reinitialize_usb_stack() {
                            reconnect_attempts = 0;
                            // Give the new stack time to initialize
                            std::thread::sleep(Duration::from_millis(200));
                        }
                    }

                    if !CDC_READY_SIGNAL.wait_ready(ENUMERATION_WAIT_TIMEOUT) {
                        if reconnect_attempts % 10 == 0 {
                            log::info!(
                                "Still waiting for CDC device... ({} attempts)",
                                reconnect_attempts
                            );
                        }
                        continue;
                    }

                    log::info!("CDC device signal received, verifying...");

                    // PERFORMANCE: Reduced settling delay from 100ms to 50ms
                    std::thread::sleep(Duration::from_millis(50));

                    // Device reconnected - use atomic to ensure only one thread configures
                    std::sync::atomic::fence(Ordering::Acquire);
                    if verify_cdc_device_ready() {
                        // Atomically check and mark as configured
                        if CDC_READY_SIGNAL.mark_configured() {
                            configure_cdc_line_state();
                            log::info!("CDC device reconnected and reconfigured");
                        } else {
                            log::info!(
                                "CDC device reconnected (already configured by other thread)"
                            );
                        }
                        reconnect_attempts = 0; // Reset on successful reconnection
                    } else {
                        log::warn!("CDC device verification failed after signal, retrying...");
                    }
                    continue;
                }
            }
            .0
        };

        // Reset reconnect counter on successful device access
        reconnect_attempts = 0;

        match unsafe {
            usbh_cdc_acm_bulk_in_transfer(
                cdc_acm_class,
                aligned_buffer.as_mut_ptr(),
                aligned_buffer.len() as u32,
                u32::MAX,
            )
        } {
            x if x < 0 => {
                log::error!("Unable to receive data (error code: {})", x);
                // Small delay before retry to avoid tight error loop
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            _ => {}
        };

        match sender.try_send(aligned_buffer.get_data()) {
            Ok(_) => {}
            Err(std::sync::mpsc::TrySendError::Full(_)) => {
                log::warn!(
                    "Receive buffer is full. You must ingest in order to receive additional information."
                );
            }
            Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                log::error!("Disconnect error");
            }
        }
    }
}

unsafe fn send_usb_data(
    receiver: Receiver<TSenderAndReceiver>,
    write_throttle_info: WriteThrottleInfo,
) {
    let mut reconnect_attempts = 0;

    loop {
        // Device should already be ready, but handle disconnection gracefully
        let cdc_acm_class: *mut usbh_cdc_acm = {
            let Ok(locker) = CDC_LOCKER.read() else {
                log::error!("Failed to acquire read lock in send_usb_data()");
                std::thread::sleep(Duration::from_millis(10));
                continue;
            };
            match locker.as_ref() {
                Some(wrapper) => wrapper,
                None => {
                    drop(locker);
                    reconnect_attempts += 1;
                    if reconnect_attempts == 1 {
                        log::warn!("Send thread: CDC device not connected, waiting for device...");
                    }

                    if !CDC_READY_SIGNAL.wait_ready(ENUMERATION_WAIT_TIMEOUT) {
                        if reconnect_attempts % 10 == 0 {
                            log::info!(
                                "Send thread: Still waiting for CDC device... ({} attempts)",
                                reconnect_attempts
                            );
                        }
                        continue;
                    }

                    log::info!("Send thread: CDC device signal received, verifying...");

                    // PERFORMANCE: Reduced settling delay from 100ms to 50ms
                    std::thread::sleep(Duration::from_millis(50));

                    // Device reconnected - use atomic to ensure only one thread configures
                    std::sync::atomic::fence(Ordering::Acquire);
                    if verify_cdc_device_ready() {
                        // Atomically check and mark as configured
                        if CDC_READY_SIGNAL.mark_configured() {
                            configure_cdc_line_state();
                            log::info!("CDC device reconnected and reconfigured");
                        } else {
                            log::info!(
                                "CDC device reconnected (already configured by other thread)"
                            );
                        }
                        reconnect_attempts = 0; // Reset on successful reconnection
                    } else {
                        log::warn!("CDC device verification failed in send thread, retrying...");
                    }
                    continue;
                }
            }
            .0
        };

        // Reset reconnect counter on successful device access
        reconnect_attempts = 0;

        let mut data = match receiver.recv() {
            Ok(data) => AlignedBuffer::from(data),
            Err(e) => {
                log::error!("Error occurred {e}");
                continue;
            }
        };

        match unsafe {
            usbh_cdc_acm_bulk_out_transfer(
                cdc_acm_class,
                data.as_mut_ptr(),
                data.len() as u32,
                u32::MAX,
            )
        } {
            x if x < 0 => {
                log::error!("Unable to send data (error code: {})", x);
                // Small delay before retry to avoid tight error loop
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            _ => {}
        };

        std::thread::sleep(write_throttle_info.delay);
    }
}
