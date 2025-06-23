//! Host interface protocol to communicate with the plugin device.

use lib_utils::MatchSliceLengths;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};

use crate::errors::{self, Error, Result};

/// Acutal host command type
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum HostCommandTypes<'a> {
    /// Configure the BLE name
    ConfigPeripheral(&'a str, u32),
}

/// Host command data
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct HostCommand<'a> {
    /// Actual command type
    #[serde(borrow)]
    pub cmd: HostCommandTypes<'a>,
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
        Ok(writer)
    }
}
