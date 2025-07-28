//! Contains the basic types to reuse

use crate::{
    errors::{Error, Result},
    DEFAULT_PACKET_SIZE,
};
use serde::{Deserialize, Serialize};

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
        if length > DEFAULT_PACKET_SIZE {
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

/// Plugin specific input and output types
pub trait PluginIO<'a>: IO<'a> {}

/// Host specific input and output types
pub trait HostIO<'a>: IO<'a> {}
