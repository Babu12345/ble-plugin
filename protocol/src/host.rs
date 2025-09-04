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
        pub async fn borrow_send_async<T: for<'b> crate::IO<'b>>(
            &self,
            input: &T,
        ) -> crate::errors::Result<()> {
            let mut buffer = [0; N];
            input.to_bytes_in_slice(&mut buffer)?;
            self.send_bytes_async(buffer).await
        }

        /// Try sending data
        pub fn borrow_try_send<T: for<'b> crate::IO<'b>>(
            &self,
            input: T,
        ) -> crate::errors::Result<()> {
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
    #[cfg(feature = "std")]
    use std::sync::mpsc;
    use std::{string::String, vec::Vec};

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
    fn test_host_bidirectional_communication() {
        use super::*;
        use crate::plugin::PluginReceivedData;

        // Create two channels: host-to-plugin and plugin-to-host
        let (host_to_plugin_tx, host_to_plugin_rx) =
            mpsc::sync_channel::<[u8; DEFAULT_PACKET_SIZE]>(10);
        let (plugin_to_host_tx, plugin_to_host_rx) =
            mpsc::sync_channel::<[u8; DEFAULT_PACKET_SIZE]>(10);

        // Create host sender and receiver
        let host_sender = HostSender::new(host_to_plugin_tx);
        let host_receiver = HostReceiver::new(plugin_to_host_rx);

        // Create test command (host sends commands)
        let host_cmd = HostCommandConfigurePeripheral {
            name: String::from_str("TestDevice").unwrap(),
            addr: Vec::from(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]),
        };

        // Create test plugin data (plugin sends data to host)
        let plugin_data = PluginData {
            src_addr: Vec::from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
            src_addr_type: BluetoothAddressType::Public as _,
            send_type: PluginDataSendType::NotifyType as _,
            characteristic_uuid: 0x2A29,
            service_uuid: 0x180A,
            data: Vec::from(b"Hello from plugin"),
        };

        // Host sends command through HostSender
        host_sender
            .send(host_cmd.clone())
            .expect("Should send successfully");

        // Plugin receives command (simulate plugin side)
        let raw_cmd = host_to_plugin_rx.recv().expect("Should receive raw data");
        let plugin_received_data = PluginReceivedData::new(raw_cmd);
        let decoded_cmd: HostCommandConfigurePeripheral = plugin_received_data
            .decode()
            .expect("Should decode successfully");
        assert_eq!(
            host_cmd, decoded_cmd,
            "Sent and received commands should match"
        );

        // Plugin sends data to host (simulate plugin side)
        let serialized_data: [u8; DEFAULT_PACKET_SIZE] = plugin_data.to_bytes().unwrap();
        plugin_to_host_tx
            .send(serialized_data)
            .expect("Should send plugin data");

        // Host receives plugin data through HostReceiver
        let received_data = host_receiver
            .receive()
            .expect("Should receive successfully");
        let decoded_data: PluginData = received_data.decode().expect("Should decode successfully");
        assert_eq!(
            plugin_data, decoded_data,
            "Sent and received data should match"
        );
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
        assert!(
            result.is_err(),
            "Should return error when channel is closed"
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_async_host_sender_receiver_with_critical_section_mutex() {
        use super::async_host::*;
        use crate::plugin::PluginReceivedData;
        use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

        crate::test_utils::init_critical_section();

        // Create two channels: host-to-plugin and plugin-to-host
        static HOST_TO_PLUGIN: Channel<CriticalSectionRawMutex, [u8; DEFAULT_PACKET_SIZE], 10> =
            Channel::new();
        static PLUGIN_TO_HOST: Channel<CriticalSectionRawMutex, [u8; DEFAULT_PACKET_SIZE], 10> =
            Channel::new();

        let host_cmd_sender = HOST_TO_PLUGIN.sender();
        let plugin_cmd_receiver = HOST_TO_PLUGIN.receiver();

        let plugin_data_sender = PLUGIN_TO_HOST.sender();
        let host_data_receiver = PLUGIN_TO_HOST.receiver();

        // Create async host sender and receiver
        let async_host_sender = AsyncHostSender::new(host_cmd_sender);
        let async_host_receiver = AsyncHostReceiver::new(host_data_receiver);

        // Create test command and plugin response
        let host_cmd = HostCommandConfigurePeripheral {
            name: String::from("AsyncTestDevice"),
            addr: Vec::from(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]),
        };

        let plugin_response = PluginData {
            src_addr: Vec::from(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
            src_addr_type: BluetoothAddressType::Random as _,
            send_type: PluginDataSendType::NotifyType as _,
            characteristic_uuid: 0x2A19,
            service_uuid: 0x180F,
            data: Vec::from(b"Host async test"),
        };

        // Use tokio to run the async test
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Host sends command through AsyncHostSender
            async_host_sender
                .send_async(host_cmd.clone())
                .await
                .expect("Should send successfully");

            // Plugin receives command (simulate plugin side)
            let raw_cmd = plugin_cmd_receiver.receive().await;
            let plugin_received_data = PluginReceivedData::new(raw_cmd);
            let decoded_cmd: HostCommandConfigurePeripheral = plugin_received_data
                .decode()
                .expect("Should decode successfully");
            assert_eq!(host_cmd, decoded_cmd);

            // Plugin sends response (simulate plugin side)
            let serialized_response: [u8; DEFAULT_PACKET_SIZE] =
                plugin_response.to_bytes().unwrap();
            plugin_data_sender.send(serialized_response).await;

            // Host receives response through AsyncHostReceiver
            let received_data = async_host_receiver
                .receive()
                .await
                .expect("Should receive successfully");
            let decoded_response: PluginData =
                received_data.decode().expect("Should decode successfully");
            assert_eq!(plugin_response, decoded_response);
        });
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_async_host_receiver_with_plugin_data_embassy() {
        use super::async_host::*;
        use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

        crate::test_utils::init_critical_section();

        // Create embassy channel for plugin-to-host communication
        static CHANNEL: Channel<CriticalSectionRawMutex, [u8; DEFAULT_PACKET_SIZE], 10> =
            Channel::new();
        let sender = CHANNEL.sender();
        let receiver = CHANNEL.receiver();

        // Create async host receiver
        let async_host_receiver = AsyncHostReceiver::new(receiver);

        // Create test plugin data
        let plugin_data = PluginData {
            src_addr: Vec::from(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
            src_addr_type: BluetoothAddressType::Random as _,
            send_type: PluginDataSendType::NotifyType as _,
            characteristic_uuid: 0x2A19,
            service_uuid: 0x180F,
            data: Vec::from(b"Async plugin data"),
        };

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Serialize and send data as if from plugin
            let serialized_data: [u8; DEFAULT_PACKET_SIZE] = plugin_data.to_bytes().unwrap();
            sender.send(serialized_data).await;

            // Receive and decode through async host receiver
            let received_data = async_host_receiver
                .receive()
                .await
                .expect("Should receive successfully");
            let decoded_data: PluginData =
                received_data.decode().expect("Should decode successfully");

            assert_eq!(
                plugin_data, decoded_data,
                "Sent and received data should match"
            );
        });
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_async_host_borrow_send_embassy() {
        use super::async_host::*;
        use crate::plugin::PluginReceivedData;
        use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

        crate::test_utils::init_critical_section();

        // Create embassy channel for host-to-plugin commands
        static CHANNEL: Channel<CriticalSectionRawMutex, [u8; DEFAULT_PACKET_SIZE], 10> =
            Channel::new();
        let sender = CHANNEL.sender();
        let receiver = CHANNEL.receiver();

        // Create async host sender
        let async_host_sender = AsyncHostSender::new(sender);

        // Create test command
        let cmd = HostCommandGetServiceInfo { uuid: 0x180A };

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Send command using borrow_send_async (no-std compatible)
            async_host_sender
                .borrow_send_async(&cmd)
                .await
                .expect("Should send successfully");

            // Verify command was sent correctly (simulate plugin receiving)
            let raw_data = receiver.receive().await;
            let plugin_received_data = PluginReceivedData::new(raw_data);
            let decoded_cmd: HostCommandGetServiceInfo = plugin_received_data
                .decode()
                .expect("Should decode successfully");

            assert_eq!(cmd, decoded_cmd, "Sent and received commands should match");
        });
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_async_host_try_send_methods() {
        use super::async_host::*;
        use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};

        // Test with NoopRawMutex for try_send methods (doesn't require critical-section)
        let channel: Channel<NoopRawMutex, [u8; DEFAULT_PACKET_SIZE], 10> = Channel::new();
        let sender = channel.sender();
        let receiver = channel.receiver();

        let async_host_sender = AsyncHostSender::new(sender);

        // Create test command
        let cmd = HostCommandStartAdvertisement {
            allow_multi_connect: true,
        };

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Test try_send method
            async_host_sender
                .try_send(cmd.clone())
                .expect("Should try_send successfully");

            // Test borrow_try_send method
            let cmd2 = HostCommandStopAdvertisement {};
            async_host_sender
                .borrow_try_send(cmd2.clone())
                .expect("Should borrow_try_send successfully");

            // Verify both messages were sent (simulate plugin receiving)
            let raw_data1 = receiver.receive().await;
            let plugin_received_data1 = crate::plugin::PluginReceivedData::new(raw_data1);
            let decoded1: HostCommandStartAdvertisement =
                plugin_received_data1.decode().expect("Should decode first");
            assert_eq!(cmd, decoded1);

            let raw_data2 = receiver.receive().await;
            let plugin_received_data2 = crate::plugin::PluginReceivedData::new(raw_data2);
            let decoded2: HostCommandStopAdvertisement = plugin_received_data2
                .decode()
                .expect("Should decode second");
            assert_eq!(cmd2, decoded2);
        });
    }
}
