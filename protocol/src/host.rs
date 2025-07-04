//! Host interface protocol to communicate with the plugin device.
use serde::{Deserialize, Serialize};

use crate::errors::{self, Error, Result};
use crate::types::{BulkHostCommand, BulkHostData};
use crate::MAX_TRANSFER_SIZE;

#[cfg(feature = "std")]
pub use self::host_std::*;

/// Securely stores received data
pub struct ReceivedData<const N: usize>([u8; N]);

/// Host std types and functions
#[cfg(feature = "std")]
mod host_std {
    use crate::{
        errors::{self, Result},
        host::{ReceivedData, THostIO},
    };
    use std::sync::mpsc::{Receiver, SyncSender};
    /// Sender
    pub struct HostSender<const N: usize>(SyncSender<[u8; N]>);

    /// Receiver
    pub struct HostReceiver<const N: usize>(Receiver<[u8; N]>);

    impl<'a, const N: usize> HostSender<N> {
        /// Create a new instance
        pub fn new(sender: SyncSender<[u8; N]>) -> Self {
            Self(sender)
        }

        /// Send the data
        pub fn send<T: THostIO<'a>>(&self, input: T) -> Result<()> {
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
}

impl<'a, const N: usize> ReceivedData<N> {
    /// Decode the data to the type
    pub fn decode<T: THostIO<'a>>(&'a self) -> Result<T> {
        T::from_bytes(&self.0)
    }
}

/// Communication types
pub trait THostIO<'a>: Serialize + Deserialize<'a> + Sized {
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
    #[cfg(feature = "std")]
    fn to_bytes<const N: usize>(&self) -> Result<[u8; N]> {
        use lib_utils::MatchSliceLengths;

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

    /// Serialize to bytes
    #[inline(always)]
    fn to_bytes_in_slice<const N: usize>(&self, buffer: &'a mut [u8; N]) -> Result<()> {
        let (left, right) = buffer.split_at_mut(2);
        let length = self.serialize_bytes_in_slice(right)? as u16;
        let length_data = [(length & 0xFF) as u8, ((length >> 8) & 0xFF) as u8];
        left.copy_from_slice(&length_data);
        if length_data.len() > N {
            return Err(errors::Error::SerializationBufferOverflow);
        }
        Ok(())
    }
}

impl<'a> THostIO<'a> for BulkHostCommand {}
impl<'a> THostIO<'a> for BulkHostData<'a> {}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;
    use crate::types::HostCommand;
    use heapless::{String, Vec};
    use uuid::Uuid;

    #[test]
    fn test_std_encoding_and_decoding() {
        let cmd = BulkHostCommand {
            commands: Vec::from_slice(&[HostCommand {
                uuid: Uuid::from_u128(0x01),
                cmd: crate::types::HostCommandTypes::ConfigService,
            }])
            .unwrap(),
        };
        let data: [u8; 256] = cmd.to_bytes().unwrap();
        let decoded_cmd = BulkHostCommand::from_bytes(&data).unwrap();
        assert_eq!(
            cmd, decoded_cmd,
            "Testing a single command being encoded and decoded"
        );

        let commands: Vec<HostCommand, 10> = (0..10)
            .map(|x| HostCommand {
                uuid: Uuid::from_u128(x),
                cmd: crate::types::HostCommandTypes::ConfigPeripheral(
                    String::from_str("Test name").unwrap(),
                    Uuid::from_u128(0x02),
                ),
            })
            .collect();

        let cmd = BulkHostCommand { commands };
        let data: [u8; 512] = cmd.to_bytes().unwrap();
        let decoded_cmd = BulkHostCommand::from_bytes(&data).unwrap();
        assert_eq!(
            cmd, decoded_cmd,
            "Testing a bulk send of commands being encoded and decoded"
        );
    }

    #[test]
    fn test_no_std_encoding_and_decoding() {
        let cmd = BulkHostCommand {
            commands: Vec::from_slice(&[HostCommand {
                uuid: Uuid::from_u128(0x01),
                cmd: crate::types::HostCommandTypes::ConfigService,
            }])
            .unwrap(),
        };
        let mut buffer = [0u8; 256];
        cmd.to_bytes_in_slice(&mut buffer).unwrap();
        let decoded_cmd = BulkHostCommand::from_bytes(&buffer).unwrap();
        assert_eq!(
            cmd, decoded_cmd,
            "Testing a single command being encoded and decoded"
        );

        let commands: Vec<HostCommand, 10> = (0..10)
            .map(|x| HostCommand {
                uuid: Uuid::from_u128(x),
                cmd: crate::types::HostCommandTypes::ConfigPeripheral(
                    String::from_str("Test name").unwrap(),
                    Uuid::from_u128(0x02),
                ),
            })
            .collect();

        let cmd = BulkHostCommand { commands };

        let mut buffer = [0u8; 512];
        cmd.to_bytes_in_slice(&mut buffer).unwrap();
        let decoded_cmd = BulkHostCommand::from_bytes(&buffer).unwrap();
        assert_eq!(
            cmd, decoded_cmd,
            "Testing a bulk send of commands being encoded and decoded"
        );
    }
}
