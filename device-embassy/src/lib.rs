//! Library for the usb device embassy implementation

#![no_std]
#![deny(missing_docs)]

pub mod errors;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use protocol::devices::host::AsyncHostProcessor;

use crate::errors::Result;

/// Cdc acm device that implements a AsyncHostProcessor
struct CdcAcmDeviceHost<const CH_SIZE: usize, const BUFFER_SIZE: usize> {}

impl<const CH_SIZE: usize, const BUFFER_SIZE: usize>
    AsyncHostProcessor<CH_SIZE, BUFFER_SIZE, NoopRawMutex, crate::errors::Error>
    for CdcAcmDeviceHost<CH_SIZE, BUFFER_SIZE>
{
    fn processors<'ch, 'b>(
        self,
        channel_buffer_size: usize,
    ) -> Result<(
        protocol::host::AsyncHostSender<'ch, NoopRawMutex, BUFFER_SIZE, CH_SIZE>,
        protocol::host::AsyncHostReceiver<'ch, NoopRawMutex, BUFFER_SIZE, CH_SIZE>,
    )> {
        todo!()
    }
}
