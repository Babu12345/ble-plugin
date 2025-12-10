// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

use std::{
    sync::{
        RwLock,
        mpsc::{Receiver, SyncSender},
    },
    time::Duration,
};

use esp_idf_sys::cherry_host::{
    usbh_cdc_acm, usbh_cdc_acm_bulk_in_transfer, usbh_cdc_acm_bulk_out_transfer,
    usbh_cdc_acm_set_line_state,
};
use lib_utils::types::AlignedBuffer;

use protocol::{DEFAULT_PACKET_SIZE, devices::WriteThrottleInfo};
use protocol::{
    devices::{ReadThrottleInfo, host::HostProcessor, plugin::PluginProcessor},
    host::{HostReceiver, HostSender},
    plugin::plugin::{PluginReceiver, PluginSender},
};

use esp_idf_sys::cherry_host::usbh_initialize;
use std::{
    marker::PhantomData,
    sync::{atomic::AtomicBool, mpsc::sync_channel},
    thread::Scope,
};

use crate::utils::{TSenderAndReceiver, ThreadSafeCDCWrapper};

static CDC_LOCKER: RwLock<Option<ThreadSafeCDCWrapper>> = RwLock::new(None);
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Strong reference to the cdc runner defined in C
#[unsafe(no_mangle)]
extern "C" fn usbh_cdc_acm_run(cdc_acm_class: *mut usbh_cdc_acm) {
    *CDC_LOCKER.write().unwrap() = Some(ThreadSafeCDCWrapper(cdc_acm_class));
    // TODO: Investigate if setting the line state here causes any issues epecially during the
    // connection setup stage
    unsafe { usbh_cdc_acm_set_line_state(cdc_acm_class, true, false) };
}

/// Strong reference to the cdc acm stopper defined in C
#[unsafe(no_mangle)]
#[allow(unused_variables)]
extern "C" fn usbh_cdc_acm_stop(cdc_acm_class: *mut usbh_cdc_acm) {
    *CDC_LOCKER.write().unwrap() = None;
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
        match IS_INITIALIZED.load(std::sync::atomic::Ordering::Relaxed) {
            true => {
                return Err(());
            }
            false => {}
        }

        match unsafe { usbh_initialize(busid, reg_base as usize) } {
            x if x < 0 => {
                return Err(());
            }
            _ => IS_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed),
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
        match IS_INITIALIZED.load(std::sync::atomic::Ordering::Relaxed) {
            true => {
                return Err(());
            }
            false => {}
        }

        match unsafe { usbh_initialize(busid, reg_base as usize) } {
            x if x < 0 => {
                return Err(());
            }
            _ => IS_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed),
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
        let to_usb = sync_channel(channel_buffer_size);
        let from_usb = sync_channel(channel_buffer_size);

        scope.spawn(move || unsafe { send_usb_data(to_usb.1, write_throttle_info) });
        scope.spawn(move || unsafe { receive_usb_data(from_usb.0) });

        Ok((PluginSender::new(to_usb.0), PluginReceiver::new(from_usb.1)))
    }
}

unsafe fn receive_usb_data(sender: SyncSender<TSenderAndReceiver>) {
    let mut aligned_buffer = AlignedBuffer::<{ size_of::<TSenderAndReceiver>() }>::new();
    loop {
        let cdc_acm_class: *mut usbh_cdc_acm = match CDC_LOCKER.read().unwrap().as_ref() {
            Some(wrapper) => wrapper,
            None => {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
        }
        .0;
        match unsafe {
            usbh_cdc_acm_bulk_in_transfer(
                cdc_acm_class,
                aligned_buffer.as_mut_ptr(),
                aligned_buffer.len() as u32,
                u32::MAX,
            )
        } {
            x if x < 0 => {
                log::error!("Unable to receive the data");
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
    loop {
        // Avoid logging on the write hot path
        let cdc_acm_class: *mut usbh_cdc_acm = match CDC_LOCKER.read().unwrap().as_ref() {
            Some(wrapper) => wrapper,
            None => {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
        }
        .0;

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
                log::error!("Unable to send the data {:?}", x);
                continue;
            }
            _ => {}
        };

        std::thread::sleep(write_throttle_info.delay);
    }
}
