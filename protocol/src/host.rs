//! Host interface protocol to communicate with the plugin device.

use std::sync::mpsc::{Receiver, SyncSender};

use lib_utils::MatchSliceLengths;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::{self, Error, Result};

/// Usb host input/output
pub struct HostIO<const N: usize> {
    /// USB sender
    sender: SyncSender<[u8; N]>,
    /// USB receiver
    receiver: Receiver<[u8; N]>,
}

impl<'a, const N: usize> HostIO<N> {
    /// Create a new instance
    pub fn new(sender: SyncSender<[u8; N]>, receiver: Receiver<[u8; N]>) -> Self {
        Self { sender, receiver }
    }

    /// Send the data
    pub fn send<T: THostIO<'a>>(&self, input: T) -> Result<()> {
        self.sender
            .send(input.to_bytes::<N>()?)
            .map_err(|_| crate::errors::Error::SendError)
    }

    /// Receive the data
    pub fn receive<T: THostIO<'a>>(&self, input: [u8; N]) -> Result<()> {
        // let input = self.receiver.recv().unwrap();
        // T::from_bytes(input.);
        todo!()
    }
}

/// Acutal host command type
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum HostCommandTypes<'a> {
    /// Configure the BLE name
    ConfigPeripheral(&'a str, Uuid),
}

/// Host command data
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct HostCommand<'a> {
    /// Actual command type
    #[serde(borrow)]
    pub cmd: HostCommandTypes<'a>,
}

/// Host command data in bulk
#[derive(Debug, Deserialize, Serialize)]
pub struct BulkHostCommand<'a> {
    /// Commands
    #[serde(borrow)]
    pub commands: Vec<HostCommand<'a>>,
}

/// Host data
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct HostData<'a> {
    /// Actual command type
    pub output: &'a [u8],
}

/// Host  data in bulk
#[derive(Debug, Deserialize, Serialize)]
pub struct BulkHostData<'a> {
    /// Data
    #[serde(borrow)]
    data: Vec<HostData<'a>>,
}

/// Communication types
pub trait THostIO<'a> {
    /// Serialize to bytes
    fn to_bytes<const N: usize>(&self) -> Result<[u8; N]>;
    /// Deserialize back to the type
    fn from_bytes<'input: 'a>(input: &'input [u8]) -> Result<Self>
    where
        Self: Sized;
}

impl<'a> THostIO<'a> for BulkHostCommand<'a> {
    /// Convert to bytes
    fn to_bytes<const N: usize>(&self) -> Result<[u8; N]> {
        let bytes = self.serialize_bytes()?;
        if bytes.len() > N {
            return Err(errors::Error::SerializationBufferOverflow);
        }
        Ok(bytes.match_size(0))
    }

    /// Convert from bytes
    fn from_bytes<'input: 'a>(input: &'input [u8]) -> Result<Self> {
        rmp_serde::from_slice(input).map_err(|_| crate::errors::Error::UnableToDeserializeFromRMP)
    }
}

impl<'a> BulkHostCommand<'a> {
    /// Serialize the host command to a Vec using RMP
    #[inline(always)]
    fn serialize_bytes(&self) -> Result<Vec<u8>> {
        let mut writer = Vec::new();
        self.serialize(&mut Serializer::new(&mut writer))
            .map_err(|_| Error::UnableToSerializeToRMP)?;
        // bincode::serde::encode_to_vec(self, bincode::config::standard()).unwrap();
        Ok(writer)
    }
}

impl<'a> THostIO<'a> for HostCommand<'a> {
    /// Convert to bytes
    fn to_bytes<const N: usize>(&self) -> Result<[u8; N]> {
        let bytes = self.serialize_bytes()?;
        if bytes.len() > N {
            return Err(errors::Error::SerializationBufferOverflow);
        }
        Ok(bytes.match_size(0))
    }

    /// Convert from bytes
    fn from_bytes<'input: 'a>(input: &'input [u8]) -> Result<Self> {
        rmp_serde::from_slice(input).map_err(|_| crate::errors::Error::UnableToDeserializeFromRMP)
    }
}

impl<'a> HostCommand<'a> {
    /// Serialize the host command to a Vec using RMP
    #[inline(always)]
    fn serialize_bytes(&self) -> Result<Vec<u8>> {
        let mut writer = Vec::new();
        self.serialize(&mut Serializer::new(&mut writer))
            .map_err(|_| Error::UnableToSerializeToRMP)?;
        // bincode::serde::encode_to_vec(self, bincode::config::standard()).unwrap();
        Ok(writer)
    }
}

impl<'a> BulkHostData<'a> {
    /// Serialize the host command to a Vec using RMP
    #[inline(always)]
    fn serialize_bytes(&self) -> Result<Vec<u8>> {
        let mut writer = Vec::new();
        self.serialize(&mut Serializer::new(&mut writer))
            .map_err(|_| Error::UnableToSerializeToRMP)?;
        Ok(writer)
    }
}

impl<'a> THostIO<'a> for BulkHostData<'a> {
    /// Convert to bytes
    fn to_bytes<const N: usize>(&self) -> Result<[u8; N]> {
        let bytes = self.serialize_bytes()?;
        if bytes.len() > N {
            return Err(errors::Error::SerializationBufferOverflow);
        }
        Ok(bytes.match_size(0))
    }

    /// Convert from bytes
    fn from_bytes<'input: 'a>(input: &'input [u8]) -> Result<Self> {
        rmp_serde::from_slice(input).map_err(|_| crate::errors::Error::UnableToDeserializeFromRMP)
    }
}
