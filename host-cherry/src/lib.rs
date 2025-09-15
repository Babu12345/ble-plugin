//! USB host implementation library of cherry usb
//!
//! Uses `heapless` for internal buffers. Protocol types use `alloc::Vec` and
//! `alloc::String` for Protocol Buffer compatibility.

#![cfg(all(target_arch = "xtensa", target_os = "espidf"))]
#[deny(missing_docs)]
mod constants;
mod processors;
mod utils;
use processors::*;
use protocol::{
    devices::{ReadThrottleInfo, WriteThrottleInfo, host::HostProcessor, plugin::PluginProcessor},
    host::{HostReceiver, HostSender},
    plugin::plugin::{PluginReceiver, PluginSender},
};

use esp_idf_sys::cherry_host::usbh_initialize;
use protocol::DEFAULT_PACKET_SIZE;
use std::{
    marker::PhantomData,
    sync::{atomic::AtomicBool, mpsc::sync_channel},
    thread::Scope,
    time::Duration,
};
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
// Initialization
// https://github.com/zleihao/CherryUSB-CDC-MSC/blob/50095e0b63bbdf6f2d5597e71edfa45dd8be6c1d/cdc_msc/middlewares/CherryUSB-1.4.0/class/cdc/usbh_cdc_acm.c#L170
// https://github.com/cherry-embedded/CherryUSB/blob/f23f5494920b64987350abc87c8154f410c6f5f9/platform/nuttx/usbh_serial.c#L180
// https://github.com/search?q=repo%3Acherry-embedded%2FCherryUSB%20usbh_cdc_acm_run&type=code
// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/host/cdc_acm/src/cdc_acm_chost.c#L33
// https://github.com/CherryUSB/cherryusb_esp32/blob/main/examples/host/sdkconfig

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
        _write_throttle_info: WriteThrottleInfo,
    ) -> Result<
        (
            HostSender<DEFAULT_PACKET_SIZE>,
            HostReceiver<DEFAULT_PACKET_SIZE>,
        ),
        (),
    > {
        let to_usb = sync_channel(channel_buffer_size);
        let from_usb = sync_channel(channel_buffer_size);

        scope.spawn(move || unsafe { send_usb_data(to_usb.1) });
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
        _write_throttle_info: WriteThrottleInfo,
    ) -> Result<
        (
            PluginSender<DEFAULT_PACKET_SIZE>,
            PluginReceiver<DEFAULT_PACKET_SIZE>,
        ),
        (),
    > {
        let to_usb = sync_channel(channel_buffer_size);
        let from_usb = sync_channel(channel_buffer_size);

        scope.spawn(move || unsafe { send_usb_data(to_usb.1) });
        scope.spawn(move || unsafe { receive_usb_data(from_usb.0) });

        Ok((PluginSender::new(to_usb.0), PluginReceiver::new(from_usb.1)))
    }
}
