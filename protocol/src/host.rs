//! Host interface protocol to communicate with the plugin device.
use std::fmt::Debug;
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
    pub fn decode<T: THostIO<'a, N>>(&'a self) -> Result<T> {
        T::from_bytes(&self.0)
    }
}

impl<'a, const N: usize> HostSender<N> {
    /// Create a new instance
    pub fn new(sender: SyncSender<[u8; N]>) -> Self {
        Self(sender)
    }

    /// Send the data
    pub fn send<T: THostIO<'a, N>>(&self, input: T) -> Result<()> {
        self.0
            .send(input.to_bytes()?)
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
    pub uuid: Uuid,
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
pub trait THostIO<'a, const N: usize>: Serialize + Deserialize<'a> + Sized + Debug {
    /// Serialize the host command to a Vec using bincode
    #[inline(always)]
    fn serialize_bytes(&self) -> Result<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
            .map_err(|_| Error::UnableToSerializeToBincode)
    }

    /// Convert from bytes
    #[inline(always)]
    fn from_bytes(input: &'a [u8]) -> Result<Self> {
        let length_lsb = input[0] as u16;
        let length_msb = input[1] as u16;
        let length = (((length_msb << 8) & 0xFF00) + (length_lsb & 0x00FF)) as usize;
        if length > 512 {
            return Err(crate::errors::Error::UnableToDeserializeFromBincode);
        }

        let res: Self = bincode::serde::borrow_decode_from_slice(
            &input[2..(2 + length)],
            bincode::config::standard(),
        )
        .map_err(|_| errors::Error::UnableToDeserializeFromBincode)?
        .0;

        Ok(res)
    }

    /// Serialize to bytes
    #[inline(always)]
    fn to_bytes(&self) -> Result<[u8; N]> {
        let mut serialized_bytes = self.serialize_bytes()?;
        let length = serialized_bytes.len() as u16;
        let mut length_data: Vec<u8> = Vec::from([length & 0xFF, (length >> 8) & 0xFF])
            .into_iter()
            .map(|x| x as u8)
            .collect();
        length_data.append(&mut serialized_bytes);

        if length_data.len() > N {
            return Err(errors::Error::SerializationBufferOverflow);
        }
        Ok(length_data.match_size(0x00))
    }
}

impl<'a, const N: usize> THostIO<'a, N> for BulkHostCommand {}

impl<'a, const N: usize> THostIO<'a, N> for BulkHostData<'a> {}
