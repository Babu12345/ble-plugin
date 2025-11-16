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
use esp_hal::{uart::Uart, Async, DriverMode};
use protocol::{
    devices::host::AsyncHostProcessor,
    host::{AsyncHostReceiver, AsyncHostSender},
};

use crate::errors::Result;

const BUFFER_SIZE: usize = 64;

/// Device host for UART communication
pub struct DeviceHostUart<
    'd,
    Dm: DriverMode,
    const CH_SIZE: usize,
    const BUFFER_SIZE: usize,
    M: RawMutex,
> {
    uart: Uart<'d, Dm>,
    receiver_connected: Option<&'d Signal<M, bool>>,
}

impl<'d, Dm: DriverMode, const CH_SIZE: usize, const BUFFER_SIZE: usize, M: RawMutex>
    DeviceHostUart<'d, Dm, CH_SIZE, BUFFER_SIZE, M>
{
    /// Create a new instance
    pub fn new(uart: Uart<'d, Dm>) -> Self {
        Self {
            uart,
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
    for DeviceHostUart<'d, Async, CH_SIZE, BUFFER_SIZE, M>
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
        let (mut rx, mut tx) = self.uart.split();

        let processor_runner = async move {
            let to_uart_fn = async {
                Timer::after_millis(100).await;
                log::info!("UART sender connection established");
                loop {
                    let data = to.1.receive().await;
                    match tx.write_all(&data).await {
                        Ok(_) => {
                            let _ = tx.flush_async().await;
                        }
                        Err(_) => {
                            log::warn!("UART write error, continuing");
                        }
                    }
                }
            };

            let from_uart_fn = async {
                Timer::after_millis(100).await;
                log::info!("UART receiver connection established");
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
                            log::warn!("UART read error, continuing");
                            if let Some(signal) = &self.receiver_connected {
                                signal.signal(false);
                            }
                        }
                    }
                }
            };

            join(to_uart_fn, from_uart_fn).await;
        };

        Ok((
            processor_runner,
            AsyncHostSender::new(to.0),
            AsyncHostReceiver::new(from.1),
        ))
    }
}
