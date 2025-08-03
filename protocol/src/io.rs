//! Contains the basic types to reuse

use crate::{
    errors::{Error, Result},
    DEFAULT_PACKET_SIZE,
};
use heapless::Vec;
use serde::{Deserialize, Serialize};

/// Message type identifier size in bytes
pub const MESSAGE_TYPE_ID_BYTES: usize = 1;

/// Magic number to validate message integrity (0xDEAD)
pub const MESSAGE_MAGIC: u16 = 0xDEAD;
/// Magic number size in bytes
pub const MESSAGE_MAGIC_BYTES: usize = 2;

/// Total header size: magic(2) + type_id(1) + length(2) = 5 bytes
pub const MESSAGE_HEADER_SIZE: usize =
    MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES + DATA_BYTES_LENGTH_IN_BYTES;

/// Message type identifiers for command discrimination
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MessageTypeId {
    /// Host command to configure a peripheral
    HostCommandConfigurePeripheral = 0x01,
    /// Host command to configure a service
    HostCommandConfigureService = 0x02,
    /// Host command to configure a characteristic
    HostCommandConfigureCharacteristic = 0x03,
    /// Host command to read a characteristic
    HostCommandConfigureCharacteristicRead = 0x04,
    /// Host command to get service information
    HostCommandGetServiceInfo = 0x05,
    /// Host command to get characteristic information
    HostCommandGetCharacteristicInfo = 0x06,
    /// Host command to start advertisement
    HostCommandStartAdvertisement = 0x07,
    /// Host command to notify a characteristic value
    HostCommandNotifyCharacteristicValue = 0x08,
    /// Host command to stop advertisement
    PluginData = 0x10,
    /// Plugin configuration error
    PluginConfigurationError = 0x11,
    /// Plugin service information response
    PluginServiceInfoResponse = 0x12,
    /// Plugin characteristic information response
    PluginCharacteristicInfoResponse = 0x13,
}

/// Trait for getting message type identifier
pub trait MessageType {
    /// Get the message type identifier
    fn message_type_id() -> MessageTypeId;
}

/// The size in bytes of the length of the sent and received serialized
/// packet
const DATA_BYTES_LENGTH_IN_BYTES: usize = 2;

/// Communication input and output types
pub trait IO<'a>: Serialize + Deserialize<'a> + Sized + MessageType {
    /// Serialize the host command to a Vec using bincode
    #[inline(always)]
    #[cfg(feature = "std")]
    fn serialize_bytes(&self) -> Result<std::vec::Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
            .map_err(|_| Error::UnableToSerializeToBincode)
    }

    #[inline(always)]
    /// Serialize bytes to a slice
    fn serialize_bytes_in_slice(&self, out: &'a mut [u8]) -> Result<usize> {
        bincode::serde::encode_into_slice(self, out, bincode::config::standard())
            .map_err(|_| Error::UnableToSerializeToBincode)
    }

    /// Convert from bytes
    #[inline(always)]
    fn from_bytes(input: &'a [u8]) -> Result<Self> {
        // Extract length from bytes 2-3 (after magic and type_id)
        let length: usize = (0..DATA_BYTES_LENGTH_IN_BYTES)
            .map(|i| ((input[MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES + i] as usize) << (i * 8)))
            .sum();
        if length > DEFAULT_PACKET_SIZE {
            return Err(Error::UnableToDeserializeFromBincode(
                "Packet size exceeds the allowable limit",
            ));
        }

        let res: Self = bincode::serde::borrow_decode_from_slice(
            &input[MESSAGE_HEADER_SIZE..(MESSAGE_HEADER_SIZE + length)],
            bincode::config::standard(),
        )
        .map_err(|_| Error::UnableToDeserializeFromBincode(""))?
        .0;

        Ok(res)
    }

    /// Serialize to bytes
    #[inline(always)]
    #[cfg(feature = "std")]
    fn to_bytes<const N: usize>(&self) -> Result<[u8; N]> {
        use lib_utils::MatchSliceLengths;

        let mut serialized_bytes = self.serialize_bytes()?;
        let length = serialized_bytes.len() as usize;

        // Create full message header: magic + type_id + length
        let mut header_data: std::vec::Vec<u8> = std::vec::Vec::new();

        // Add magic bytes (0xDEAD)
        header_data.extend_from_slice(&MESSAGE_MAGIC.to_le_bytes());

        // Add message type ID
        header_data.push(Self::message_type_id() as u8);

        // Add length bytes
        header_data
            .extend((0..DATA_BYTES_LENGTH_IN_BYTES).map(|x| ((length >> (x * 8)) & 0xFF) as u8));

        header_data.append(&mut serialized_bytes);

        if header_data.len() > N {
            return Err(Error::SerializationBufferOverflow);
        }
        Ok(header_data.match_size(0x00))
    }

    /// Serialize to bytes
    #[inline(always)]
    fn to_bytes_in_slice<const N: usize>(&self, buffer: &'a mut [u8; N]) -> Result<()> {
        let (header, payload) = buffer.split_at_mut(MESSAGE_HEADER_SIZE);
        let length = self.serialize_bytes_in_slice(payload)? as usize;

        // Create header: magic + type_id + length
        let mut header_bytes: Vec<u8, MESSAGE_HEADER_SIZE> = Vec::new();

        // Add magic bytes (0xDEAD)
        let _ = header_bytes.extend_from_slice(&MESSAGE_MAGIC.to_le_bytes());

        // Add message type ID
        let _ = header_bytes.push(Self::message_type_id() as u8);

        // Add length bytes
        for x in 0..DATA_BYTES_LENGTH_IN_BYTES {
            let _ = header_bytes.push(((length >> (x * 8)) & 0xFF) as u8);
        }

        header.copy_from_slice(&header_bytes);
        if MESSAGE_HEADER_SIZE + length > N {
            return Err(Error::SerializationBufferOverflow);
        }
        Ok(())
    }
}

/// Plugin specific input and output types
pub trait PluginIO<'a>: IO<'a> {}

/// Host specific input and output types
pub trait HostIO<'a>: IO<'a> {}
