//! Host interface protocol to communicate with the plugin device.

#[cfg(feature = "std")]
pub use self::host_std::*;
pub use async_host::*;
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

/// Async implementation  
pub mod async_host {
    use crate::host::HostReceivedData;
    use embassy_sync::{
        blocking_mutex::raw::RawMutex,
        channel::{Receiver, Sender},
    };

    /// Async sender
    pub struct AsyncHostSender<'ch, R: RawMutex, const N: usize, const CH_SIZE: usize>(
        Sender<'ch, R, [u8; N], CH_SIZE>,
    );

    /// Async receiver
    pub struct AsyncHostReceiver<'ch, R: RawMutex, const N: usize, const CH_SIZE: usize>(
        Receiver<'ch, R, [u8; N], CH_SIZE>,
    );

    impl<'a, const N: usize, const CH_SIZE: usize, R: RawMutex> AsyncHostSender<'a, R, N, CH_SIZE> {
        /// Create a new instance
        pub fn new(sender: Sender<'a, R, [u8; N], CH_SIZE>) -> Self {
            Self(sender)
        }

        /// Send the data
        #[cfg(feature = "std")]
        pub async fn send_async<T: crate::IO<'a>>(&self, input: T) -> crate::errors::Result<()> {
            self.send_bytes_async(input.to_bytes()?).await
        }

        #[cfg(feature = "std")]
        /// Try sending data
        pub fn try_send<T: crate::IO<'a>>(&self, input: T) -> crate::errors::Result<()> {
            self.try_send_bytes(input.to_bytes()?)
        }

        /// Send the data
        pub async fn borrow_send_async<T: for<'b> crate::IO<'b>>(&self, input: &T) -> crate::errors::Result<()> {
            let mut buffer = [0; N];
            input.to_bytes_in_slice(&mut buffer)?;
            self.send_bytes_async(buffer).await
        }

        /// Try sending data
        pub fn borrow_try_send<T: for<'b> crate::IO<'b>>(&self, input: T) -> crate::errors::Result<()> {
            let mut buffer = [0; N];
            input.to_bytes_in_slice(&mut buffer)?;
            self.try_send_bytes(buffer)
        }

        /// Send bytes directly
        async fn send_bytes_async(&self, buffer: [u8; N]) -> crate::errors::Result<()> {
            self.0.send(buffer).await;
            Ok(())
        }

        /// Try sending bytes directly
        fn try_send_bytes(&self, buffer: [u8; N]) -> crate::errors::Result<()> {
            self.0
                .try_send(buffer)
                .map_err(|_| crate::errors::Error::SendError)
        }
    }

    impl<'a, const N: usize, const CH_SIZE: usize, R: RawMutex> AsyncHostReceiver<'a, R, N, CH_SIZE> {
        /// Create a new instance
        pub fn new(receiver: Receiver<'a, R, [u8; N], CH_SIZE>) -> Self {
            Self(receiver)
        }

        /// Receive the data
        pub async fn receive(&self) -> crate::errors::Result<HostReceivedData<N>> {
            let input = self.0.receive().await;
            Ok(HostReceivedData::new(input))
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use core::str::FromStr;
    use std::{string::String, vec::Vec};
    #[cfg(feature = "std")]
    use std::sync::mpsc;

    use crate::plugin::PluginReceivedData;
    use crate::protocol::*;
    use crate::DEFAULT_PACKET_SIZE;
    use crate::IO;

    #[test]
    #[cfg(feature = "std")]
    fn test_std_encoding_and_decoding() {
        let cmd = HostCommandConfigurePeripheral {
            name: String::from_str("Hello").unwrap(),
            addr: Vec::from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
        };
        let data: [u8; DEFAULT_PACKET_SIZE] = cmd.to_bytes().unwrap();
        let received_data = PluginReceivedData::new(data);
        let decoded_cmd: HostCommandConfigurePeripheral = received_data.decode().unwrap();

        assert_eq!(
            cmd, decoded_cmd,
            "Testing a single command being encoded and decoded"
        );
    }

    #[test]
    fn test_no_std_encoding_and_decoding() {
        let cmd = HostCommandConfigurePeripheral {
            name: String::from_str("Hello").unwrap(),
            addr: Vec::from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
        };
        let mut buffer = [0u8; DEFAULT_PACKET_SIZE];
        cmd.to_bytes_in_slice(&mut buffer).unwrap();
        let received_data = PluginReceivedData::new(buffer);
        let decoded_cmd: HostCommandConfigurePeripheral = received_data.decode().unwrap();
        assert_eq!(
            cmd, decoded_cmd,
            "Testing a single command being encoded and decoded"
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_host_sender_and_plugin_receiver_communication() {
        use super::*;
        use crate::plugin::PluginReceivedData;

        // Create a channel for host-to-plugin communication
        let (tx, rx) = mpsc::sync_channel::<[u8; DEFAULT_PACKET_SIZE]>(10);
        
        // Create host sender and plugin receiver  
        let host_sender = HostSender::new(tx);

        // Create test command (host sends commands)
        let cmd = HostCommandConfigurePeripheral {
            name: String::from_str("TestDevice").unwrap(),
            addr: Vec::from(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]),
        };

        // Send command through host sender
        host_sender.send(cmd.clone()).expect("Should send successfully");

        // Receive raw data from channel
        let raw_data = rx.recv().expect("Should receive raw data");
        let plugin_received_data = PluginReceivedData::new(raw_data);
        let decoded_cmd: HostCommandConfigurePeripheral = plugin_received_data.decode().expect("Should decode successfully");

        assert_eq!(cmd, decoded_cmd, "Sent and received commands should match");
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_host_receiver_with_plugin_data() {
        use super::*;

        // Create a channel for plugin-to-host communication
        let (tx, rx) = mpsc::sync_channel::<[u8; DEFAULT_PACKET_SIZE]>(10);
        let host_receiver = HostReceiver::new(rx);

        // Create test plugin data (plugin sends data to host)
        let plugin_data = PluginData {
            src_addr: Vec::from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
            src_addr_type: BluetoothAddressType::Public as _,
            send_type: PluginDataSendType::NotifyType as _,
            characteristic_uuid: 0x2A29,
            service_uuid: 0x180A,
            data: Vec::from(b"Hello from plugin"),
        };

        // Serialize and send data as if from plugin
        let serialized_data: [u8; DEFAULT_PACKET_SIZE] = plugin_data.to_bytes().unwrap();
        tx.send(serialized_data).expect("Should send plugin data");

        // Receive and decode through host receiver
        let received_data = host_receiver.receive().expect("Should receive successfully");
        let decoded_data: PluginData = received_data.decode().expect("Should decode successfully");

        assert_eq!(plugin_data, decoded_data, "Sent and received data should match");
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_host_receiver_empty_channel() {
        use super::*;

        let (tx, rx) = mpsc::sync_channel::<[u8; DEFAULT_PACKET_SIZE]>(10);
        let host_receiver = HostReceiver::new(rx);

        // Drop sender to close channel
        drop(tx);

        // Should get receive error
        let result = host_receiver.receive();
        assert!(result.is_err(), "Should return error when channel is closed");
    }
}
