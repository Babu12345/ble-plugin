//! Defines the traits for the plugin
//! The peripheral is the device that will connect to the host and receives and transmits data to the primary
//! Typically this can be bluetooth but can really be any other propriary or open source interface if required.
//! The communication protocol between the plugin and the host is typically USB based.

/// Embassy channel send and receive
use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    channel::{Receiver, Sender},
};

use crate::{
    errors::{self, Result},
    host::ReceivedData,
};

use crate::host::THostIO;

/// Sender
pub struct PluginSender<'ch, R: RawMutex, const N: usize, const CH_SIZE: usize>(
    Sender<'ch, R, [u8; N], CH_SIZE>,
);

/// Receiver
pub struct PluginReceiver<'ch, R: RawMutex, const N: usize, const CH_SIZE: usize>(
    Receiver<'ch, R, [u8; N], CH_SIZE>,
);

impl<'a, const N: usize, const CH_SIZE: usize, R: RawMutex> PluginSender<'a, R, N, CH_SIZE> {
    /// Create a new instance
    pub fn new(sender: Sender<'a, R, [u8; N], CH_SIZE>) -> Self {
        Self(sender)
    }

    /// Send the data
    #[cfg(feature = "std")]
    pub async fn send_async<T: THostIO<'a>>(&self, input: T) -> Result<()> {
        self.send_bytes_async(input.to_bytes()?).await
    }

    #[cfg(feature = "std")]
    /// Try sending data
    pub fn try_send<T: THostIO<'a>>(&self, input: T) -> Result<()> {
        self.try_send_bytes(input.to_bytes()?)
    }

    /// Send the data
    pub async fn borrow_send_async<T: for<'b> THostIO<'b>>(&self, input: &T) -> Result<()> {
        let mut buffer = [0; N];
        input.to_bytes_in_slice(&mut buffer)?;
        self.send_bytes_async(buffer).await
    }

    /// Try sending data
    pub fn borrow_try_send<T: for<'b> THostIO<'b>>(&self, input: T) -> Result<()> {
        let mut buffer = [0; N];
        input.to_bytes_in_slice(&mut buffer)?;
        self.try_send_bytes(buffer)
    }

    /// Send bytes directly
    async fn send_bytes_async(&self, buffer: [u8; N]) -> Result<()> {
        self.0.send(buffer).await;
        Ok(())
    }

    /// Try sending bytes directly
    fn try_send_bytes(&self, buffer: [u8; N]) -> Result<()> {
        self.0
            .try_send(buffer)
            .map_err(|_| errors::Error::SendError)
    }
}

impl<'a, const N: usize, const CH_SIZE: usize, R: RawMutex> PluginReceiver<'a, R, N, CH_SIZE> {
    /// Create a new instance
    pub fn new(receiver: Receiver<'a, R, [u8; N], CH_SIZE>) -> Self {
        Self(receiver)
    }

    /// Receive the data
    pub async fn receive(&self) -> Result<ReceivedData<N>> {
        let input = self.0.receive().await;
        Ok(ReceivedData::new(input))
    }
}
