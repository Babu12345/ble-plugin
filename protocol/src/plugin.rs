//! Defines the traits for the plugin
//! The peripheral is the device that will connect to the host and receives and transmits data to the primary
//! Typically this can be bluetooth but can really be any other propriary or open source interface if required.
//! The communication protocol between the plugin and the host is typically USB based.

/// Embassy channel send and receive
use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    channel::{Receiver, Sender},
};

use crate::errors::{self, Result};

use crate::IO;
pub use async_plugin::*;
pub use common::*;

/// Common types and traits
mod common {
    use crate::{errors::Result, HostIO};

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

        /// Get the size of the data
        pub fn size(&self) -> usize {
            self.0.len()
        }

        /// Get access to the raw bytes
        pub fn raw_bytes(&self) -> &[u8] {
            &self.0
        }
    }
}

/// Async implementation
pub mod async_plugin {
    use crate::host::HostReceivedData;

    use super::*;

    /// Async sender
    pub struct AsyncPluginSender<'ch, R: RawMutex, const N: usize, const CH_SIZE: usize>(
        Sender<'ch, R, [u8; N], CH_SIZE>,
    );

    /// Async receiver
    pub struct AsyncPluginReceiver<'ch, R: RawMutex, const N: usize, const CH_SIZE: usize>(
        Receiver<'ch, R, [u8; N], CH_SIZE>,
    );

    impl<'a, const N: usize, const CH_SIZE: usize, R: RawMutex> AsyncPluginSender<'a, R, N, CH_SIZE> {
        /// Create a new instance
        pub fn new(sender: Sender<'a, R, [u8; N], CH_SIZE>) -> Self {
            Self(sender)
        }

        /// Send the data
        #[cfg(feature = "std")]
        pub async fn send_async<T: IO<'a>>(&self, input: T) -> Result<()> {
            self.send_bytes_async(input.to_bytes()?).await
        }

        #[cfg(feature = "std")]
        /// Try sending data
        pub fn try_send<T: IO<'a>>(&self, input: T) -> Result<()> {
            self.try_send_bytes(input.to_bytes()?)
        }

        /// Send the data
        pub async fn borrow_send_async<T: for<'b> IO<'b>>(&self, input: &T) -> Result<()> {
            let mut buffer = [0; N];
            input.to_bytes_in_slice(&mut buffer)?;
            self.send_bytes_async(buffer).await
        }

        /// Try sending data
        pub fn borrow_try_send<T: for<'b> IO<'b>>(&self, input: T) -> Result<()> {
            let mut buffer = [0; N];
            input.to_bytes_in_slice(&mut buffer)?;
            self.try_send_bytes(buffer)
        }

        /// Send bytes directly
        async fn send_bytes_async(&self, buffer: [u8; N]) -> Result<()> {
            self.0.send(buffer).await;
            Ok(())
        }

        /// Try sending bytes directly
        fn try_send_bytes(&self, buffer: [u8; N]) -> Result<()> {
            self.0
                .try_send(buffer)
                .map_err(|_| errors::Error::SendError)
        }
    }

    impl<'a, const N: usize, const CH_SIZE: usize, R: RawMutex> AsyncPluginReceiver<'a, R, N, CH_SIZE> {
        /// Create a new instance
        pub fn new(receiver: Receiver<'a, R, [u8; N], CH_SIZE>) -> Self {
            Self(receiver)
        }

        /// Receive the data
        pub async fn receive(&self) -> Result<HostReceivedData<N>> {
            let input = self.0.receive().await;
            Ok(HostReceivedData::new(input))
        }
    }
}

/// Standard non-async version of the plugin implementation
#[cfg(feature = "std")]
pub mod plugin {
    use super::*;
    use crate::{
        errors::{self, Result},
        PluginIO,
    };
    use std::sync::mpsc::{Receiver, SyncSender};
    /// Sender
    pub struct PluginSender<const N: usize>(SyncSender<[u8; N]>);

    /// Receiver
    pub struct PluginReceiver<const N: usize>(Receiver<[u8; N]>);

    impl<'a, const N: usize> PluginSender<N> {
        /// Create a new instance
        pub fn new(sender: SyncSender<[u8; N]>) -> Self {
            Self(sender)
        }

        /// Send the data
        pub fn send<T: PluginIO<'a>>(&self, input: T) -> Result<()> {
            self.0
                .send(input.to_bytes()?)
                .map_err(|_| crate::errors::Error::SendError)
        }
    }

    impl<'a, const N: usize> PluginReceiver<N> {
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
    extern crate std;
    use crate::host::HostReceivedData;
    use crate::protocol::*;
    use crate::DEFAULT_PACKET_SIZE;
    use crate::IO;
    use std::vec::Vec;

    #[test]
    #[cfg(feature = "std")]
    fn test_std_encoding_and_decoding() {
        let cmd = PluginData {
            src_addr: Vec::from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
            src_addr_type: BluetoothAddressType::Public as _,
            send_type: PluginDataSendType::NotifyType as _,
            characteristic_uuid: 0x2A29,
            service_uuid: 0x180A,
            data: Vec::from(b"Cool test"),
        };

        let data: [u8; DEFAULT_PACKET_SIZE] = cmd.to_bytes().unwrap();
        let received_data = HostReceivedData::new(data);
        let decoded_cmd: PluginData = received_data.decode().unwrap();

        assert_eq!(
            cmd, decoded_cmd,
            "Testing a single command being encoded and decoded"
        );
    }

    #[test]
    fn test_no_std_encoding_and_decoding() {
        let cmd = PluginData {
            src_addr: Vec::from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
            src_addr_type: BluetoothAddressType::Public as _,
            send_type: PluginDataSendType::NotifyType as _,
            characteristic_uuid: 0x2A29,
            service_uuid: 0x180A,
            data: Vec::from(b"Another one\0"),
        };

        let mut buffer = [0u8; DEFAULT_PACKET_SIZE];
        cmd.to_bytes_in_slice(&mut buffer).unwrap();
        let received_data = HostReceivedData::new(buffer);
        let decoded_cmd: PluginData = received_data.decode().unwrap();
        assert_eq!(
            cmd, decoded_cmd,
            "Testing a single command being encoded and decoded"
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_plugin_sender_and_host_receiver_communication() {
        use std::sync::mpsc;
        use super::plugin::*;
        use crate::host::HostReceivedData;

        // Create a channel for plugin-to-host communication
        let (tx, rx) = mpsc::sync_channel::<[u8; DEFAULT_PACKET_SIZE]>(10);
        
        // Create plugin sender
        let plugin_sender = PluginSender::new(tx);

        // Create test plugin data (plugin sends to host)
        let data = PluginData {
            src_addr: Vec::from(&[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]),
            src_addr_type: BluetoothAddressType::Random as _,
            send_type: PluginDataSendType::WriteType as _,
            characteristic_uuid: 0x2A19,
            service_uuid: 0x180F,
            data: Vec::from(b"Battery Level: 85%"),
        };

        // Send data through plugin sender
        plugin_sender.send(data.clone()).expect("Should send successfully");

        // Receive raw data from channel
        let raw_data = rx.recv().expect("Should receive raw data");
        let host_received_data = HostReceivedData::new(raw_data);
        let decoded_data: PluginData = host_received_data.decode().expect("Should decode successfully");

        assert_eq!(data, decoded_data, "Sent and received data should match");
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_plugin_receiver_with_host_commands() {
        use std::sync::mpsc;
        use super::plugin::*;

        // Create channel for host-to-plugin communication
        let (tx, rx) = mpsc::sync_channel::<[u8; DEFAULT_PACKET_SIZE]>(10);
        let plugin_receiver = PluginReceiver::new(rx);

        // Create test host command (host sends commands to plugin)
        let cmd = HostCommandConfigurePeripheral {
            name: String::from("TestDevice"),
            addr: Vec::from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
        };

        // Serialize and send command as if from host
        let serialized_cmd: [u8; DEFAULT_PACKET_SIZE] = cmd.to_bytes().unwrap();
        tx.send(serialized_cmd).expect("Should send host command");

        // Receive and verify through plugin receiver
        let received_data = plugin_receiver.receive().expect("Should receive command");
        let decoded_cmd: HostCommandConfigurePeripheral = received_data.decode().expect("Should decode command");
        assert_eq!(cmd, decoded_cmd);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_plugin_receiver_empty_channel() {
        use std::sync::mpsc;
        use super::plugin::*;

        let (tx, rx) = mpsc::sync_channel::<[u8; DEFAULT_PACKET_SIZE]>(10);
        let plugin_receiver = PluginReceiver::new(rx);

        // Drop sender to close channel
        drop(tx);

        // Should get receive error
        let result = plugin_receiver.receive();
        assert!(result.is_err(), "Should return error when channel is closed");
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_plugin_received_data_raw_access() {
        use super::*;
        
        let cmd = HostCommandGetServiceInfo { uuid: 0x1234 };
        let bytes: [u8; DEFAULT_PACKET_SIZE] = cmd.to_bytes().unwrap();
        let received_data = PluginReceivedData::new(bytes);

        // Test raw access methods
        assert_eq!(received_data.size(), DEFAULT_PACKET_SIZE);
        assert_eq!(received_data.raw_bytes().len(), DEFAULT_PACKET_SIZE);
        assert_eq!(received_data.raw_bytes(), &bytes);
    }
}
