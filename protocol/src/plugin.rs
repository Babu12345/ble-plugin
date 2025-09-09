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
    use strum::IntoEnumIterator;

    use crate::{
        errors::Result, protocol::MessageTypeId, HostIO, MESSAGE_HEADER_SIZE, MESSAGE_MAGIC,
        MESSAGE_MAGIC_BYTES,
    };

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
        pub(crate) fn raw_bytes(&self) -> &[u8] {
            &self.0
        }

        /// Extract message type ID from received data with validation
        ///
        /// This method validates the message header format and extracts the message type ID
        /// for efficient command dispatch. It performs integrity checks including magic
        /// number validation and header size verification.
        ///
        /// # Arguments
        ///
        /// * `data` - Raw USB data buffer containing message header and payload
        ///
        /// # Returns
        ///
        /// * `Ok(MessageTypeId)` - Successfully extracted message type ID
        /// * `Err(Error)` - Invalid message format or unknown type ID
        ///
        /// # Errors
        ///
        /// * `InvalidDataLengthForHeader` - Data too short
        /// * `InvalidMagicNumber` - Invalid magic number
        /// * `InvalidMessageType` - Unrecognized message type ID
        ///
        /// # Message Header Format
        ///
        /// ```text
        /// [0]:   Magic number (0xDE)
        /// [2]:   Message type ID
        /// [3-4]: Payload length (little-endian)
        /// [5+]:  Payload data
        /// ```
        pub fn extract_message_type_id(&self) -> Result<MessageTypeId> {
            let data = self.raw_bytes();
            // Check if we have enough bytes for a valid header
            if data.len() < MESSAGE_HEADER_SIZE {
                return Err(crate::errors::Error::InvalidDataLengthForHeader);
            }

            // Verify magic number
            let magic = data[0];
            if magic != MESSAGE_MAGIC {
                return Err(crate::errors::Error::InvalidMagicNumber);
            }

            // Extract message type ID
            let type_id =
                u16::from_le_bytes([data[MESSAGE_MAGIC_BYTES], data[MESSAGE_MAGIC_BYTES + 1]]);

            let message_type_id = MessageTypeId::iter()
                .find(|message_type_id| (*message_type_id as i32) == (type_id as i32));

            match message_type_id {
                Some(id) => Ok(id),
                None => Err(crate::errors::Error::InvalidMessageType),
            }
        }
    }
}

/// Async implementation
pub mod async_plugin {
    use crate::plugin::PluginReceivedData;

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
        pub async fn receive(&self) -> Result<PluginReceivedData<N>> {
            let input = self.0.receive().await;
            Ok(PluginReceivedData::new(input))
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
    fn test_plugin_bidirectional_communication() {
        use super::plugin::*;
        use crate::host::HostReceivedData;
        use std::sync::mpsc;

        // Create two channels: host-to-plugin and plugin-to-host
        let (host_to_plugin_tx, host_to_plugin_rx) =
            mpsc::sync_channel::<[u8; DEFAULT_PACKET_SIZE]>(10);
        let (plugin_to_host_tx, plugin_to_host_rx) =
            mpsc::sync_channel::<[u8; DEFAULT_PACKET_SIZE]>(10);

        // Create plugin sender and receiver
        let plugin_sender = PluginSender::new(plugin_to_host_tx);
        let plugin_receiver = PluginReceiver::new(host_to_plugin_rx);

        // Create test host command (host sends commands to plugin)
        let host_cmd = HostCommandConfigurePeripheral {
            name: String::from("TestDevice"),
            addr: Vec::from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
        };

        // Create test plugin data (plugin sends to host)
        let plugin_data = PluginData {
            src_addr: Vec::from(&[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]),
            src_addr_type: BluetoothAddressType::Random as _,
            send_type: PluginDataSendType::WriteType as _,
            characteristic_uuid: 0x2A19,
            service_uuid: 0x180F,
            data: Vec::from(b"Battery Level: 85%"),
        };

        // Host sends command to plugin
        let serialized_cmd: [u8; DEFAULT_PACKET_SIZE] = host_cmd.to_bytes().unwrap();
        host_to_plugin_tx
            .send(serialized_cmd)
            .expect("Should send host command");

        // Plugin receives command through PluginReceiver
        let received_data = plugin_receiver.receive().expect("Should receive command");
        let decoded_cmd: HostCommandConfigurePeripheral =
            received_data.decode().expect("Should decode command");
        assert_eq!(host_cmd, decoded_cmd);

        // Plugin sends data to host through PluginSender
        plugin_sender
            .send(plugin_data.clone())
            .expect("Should send successfully");

        // Host receives plugin data
        let raw_data = plugin_to_host_rx.recv().expect("Should receive raw data");
        let host_received_data = HostReceivedData::new(raw_data);
        let decoded_data: PluginData = host_received_data
            .decode()
            .expect("Should decode successfully");

        assert_eq!(
            plugin_data, decoded_data,
            "Sent and received data should match"
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_plugin_receiver_empty_channel() {
        use super::plugin::*;
        use std::sync::mpsc;

        let (tx, rx) = mpsc::sync_channel::<[u8; DEFAULT_PACKET_SIZE]>(10);
        let plugin_receiver = PluginReceiver::new(rx);

        // Drop sender to close channel
        drop(tx);

        // Should get receive error
        let result = plugin_receiver.receive();
        assert!(
            result.is_err(),
            "Should return error when channel is closed"
        );
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

    #[test]
    #[cfg(feature = "std")]
    fn test_async_plugin_sender_receiver_with_critical_section_mutex() {
        use super::async_plugin::*;
        use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

        // Test with CriticalSectionRawMutex
        static CHANNEL: Channel<CriticalSectionRawMutex, [u8; DEFAULT_PACKET_SIZE], 10> =
            Channel::new();

        let sender = CHANNEL.sender();
        let receiver = CHANNEL.receiver();

        let async_plugin_sender = AsyncPluginSender::new(sender);

        // Create test plugin data
        let plugin_data = PluginData {
            src_addr: Vec::from(&[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]),
            src_addr_type: BluetoothAddressType::Random as _,
            send_type: PluginDataSendType::WriteType as _,
            characteristic_uuid: 0x2A19,
            service_uuid: 0x180F,
            data: Vec::from(b"Embassy async plugin data"),
        };

        // Use tokio to run the async test
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Send data through async plugin sender
            async_plugin_sender
                .send_async(plugin_data.clone())
                .await
                .expect("Should send successfully");

            // Verify data was sent correctly (simulate host receiving)
            let raw_data = receiver.receive().await;
            let host_received_data = crate::host::HostReceivedData::new(raw_data);
            let decoded_data: PluginData = host_received_data
                .decode()
                .expect("Should decode successfully");

            assert_eq!(
                plugin_data, decoded_data,
                "Sent and received data should match"
            );
        });
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_async_plugin_bidirectional_embassy() {
        use super::async_plugin::*;
        use crate::host::HostReceivedData;
        use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

        // Create two channels: host-to-plugin and plugin-to-host
        static HOST_TO_PLUGIN: Channel<CriticalSectionRawMutex, [u8; DEFAULT_PACKET_SIZE], 10> =
            Channel::new();
        static PLUGIN_TO_HOST: Channel<CriticalSectionRawMutex, [u8; DEFAULT_PACKET_SIZE], 10> =
            Channel::new();

        let host_sender = HOST_TO_PLUGIN.sender();
        let plugin_receiver_chan = HOST_TO_PLUGIN.receiver();

        let plugin_sender_chan = PLUGIN_TO_HOST.sender();
        let host_receiver = PLUGIN_TO_HOST.receiver();

        // Create async plugin sender and receiver
        let async_plugin_sender = AsyncPluginSender::new(plugin_sender_chan);
        let async_plugin_receiver = AsyncPluginReceiver::new(plugin_receiver_chan);

        // Create test host command and plugin response
        let host_cmd = HostCommandConfigureService { uuid: 0x180A };

        let plugin_response = PluginData {
            src_addr: Vec::from(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]),
            src_addr_type: BluetoothAddressType::Public as _,
            send_type: PluginDataSendType::NotifyType as _,
            characteristic_uuid: 0x2A19,
            service_uuid: 0x180F,
            data: Vec::from(b"Bidirectional test"),
        };

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Send host command
            let serialized_cmd: [u8; DEFAULT_PACKET_SIZE] = host_cmd.to_bytes().unwrap();
            host_sender.send(serialized_cmd).await;

            // Plugin receives host command through AsyncPluginReceiver
            let received_cmd = async_plugin_receiver
                .receive()
                .await
                .expect("Should receive command");
            let decoded_cmd: HostCommandConfigureService =
                received_cmd.decode().expect("Should decode command");
            assert_eq!(host_cmd, decoded_cmd);

            // Plugin sends response through AsyncPluginSender
            async_plugin_sender
                .send_async(plugin_response.clone())
                .await
                .expect("Should send response");

            // Host receives plugin response
            let raw_response = host_receiver.receive().await;
            let host_received_data = HostReceivedData::new(raw_response);
            let decoded_response: PluginData =
                host_received_data.decode().expect("Should decode response");
            assert_eq!(plugin_response, decoded_response);
        });
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_async_plugin_borrow_send_embassy() {
        use super::async_plugin::*;
        use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

        // Create embassy channel
        static CHANNEL: Channel<CriticalSectionRawMutex, [u8; DEFAULT_PACKET_SIZE], 10> =
            Channel::new();
        let sender = CHANNEL.sender();
        let receiver = CHANNEL.receiver();

        // Create async plugin sender
        let async_plugin_sender = AsyncPluginSender::new(sender);

        // Create test plugin error response
        let error_response = PluginConfigurationError {
            error_type: PluginConfigurationErrorType::PeripheralNameTooLong as _,
        };

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Send response using borrow_send_async (no-std compatible)
            async_plugin_sender
                .borrow_send_async(&error_response)
                .await
                .expect("Should send successfully");

            // Verify response was sent correctly (simulate host receiving)
            let raw_data = receiver.receive().await;
            let host_received_data = crate::host::HostReceivedData::new(raw_data);
            let decoded_response: PluginConfigurationError = host_received_data
                .decode()
                .expect("Should decode successfully");

            assert_eq!(
                error_response, decoded_response,
                "Sent and received responses should match"
            );
        });
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_async_plugin_multiple_message_types_embassy() {
        use super::async_plugin::*;
        use crate::host::HostReceivedData;
        use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};

        // Create embassy channel with NoopRawMutex for multiple message test
        let channel: Channel<NoopRawMutex, [u8; DEFAULT_PACKET_SIZE], 10> = Channel::new();
        let sender = channel.sender();
        let receiver = channel.receiver();

        // Create async plugin sender
        let async_plugin_sender = AsyncPluginSender::new(sender);

        // Create different types of plugin messages
        let data_msg = PluginData {
            src_addr: Vec::from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
            src_addr_type: BluetoothAddressType::Public as _,
            send_type: PluginDataSendType::NotifyType as _,
            characteristic_uuid: 0x2A29,
            service_uuid: 0x180A,
            data: Vec::from(b"Multi-message test"),
        };

        let service_info_msg = PluginServiceInfoResponse {
            service_uuid: 0x180A,
            characteristic_uuids: Vec::from(&[0x2A29, 0x2A24]),
            exists: true,
        };

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Send messages sequentially
            async_plugin_sender
                .send_async(data_msg.clone())
                .await
                .expect("Should send data message");
            async_plugin_sender
                .send_async(service_info_msg.clone())
                .await
                .expect("Should send service info");

            // Receive and verify first message
            let raw_data1 = receiver.receive().await;
            let host_received_data1 = HostReceivedData::new(raw_data1);
            let decoded_data1: PluginData = host_received_data1
                .decode()
                .expect("Should decode data message");
            assert_eq!(data_msg, decoded_data1);

            // Receive and verify second message
            let raw_data2 = receiver.receive().await;
            let host_received_data2 = HostReceivedData::new(raw_data2);
            let decoded_data2: PluginServiceInfoResponse = host_received_data2
                .decode()
                .expect("Should decode service info");
            assert_eq!(service_info_msg, decoded_data2);
        });
    }

    #[test]
    fn test_plugin_received_data_extract_message_type_id_valid() {
        use super::*;
        use crate::{protocol::MessageTypeId, MESSAGE_MAGIC, MESSAGE_MAGIC_BYTES};

        // Create a valid message header with PluginData type
        let mut buffer = [0u8; DEFAULT_PACKET_SIZE];

        // Set magic number
        buffer[0] = MESSAGE_MAGIC;

        // Set message type ID (2 bytes)
        let type_id_bytes = (MessageTypeId::TypePluginData as u16).to_le_bytes();
        buffer[MESSAGE_MAGIC_BYTES] = type_id_bytes[0];
        buffer[MESSAGE_MAGIC_BYTES + 1] = type_id_bytes[1];

        // Set length (little-endian) - some reasonable payload size
        let payload_length = 20u16;
        buffer[3] = (payload_length & 0xFF) as u8;
        buffer[4] = ((payload_length >> 8) & 0xFF) as u8;

        let received_data = PluginReceivedData::new(buffer);
        let result = received_data.extract_message_type_id();

        assert!(
            result.is_ok(),
            "Should extract message type ID successfully"
        );
        assert_eq!(result.unwrap(), MessageTypeId::TypePluginData);
    }

    #[test]
    fn test_plugin_received_data_extract_message_type_id_invalid_magic() {
        use super::*;
        use crate::{protocol::MessageTypeId, MESSAGE_MAGIC_BYTES};

        let mut buffer = [0u8; DEFAULT_PACKET_SIZE];

        // Set invalid magic number
        buffer[0] = 0xFF;
        buffer[1] = 0xFF;

        // Set valid message type ID (2 bytes)
        let type_id_bytes = (MessageTypeId::TypePluginData as u16).to_le_bytes();
        buffer[MESSAGE_MAGIC_BYTES] = type_id_bytes[0];
        buffer[MESSAGE_MAGIC_BYTES + 1] = type_id_bytes[1];

        let received_data = PluginReceivedData::new(buffer);
        let result = received_data.extract_message_type_id();

        assert!(result.is_err(), "Should fail with invalid magic number");
        assert!(matches!(
            result.unwrap_err(),
            crate::errors::Error::InvalidMagicNumber
        ));
    }

    #[test]
    fn test_plugin_received_data_extract_message_type_id_invalid_length() {
        use super::*;

        // Create buffer smaller than header size
        let small_buffer = [0u8; 3]; // Less than MESSAGE_HEADER_SIZE (5)
        let received_data = PluginReceivedData::new(small_buffer);
        let result = received_data.extract_message_type_id();

        assert!(result.is_err(), "Should fail with insufficient data length");
        assert!(matches!(
            result.unwrap_err(),
            crate::errors::Error::InvalidDataLengthForHeader
        ));
    }

    #[test]
    fn test_plugin_received_data_extract_message_type_id_invalid_type() {
        use super::*;
        use crate::{MESSAGE_MAGIC, MESSAGE_MAGIC_BYTES};

        let mut buffer = [0u8; DEFAULT_PACKET_SIZE];

        // Set valid magic number
        buffer[0] = MESSAGE_MAGIC;

        // Set invalid message type ID (0xFF is not defined in the enum)
        buffer[MESSAGE_MAGIC_BYTES] = 0xFF;
        buffer[MESSAGE_MAGIC_BYTES + 1] = 0xFF;

        let received_data = PluginReceivedData::new(buffer);
        let result = received_data.extract_message_type_id();

        assert!(result.is_err(), "Should fail with invalid message type ID");
        assert!(matches!(
            result.unwrap_err(),
            crate::errors::Error::InvalidMessageType
        ));
    }

    #[test]
    fn test_plugin_received_data_extract_message_type_id_all_valid_types() {
        use super::*;
        use crate::{protocol::MessageTypeId, MESSAGE_MAGIC, MESSAGE_MAGIC_BYTES};

        let test_cases = [
            MessageTypeId::TypeHostCommandConfigurePeripheral,
            MessageTypeId::TypeHostCommandConfigureService,
            MessageTypeId::TypeHostCommandConfigureCharacteristic,
            MessageTypeId::TypePluginData,
            MessageTypeId::TypePluginConfigurationError,
            MessageTypeId::TypePluginServiceInfoResponse,
        ];

        for &expected_type_id in &test_cases {
            let mut buffer = [0u8; DEFAULT_PACKET_SIZE];

            // Set valid magic number
            buffer[0] = MESSAGE_MAGIC;

            // Set message type ID (2 bytes)
            let type_id_bytes = (expected_type_id as u16).to_le_bytes();
            buffer[MESSAGE_MAGIC_BYTES] = type_id_bytes[0];
            buffer[MESSAGE_MAGIC_BYTES + 1] = type_id_bytes[1];

            // Set valid length
            buffer[3] = 10; // 10-byte payload
            buffer[4] = 0;

            let received_data = PluginReceivedData::new(buffer);
            let result = received_data.extract_message_type_id();

            assert!(
                result.is_ok(),
                "Should extract {} successfully",
                expected_type_id as u8
            );
            assert_eq!(result.unwrap(), expected_type_id);
        }
    }
}
