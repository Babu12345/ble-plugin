//! Processor crate

use core::future::Future;

use embassy_futures::join::join;
use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    channel::{Receiver, Sender},
    signal::Signal,
};
use embassy_time::Timer;
use embedded_io_async::{Read, Write};
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use protocol::{
    devices::host::AsyncHostProcessor,
    host::{AsyncHostReceiver, AsyncHostSender},
};

use crate::errors::Result;

const BUFFER_SIZE: usize = 64;

/// Device host for jtag communication
pub struct DeviceHostJtag<'d, const CH_SIZE: usize, const BUFFER_SIZE: usize, M: RawMutex> {
    jtag: UsbSerialJtag<'d, esp_hal::Async>,
    receiver_connected: Option<&'d Signal<M, bool>>,
}

impl<'d, const CH_SIZE: usize, const BUFFER_SIZE: usize, M: RawMutex>
    DeviceHostJtag<'d, CH_SIZE, BUFFER_SIZE, M>
{
    /// Create a new instance
    pub fn new(jtag: UsbSerialJtag<'d, esp_hal::Async>) -> Self {
        Self {
            jtag,
            receiver_connected: None,
        }
    }

    /// Add a signal for receiver connection
    pub fn add_connection_signal(mut self, signal: &'d Signal<M, bool>) -> Self {
        self.receiver_connected = Some(signal);
        self
    }
}
