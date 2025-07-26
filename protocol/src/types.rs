//! Contains the basic types to reuse

use crate::{
    errors::{Error, Result},
    MAX_TRANSFER_SIZE,
};
use serde::{Deserialize, Serialize};

pub use host::*;
pub use plugin::*;

/// Communication input and output types
pub trait IO<'a>: Serialize + Deserialize<'a> + Sized {
    /// The size in bytes of the length of the sent and received serialized
    /// packet
    const DATA_BYTES_LENGTH_IN_BYTES: usize = 2;

    /// Serialize the host command to a Vec using bincode
    #[inline(always)]
    #[cfg(feature = "std")]
    fn serialize_bytes(&self) -> Result<Vec<u8>> {
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
        let length_lsb = input[0] as u16;
        let length_msb = input[1] as u16;
        let length = (((length_msb << 8) & 0xFF00) + (length_lsb & 0x00FF)) as usize;
        if length > MAX_TRANSFER_SIZE {
            return Err(Error::UnableToDeserializeFromBincode);
        }

        let res: Self = bincode::serde::borrow_decode_from_slice(
            &input[2..(2 + length)],
            bincode::config::standard(),
        )
        .map_err(|_| Error::UnableToDeserializeFromBincode)?
        .0;

        Ok(res)
    }

    /// Serialize to bytes
    #[inline(always)]
    #[cfg(feature = "std")]
    fn to_bytes<const N: usize>(&self) -> Result<[u8; N]> {
        use lib_utils::MatchSliceLengths;

        let mut serialized_bytes = self.serialize_bytes()?;
        let length = serialized_bytes.len() as u16;

        let mut length_data: Vec<u8> = (0..Self::DATA_BYTES_LENGTH_IN_BYTES)
            .map(|x| ((length >> (x * 8)) & 0xFF) as u8)
            .collect();

        length_data.append(&mut serialized_bytes);

        if length_data.len() > N {
            return Err(Error::SerializationBufferOverflow);
        }
        Ok(length_data.match_size(0x00))
    }

    /// Serialize to bytes
    #[inline(always)]
    fn to_bytes_in_slice<const N: usize>(&self, buffer: &'a mut [u8; N]) -> Result<()> {
        let (left, right) = buffer.split_at_mut(Self::DATA_BYTES_LENGTH_IN_BYTES);
        let length = self.serialize_bytes_in_slice(right)? as u16;
        let length_data = [(length & 0xFF) as u8, ((length >> 8) & 0xFF) as u8];
        left.copy_from_slice(&length_data);
        if length_data.len() > N {
            return Err(Error::SerializationBufferOverflow);
        }
        Ok(())
    }
}

/// Host types
pub mod host {
    use super::*;
    use crate::MAX_NAME_SIZE;
    use heapless::String;
    use protocol_io::HostIO;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    /// Host specific input and output types
    pub trait HostIO<'a>: IO<'a> {}

    /// Securely stores received data
    pub struct HostReceivedData<const N: usize>([u8; N]);

    impl<'a, const N: usize> HostReceivedData<N> {
        /// Create a new ReceivedData struct that can be used for decoding
        pub fn new(input: [u8; N]) -> Self {
            Self(input)
        }

        /// Decode the data to the type
        pub fn decode<T: PluginIO<'a>>(&'a self) -> Result<T> {
            T::from_bytes(&self.0)
        }
    }

    /// Host command. Configure peripheral
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandConfigurePeripheral {
        /// Peripheral name
        pub name: String<MAX_NAME_SIZE>,
        /// Peripheral UUID
        pub uuid: Uuid,
    }

    /// Host command. Configure peripheral
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandConfigureService {}
}

/// Plugin types
pub mod plugin {
    use protocol_io::PluginIO;
    use uuid::Uuid;

    use super::*;
    use crate::types::IO;

    /// Securely stores received data
    pub struct PluginReceivedData<const N: usize>([u8; N]);

    impl<'a, const N: usize> PluginReceivedData<N> {
        /// Create a new ReceivedData struct that can be used for decoding
        pub fn new(input: [u8; N]) -> Self {
            Self(input)
        }

        /// Decode the data to the type
        pub fn decode<T: HostIO<'a>>(&'a self) -> Result<T> {
            T::from_bytes(&self.0)
        }
    }

    /// Plugin specific input and output types
    pub trait PluginIO<'a>: IO<'a> {}

    /// Represents the send type of the data. Was it due to a
    /// write event (central -> peripheral), notify event (peripheral -> client),
    /// or read attempt (central -> peripheral). Depending on which a response
    /// might be expected or sent
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[repr(u8)]
    pub enum PluginDataSendType {
        /// Notified from the central bluetooth device
        Notify,
        /// Read attempt from the central bluetooth device
        Read,
        /// Written from the central bluetooth device
        Write,
    }

    /// Plugin data
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PluginIO)]
    pub struct PluginData<'a> {
        /// Source peripheral id that this data is orginating from.
        pub src_id: Uuid,
        /// Send type of the data
        pub send_type: PluginDataSendType,
        /// Actual command type
        pub data: &'a [u8],
    }
}
