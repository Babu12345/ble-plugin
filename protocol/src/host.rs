//! Host interface protocol to communicate with the plugin device.
use std::sync::mpsc::{Receiver, SyncSender};

use heapless::String;
use lib_utils::MatchSliceLengths;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::{self, Error, Result};
use crate::MAX_NAME_SIZE;

/// Securely stores received data
pub struct ReceivedData<const N: usize>([u8; N]);

/// Sender
pub struct HostSender<const N: usize>(SyncSender<[u8; N]>);

/// Receiver
pub struct HostReceiver<const N: usize>(Receiver<[u8; N]>);

impl<'a, const N: usize> ReceivedData<N> {
    /// Decode the data to the type
    pub fn decode<T: THostIO<'a>>(&'a self) -> Result<T> {
        T::from_bytes(&self.0)
    }
}

impl<'a, const N: usize> HostSender<N> {
    /// Create a new instance
    pub fn new(sender: SyncSender<[u8; N]>) -> Self {
        Self(sender)
    }

    /// Send the data
    pub fn send<T: THostIO<'a>>(&self, input: T) -> Result<()> {
        self.0
            .send(input.to_bytes::<N>()?)
            .map_err(|_| crate::errors::Error::SendError)
    }
}

impl<'a, const N: usize> HostReceiver<N> {
    /// Create a new instance
    pub fn new(receiver: Receiver<[u8; N]>) -> Self {
        Self(receiver)
    }

    /// Receive the data
    pub fn receive(&self) -> Result<ReceivedData<N>> {
        let input = self.0.recv().map_err(|_| errors::Error::ReceiveError)?;
        Ok(ReceivedData(input))
    }
}

/// Acutal host command type
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum HostCommandTypes {
    /// Configure the BLE name
    ConfigPeripheral(String<MAX_NAME_SIZE>, Uuid),
}

/// Host command data
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HostCommand {
    /// Unique command id
    pub id: Uuid,
    /// Actual command type
    pub cmd: HostCommandTypes,
}

/// Host command data in bulk
#[derive(Debug, Deserialize, Serialize)]
pub struct BulkHostCommand {
    /// Commands
    pub commands: Vec<HostCommand>,
}

/// Host data
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct HostData<'a> {
    /// Actual command type
    pub data: &'a [u8],
}

/// Host  data in bulk
#[derive(Debug, Deserialize, Serialize)]
pub struct BulkHostData<'a> {
    /// Data
    #[serde(borrow)]
    data: Vec<HostData<'a>>,
}

/// Communication types
pub trait THostIO<'a>: Serialize + Deserialize<'a> + Sized {
    /// Serialize the host command to a Vec using bincode
    #[inline(always)]
    fn serialize_bytes(&self) -> Result<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
            .map_err(|_| Error::UnableToSerializeToBincode)
    }

    /// Convert from bytes
    #[inline(always)]
    fn from_bytes(input: &'a [u8]) -> Result<Self> {
        bincode::serde::borrow_decode_from_slice(input, bincode::config::standard())
            .map_err(|_| crate::errors::Error::UnableToDeserializeFromBincode)?
            .0
    }

    /// Serialize to bytes
    #[inline(always)]
    fn to_bytes<const N: usize>(&self) -> Result<[u8; N]> {
        let bytes = self.serialize_bytes()?;
        if bytes.len() > N {
            return Err(errors::Error::SerializationBufferOverflow);
        }
        Ok(bytes.match_size(0x00))
    }
}

impl<'a> THostIO<'a> for BulkHostCommand {}

impl<'a> THostIO<'a> for BulkHostData<'a> {}
