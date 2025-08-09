//! Host interface protocol to communicate with the plugin device.

#[cfg(feature = "std")]
pub use self::host_std::*;
pub use common::*;

/// Common types and traits
mod common {
    use crate::{errors::Result, PluginIO};

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
}

/// Std sync send and receive
#[cfg(feature = "std")]
mod host_std {
    use super::*;
    use crate::{
        errors::{self, Result},
        HostIO,
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
        pub fn send<T: HostIO<'a>>(&self, input: T) -> Result<()> {
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
        pub fn receive(&self) -> Result<HostReceivedData<N>> {
            let input = self.0.recv().map_err(|_| errors::Error::ReceiveError)?;
            Ok(HostReceivedData::new(input))
        }
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use crate::plugin::PluginReceivedData;
    use crate::IO;
    use crate::{io_types::HostCommandConfigurePeripheral, DEFAULT_PACKET_SIZE};
    use heapless::String;

    #[test]
    fn test_std_encoding_and_decoding() {
        let cmd = HostCommandConfigurePeripheral {
            name: String::from_str("Hello").unwrap(),
            addr: [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        };
        let data: [u8; DEFAULT_PACKET_SIZE] = cmd.to_bytes().unwrap();
        let decoded_cmd: HostCommandConfigurePeripheral =
            PluginReceivedData::new(data).decode().unwrap();

        assert_eq!(
            cmd, decoded_cmd,
            "Testing a single command being encoded and decoded"
        );
    }

    #[test]
    fn test_no_std_encoding_and_decoding() {
        let cmd = HostCommandConfigurePeripheral {
            name: String::from_str("Hello").unwrap(),
            addr: [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        };
        let mut buffer = [0u8; DEFAULT_PACKET_SIZE];
        cmd.to_bytes_in_slice(&mut buffer).unwrap();
        let decoded_cmd: HostCommandConfigurePeripheral =
            PluginReceivedData::new(buffer).decode().unwrap();
        assert_eq!(
            cmd, decoded_cmd,
            "Testing a single command being encoded and decoded"
        );
    }
}
