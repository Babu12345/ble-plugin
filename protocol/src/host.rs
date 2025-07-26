//! Host interface protocol to communicate with the plugin device.

#[cfg(feature = "std")]
pub use self::host_std::*;

/// Std sync send and receive
#[cfg(feature = "std")]
mod host_std {
    use crate::{
        errors::{self, Result},
        types::{HostIO, PluginReceivedData},
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
        pub fn receive(&self) -> Result<PluginReceivedData<N>> {
            let input = self.0.recv().map_err(|_| errors::Error::ReceiveError)?;
            Ok(PluginReceivedData::new(input))
        }
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use crate::types::{PluginReceivedData, IO};
    use crate::{types::HostCommandConfigurePeripheral, MAX_TRANSFER_SIZE};
    use heapless::String;
    use uuid::Uuid;

    #[test]
    fn test_std_encoding_and_decoding() {
        let cmd = HostCommandConfigurePeripheral {
            name: String::from_str("Hello").unwrap(),
            uuid: Uuid::from_u128(0x01),
        };
        let data: [u8; MAX_TRANSFER_SIZE] = cmd.to_bytes().unwrap();
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
            uuid: Uuid::from_u128(0x01),
        };
        let mut buffer = [0u8; MAX_TRANSFER_SIZE];
        cmd.to_bytes_in_slice(&mut buffer).unwrap();
        let decoded_cmd: HostCommandConfigurePeripheral =
            PluginReceivedData::new(buffer).decode().unwrap();
        assert_eq!(
            cmd, decoded_cmd,
            "Testing a single command being encoded and decoded"
        );
    }
}
