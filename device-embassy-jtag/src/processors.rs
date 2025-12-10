// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

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

impl<'d, const CH_SIZE: usize, M: RawMutex>
    AsyncHostProcessor<CH_SIZE, BUFFER_SIZE, M, crate::errors::Error>
    for DeviceHostJtag<'d, CH_SIZE, BUFFER_SIZE, M>
{
    type T<'ch>
        = (
        Sender<'ch, M, [u8; BUFFER_SIZE], CH_SIZE>,
        Receiver<'ch, M, [u8; BUFFER_SIZE], CH_SIZE>,
    )
    where
        M: 'ch;

    fn processors<'ch>(
        self,
        to: Self::T<'ch>,
        from: Self::T<'ch>,
    ) -> Result<(
        impl Future<Output = ()>,
        AsyncHostSender<'ch, M, BUFFER_SIZE, CH_SIZE>,
        AsyncHostReceiver<'ch, M, BUFFER_SIZE, CH_SIZE>,
    )> {
        let (mut rx, mut tx) = self.jtag.split();

        let processor_runner = async move {
            let to_jtag_fn = async {
                Timer::after_millis(100).await;
                log::info!("JTAG sender connection established");
                loop {
                    let data = to.1.receive().await;
                    match tx.write_all(&data).await {
                        Ok(_) => {
                            let _ = tx.flush().await;
                        }
                        Err(_) => {
                            log::warn!("JTAG write error, continuing");
                        }
                    }
                }
            };

            let from_jtag_fn = async {
                Timer::after_millis(100).await;
                log::info!("JTAG receiver connection established");
                if let Some(signal) = &self.receiver_connected {
                    signal.signal(true);
                }
                let mut buf = [0; BUFFER_SIZE];
                loop {
                    match rx.read(&mut buf).await {
                        Ok(_len) => {
                            from.0.send(buf).await;
                            buf = [0; BUFFER_SIZE];
                        }
                        Err(_) => {
                            log::warn!("JTAG read error, continuing");
                            if let Some(signal) = &self.receiver_connected {
                                signal.signal(false);
                            }
                        }
                    }
                }
            };

            join(to_jtag_fn, from_jtag_fn).await;
        };

        Ok((
            processor_runner,
            AsyncHostSender::new(to.0),
            AsyncHostReceiver::new(from.1),
        ))
    }
}
