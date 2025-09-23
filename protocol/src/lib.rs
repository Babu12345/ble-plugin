//! # Protocol - BLE Plugin Communication Protocol
//!
//! A comprehensive communication protocol library for BLE-USB bridge systems, defining standardized
//! message formats, serialization, and type-safe command/response structures for plugin devices.
//!
//! ## Overview
//!
//! This library defines the complete communication protocol between host devices (PCs, mobile devices, embedded devices)
//! and BLE plugin devices (ESP32-based bridge devices). It provides type-safe message definitions,
//! efficient serialization, and protocol validation to ensure reliable communication across the
//! USB-BLE bridge.
//!
//! **Note**: When this library references "Host", it refers to the device accessing the capabilities
//! of the plugin (typically a PC or mobile device), not the USB host protocol implementation. The plugin
//! device can implement either USB host or device protocols as needed.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐     USB Commands     ┌─────────────────┐     BLE Operations      ┌─────────────┐
//! │   Host Device   │ ──────────────────►  │  Plugin Device  │ ──────────────────────► │ BLE Clients │
//! │  (PC/Mobile/    │ ◄──────────────────  │  (ESP32 + BLE)  │ ◄────────────────────── │             │
//! │   Embedded)     │     USB Responses    └─────────────────┘     BLE Callbacks       └─────────────┘
//! └─────────────────┘
//! ```
//!
//! ## Protocol Features
//!
//! - **Type-Safe Messages**: Rust type system ensures protocol correctness
//! - **Flexible Serialization**: Protocol Buffers (prost or quick-protobuf) with optional bincode support
//! - **Message Validation**: Magic number and header integrity checking  
//! - **Version Compatibility**: Structured message IDs for protocol evolution
//! - **Cross-Platform**: Supports both embedded (no_std) and standard environments
//! - **Extensible Design**: Easy addition of new command and response types
//!
//! ## Serialization Configuration
//!
//! This crate requires **exactly one** primary protobuf implementation to be enabled via feature flags:
//!
//! - `protocol_buffer`: Protocol Buffers using prost for maximum compatibility
//! - `quick_protocol_buffer`: Protocol Buffers using quick-protobuf for high performance and embedded systems
//! - `bincode_serialization`: Optional bincode support (can be combined with either protobuf implementation)
//!
//! **Embedded Note**: `quick_protocol_buffer` is recommended for embedded systems, especially
//! those without atomic CAS (Compare-And-Swap) operations, as prost requires atomic support.
//!
//! The mutual exclusivity is enforced at compile-time. The crate will fail to build if:
//! - Both `protocol_buffer` and `quick_protocol_buffer` are enabled simultaneously
//! - Neither `protocol_buffer` nor `quick_protocol_buffer` is enabled
//!
//! This ensures consistent serialization behavior throughout your application
//!
//! **Memory Note**: Protocol types use `alloc::Vec` and `alloc::String` for Protocol Buffer
//! compatibility. Internal message headers use `heapless::Vec` for predictable allocation.
//!
//! ## Message Protocol Format
//!
//! All messages use a standardized 5-byte header followed by serialized payload:
//!
//! ```text
//! ┌─────────────┬─────────────┬─────────────┬─────────────────┐
//! │   Magic     │   Type ID   │   Length    │     Payload     │
//! │  (1 byte)   │  (2 bytes)  │  (2 bytes)  │  (limited size) │
//! └─────────────┴─────────────┴─────────────┴─────────────────┘
//! ```
//!
//! - **Magic Number**: 0xDE for message integrity validation
//! - **Type ID**: Unique identifier for each message type (enables O(1) dispatch)
//! - **Length**: Payload size in bytes (little-endian)
//! - **Payload**: Protocol Buffer serialized message data (optionally with bincode support)
//!
//! **Size Constraints**: The total message size (header + payload) cannot exceed
//! [`DEFAULT_PACKET_SIZE`]. With a [`MESSAGE_HEADER_SIZE`] header, the maximum payload
//! size is [`DEFAULT_PACKET_SIZE`] - [`MESSAGE_HEADER_SIZE`] bytes.
//!
//! ## Message Categories
//!
//! ### Host Commands
//! Commands sent from host devices to configure and control the BLE plugin:
//!
//! - **Peripheral Management**: Configure device name, address, advertising
//! - **Service Operations**: Create and manage BLE services
//! - **Characteristic Control**: Create characteristics with properties
//! - **Data Operations**: Read/write/notify characteristic values
//! - **Query Commands**: Get service and characteristic information
//!
//! ### Plugin Responses
//! Responses and data sent from plugin devices back to hosts:
//!
//! - **Configuration Responses**: Success/error status for commands
//! - **Data Forwarding**: BLE client data forwarded to host
//! - **Information Responses**: Service and characteristic details
//! - **Error Notifications**: Detailed error information
//!
//! ## Core Modules
//!
//! - [`io`]: Core serialization traits and message header handling
//! - [`host`]: Host-specific communication utilities
//! - [`plugin`]: Plugin-specific communication channels
//! - [`errors`]: Comprehensive error handling
//!
//! ## Usage Examples
//!
//! ### Basic Message Creation
//!
//! ```rust
//! use protocol::io_types::HostCommandConfigurePeripheral;
//!
//! // Create a peripheral configuration command
//! let command = HostCommandConfigurePeripheral {
//!     name: String::try_from("MyDevice").unwrap(),
//!     addr: Vec::from(&[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]), // 6-byte BLE address
//! };
//! ```
//!
//! ### Message Serialization
//!
//! ```rust,no_run
//! use protocol::{IO, DEFAULT_PACKET_SIZE};
//! # use protocol::io_types::HostCommandConfigurePeripheral;
//! # let command: HostCommandConfigurePeripheral = panic!("Documentation example");
//!
//! // Serialize to fixed-size buffer with header
//! let serialized: [u8; DEFAULT_PACKET_SIZE] = command.to_bytes()?;
//!
//! // Or serialize to provided buffer
//! let mut buffer = [0u8; DEFAULT_PACKET_SIZE];
//! command.to_bytes_in_slice(&mut buffer)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Message Deserialization
//!
//! ```rust,no_run
//! use protocol::{IO, io_types::HostCommandConfigurePeripheral};
//!
//! // Deserialize from received bytes (includes header validation)
//! let received_data: &[u8] = &[/* USB data */];
//! let command = HostCommandConfigurePeripheral::from_bytes(received_data)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Serialization Configuration
//!
//! The protocol uses **Protocol Buffers** by default for cross-platform compatibility.
//! For high-performance Rust-to-Rust communication, bincode is available via feature flag:
//!
//! ```toml
//! # Default: Protocol Buffers (cross-platform)
//! [dependencies]
//! protocol = { version = "..." }
//!
//! # Alternative: Bincode for Rust-to-Rust performance
//! [dependencies]
//! protocol = { version = "...", features = ["bincode_serialization"] }
//! ```
//!
//! **When to use each:**
//! - **Protobuf** (default): Cross-language compatibility, schema evolution, future-proof
//! - **Bincode** (feature flag): High-performance Rust-only communication, embedded constraints
//!
//! ## Protocol Constants
//!
//! - [`MAX_NAME_SIZE`]: Maximum length for device names (30 characters)
//! - [`DEFAULT_PACKET_SIZE`]: Standard USB packet size (256 bytes)
//! - [`MESSAGE_HEADER_SIZE`]: Protocol header size (5 bytes)
//! - [`MESSAGE_MAGIC`]: Magic number for validation (0xDE)
//!
//! ## Feature Flags
//!
//! - `std`: Standard library support (enabled by default)
//! - `protocol_buffer`: Protocol Buffers serialization support using prost (enabled by default)
//! - `quick_protocol_buffer`: Fast Protocol Buffers serialization support using quick-protobuf
//! - `bincode_serialization`: Optional bincode serialization support
//!
//! ## Compatibility
//!
//! - **Rust Version**: 1.70+
//! - **Embedded**: Full no_std support
//! - **Platforms**: Cross-platform (desktop, mobile, embedded)
//! - **Endianness**: Little-endian byte order for consistency

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
exactly_one_feature!("protocol_buffer", "quick_protocol_buffer");

pub mod errors;
pub mod host;
pub mod io;
pub mod plugin;
/// Types to interface with the plugin io
pub mod protocol;
pub use io::*;
use lib_utils::exactly_one_feature;
pub mod devices;
pub mod utils;
#[cfg(test)]
mod validation;

/// Maximum size for BLE peripheral device names
///
/// This constant defines the maximum length for device names used in BLE advertising
/// and peripheral configuration. The limit ensures compatibility with BLE advertising
/// packet size constraints and embedded system memory limitations.

pub const MAX_NAME_SIZE: usize = 30;

/// Default USB packet size for communication
///
/// This represents the standard transfer size for USB communication between host and plugin
/// devices. The value is optimized for USB High-Speed (HS) transfers while maintaining
/// compatibility with Full-Speed (FS) devices.
///
/// - **Full-Speed USB**: Supports 8, 16, 32, or 64 bytes maximum
/// - **High-Speed USB**: Supports up to 512 bytes maximum  
/// - **Chosen Value**: 64 bytes for optimal performance across both modes
///
/// # Usage
///
/// ```rust
/// use protocol::DEFAULT_PACKET_SIZE;
///
/// let buffer: [u8; DEFAULT_PACKET_SIZE] = [0; DEFAULT_PACKET_SIZE];
/// ```
pub const DEFAULT_PACKET_SIZE: usize = 64;

#[cfg(test)]
mod tests {
    extern crate std;
    use crate::protocol::*;
    use crate::{
        io::{
            DATA_BYTES_LENGTH_IN_BYTES, MESSAGE_HEADER_SIZE, MESSAGE_MAGIC, MESSAGE_MAGIC_BYTES,
            MESSAGE_TYPE_ID_BYTES,
        },
        MessageType, DEFAULT_PACKET_SIZE, IO,
    };

    #[test]
    fn test_max_transfer_size() {
        assert!(
            DEFAULT_PACKET_SIZE <= 512,
            "The max transfer size is 512 for high speed usb"
        )
    }

    #[test]
    fn test_message_header_constants() {
        // Test that header constants are correct
        assert_eq!(MESSAGE_MAGIC, 0xDE, "Magic number should be 0xDE");
        assert_eq!(MESSAGE_MAGIC_BYTES, 1, "Magic number should be 1 byte");
        assert_eq!(
            MESSAGE_TYPE_ID_BYTES, 2,
            "Message type ID should be 2 bytes"
        );
        assert_eq!(
            DATA_BYTES_LENGTH_IN_BYTES, 2,
            "Data length should be 2 bytes"
        );
        assert_eq!(
            MESSAGE_HEADER_SIZE, 5,
            "Total header size should be 5 bytes"
        );
        assert_eq!(
            MESSAGE_HEADER_SIZE,
            MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES + DATA_BYTES_LENGTH_IN_BYTES,
            "Header size should equal sum of component sizes"
        );
    }

    #[test]
    fn test_message_type_id_values() {
        // Test that message type IDs have expected values
        assert_eq!(
            MessageTypeId::TypeHostCommandConfigurePeripheral as u8,
            0x01
        );
        assert_eq!(MessageTypeId::TypeHostCommandConfigureService as u8, 0x02);
        assert_eq!(
            MessageTypeId::TypeHostCommandConfigureCharacteristic as u8,
            0x03
        );
        assert_eq!(
            MessageTypeId::TypeHostCommandConfigureCharacteristicRead as u8,
            0x04
        );
        assert_eq!(MessageTypeId::TypeHostCommandGetServiceInfo as u8, 0x05);
        assert_eq!(
            MessageTypeId::TypeHostCommandGetCharacteristicInfo as u8,
            0x06
        );
        assert_eq!(MessageTypeId::TypeHostCommandStartAdvertisement as u8, 0x07);
        assert_eq!(
            MessageTypeId::TypeHostCommandNotifyCharacteristicValue as u8,
            0x08
        );
        assert_eq!(MessageTypeId::TypePluginData as u8, 0x80);
        assert_eq!(MessageTypeId::TypePluginConfigurationError as u8, 0x81);
        assert_eq!(MessageTypeId::TypePluginServiceInfoResponse as u8, 0x82);
        assert_eq!(
            MessageTypeId::TypePluginCharacteristicInfoResponse as u8,
            0x83
        );
    }

    #[test]
    fn test_message_type_implementations() {
        // Test that each message type has correct MessageType implementation
        assert_eq!(
            HostCommandConfigurePeripheral::MESSAGE_TYPE_ID as u8,
            MessageTypeId::TypeHostCommandConfigurePeripheral as u8
        );
        assert_eq!(
            HostCommandConfigureService::MESSAGE_TYPE_ID as u8,
            MessageTypeId::TypeHostCommandConfigureService as u8
        );
        assert_eq!(
            HostCommandConfigureCharacteristic::MESSAGE_TYPE_ID as u8,
            MessageTypeId::TypeHostCommandConfigureCharacteristic as u8
        );
        assert_eq!(
            HostCommandConfigureCharacteristicRead::MESSAGE_TYPE_ID as u8,
            MessageTypeId::TypeHostCommandConfigureCharacteristicRead as u8
        );
        assert_eq!(
            HostCommandGetServiceInfo::MESSAGE_TYPE_ID as u8,
            MessageTypeId::TypeHostCommandGetServiceInfo as u8
        );
        assert_eq!(
            HostCommandGetCharacteristicInfo::MESSAGE_TYPE_ID as u8,
            MessageTypeId::TypeHostCommandGetCharacteristicInfo as u8
        );
        assert_eq!(
            HostCommandStartAdvertisement::MESSAGE_TYPE_ID as u8,
            MessageTypeId::TypeHostCommandStartAdvertisement as u8
        );
        assert_eq!(
            HostCommandNotifyCharacteristicValue::MESSAGE_TYPE_ID as u8,
            MessageTypeId::TypeHostCommandNotifyCharacteristicValue as u8
        );
        assert_eq!(
            PluginConfigurationError::MESSAGE_TYPE_ID as u8,
            MessageTypeId::TypePluginConfigurationError as u8
        );
        assert_eq!(
            PluginServiceInfoResponse::MESSAGE_TYPE_ID as u8,
            MessageTypeId::TypePluginServiceInfoResponse as u8
        );
        assert_eq!(
            PluginCharacteristicInfoResponse::MESSAGE_TYPE_ID as u8,
            MessageTypeId::TypePluginCharacteristicInfoResponse as u8
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_host_command_serialization_with_header() {
        // Test HostCommandConfigurePeripheral serialization
        let cmd = HostCommandConfigurePeripheral {
            name: String::try_from("TestDevice").expect("Should create string"),
            addr: std::vec::Vec::from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
        };

        let serialized: [u8; DEFAULT_PACKET_SIZE] =
            cmd.to_bytes().expect("Should serialize successfully");

        // Verify magic number in first byte
        let magic = serialized[0];
        assert_eq!(
            magic, MESSAGE_MAGIC,
            "Magic number should be present in header"
        );

        // Verify message type ID in bytes 1-2
        let type_id = u16::from_le_bytes([
            serialized[MESSAGE_MAGIC_BYTES],
            serialized[MESSAGE_MAGIC_BYTES + 1],
        ]) as u8;
        assert_eq!(
            type_id,
            MessageTypeId::TypeHostCommandConfigurePeripheral as u8,
            "Message type ID should be correct"
        );

        // Verify data length in bytes 3-4
        let length_bytes = [
            serialized[MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES],
            serialized[MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES + 1],
        ];
        let data_length = u16::from_le_bytes(length_bytes);
        assert!(data_length > 0, "Data length should be greater than 0");
        assert!(
            (data_length as usize) < DEFAULT_PACKET_SIZE,
            "Data length should be less than packet size"
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_plugin_response_serialization_with_header() {
        // Test PluginServiceInfoResponse serialization
        let mut char_uuids: std::vec::Vec<u32> = Vec::new();
        char_uuids.push(0);
        char_uuids.push(u32::MAX);

        let response = PluginServiceInfoResponse {
            service_uuid: 0,
            characteristic_uuids: char_uuids,
            exists: true,
        };

        let serialized: [u8; DEFAULT_PACKET_SIZE] =
            response.to_bytes().expect("Should serialize successfully");

        // Verify magic number
        let magic = serialized[0];
        assert_eq!(magic, MESSAGE_MAGIC, "Magic number should be present");

        // Verify message type ID
        let type_id = u16::from_le_bytes([
            serialized[MESSAGE_MAGIC_BYTES],
            serialized[MESSAGE_MAGIC_BYTES + 1],
        ]) as u8;
        assert_eq!(
            type_id,
            MessageTypeId::TypePluginServiceInfoResponse as u8,
            "Message type ID should be correct for plugin response"
        );
    }

    #[test]
    fn test_message_header_roundtrip() {
        // Create a command
        let original_cmd = HostCommandGetServiceInfo { uuid: 0 };

        // Serialize with header
        let mut buffer = [0u8; DEFAULT_PACKET_SIZE];
        original_cmd
            .to_bytes_in_slice(&mut buffer)
            .expect("Should serialize");

        // Verify header structure before deserialization
        let magic = buffer[0];
        assert_eq!(magic, MESSAGE_MAGIC);

        let type_id =
            u16::from_le_bytes([buffer[MESSAGE_MAGIC_BYTES], buffer[MESSAGE_MAGIC_BYTES + 1]])
                as u8;
        assert_eq!(type_id, MessageTypeId::TypeHostCommandGetServiceInfo as u8);

        // Deserialize and verify round-trip
        let deserialized_cmd = HostCommandGetServiceInfo::from_bytes(&buffer)
            .expect("Should deserialize successfully");

        assert_eq!(
            original_cmd.uuid, deserialized_cmd.uuid,
            "Round-trip should preserve data"
        );
    }

    #[test]
    fn test_header_size_constraints() {
        // Ensure header size doesn't exceed reasonable limits
        assert!(
            MESSAGE_HEADER_SIZE <= 16,
            "Header should be reasonably small"
        );
        assert!(
            MESSAGE_HEADER_SIZE >= 4,
            "Header should contain minimum required fields"
        );

        // Ensure we have room for payload
        assert!(
            MESSAGE_HEADER_SIZE < DEFAULT_PACKET_SIZE / 2,
            "Header should leave plenty of room for payload"
        );
    }

    #[test]
    fn test_message_header_size_calculation() {
        // Verify that MESSAGE_HEADER_SIZE calculation is correct
        let expected_size = 1 + 2 + 2; // magic + type_id + length
        assert_eq!(MESSAGE_HEADER_SIZE, expected_size);
        assert_eq!(MESSAGE_HEADER_SIZE, 5);
    }

    #[test]
    fn test_invalid_message_length() {
        // Create a buffer with an invalid (too large) length field
        let mut invalid_buffer = [0u8; DEFAULT_PACKET_SIZE];

        // Set correct magic and type ID
        invalid_buffer[0] = MESSAGE_MAGIC;
        let type_id_bytes = (MessageTypeId::TypeHostCommandGetServiceInfo as u16).to_le_bytes();
        invalid_buffer[1] = type_id_bytes[0];
        invalid_buffer[2] = type_id_bytes[1];

        // Set impossibly large length (bigger than packet size)
        let invalid_length = DEFAULT_PACKET_SIZE + 100;
        invalid_buffer[3] = (invalid_length & 0xFF) as u8;
        invalid_buffer[4] = ((invalid_length >> 8) & 0xFF) as u8;

        // Should fail to deserialize due to invalid length
        let result = HostCommandGetServiceInfo::from_bytes(&invalid_buffer);
        assert!(
            result.is_err(),
            "Should reject message with invalid length field"
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_configure_profile_serialization() {
        let cmd = HostCommandConfigureProfile {
            profile: BleProfile::Custom as _,
            save_on_disconnect: false,
        };

        // Test serialization
        let serialized: [u8; DEFAULT_PACKET_SIZE] = cmd.to_bytes().expect("Should serialize");

        // Verify magic number
        let magic = serialized[0];
        assert_eq!(magic, MESSAGE_MAGIC);

        // Verify message type ID
        let type_id = u16::from_le_bytes([
            serialized[MESSAGE_MAGIC_BYTES],
            serialized[MESSAGE_MAGIC_BYTES + 1],
        ]) as u8;
        assert_eq!(
            type_id,
            MessageTypeId::TypeHostCommandConfigureProfile as u8
        );

        // Test round-trip
        let deserialized =
            HostCommandConfigureProfile::from_bytes(&serialized).expect("Should deserialize");
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_ble_profile_enum_values() {
        // Verify enum values
        assert_eq!(BleProfile::Custom as u8, 1);

        // Test that enum can be created and compared
        let profile1 = BleProfile::Custom;
        let profile2 = BleProfile::Custom;
        assert_eq!(profile1, profile2);
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
#[allow(missing_docs)]
pub mod test_utils {
    struct CriticalSection;

    unsafe impl critical_section::Impl for CriticalSection {
        unsafe fn acquire() {}
        unsafe fn release(_token: ()) {}
    }

    critical_section::set_impl!(CriticalSection);
}
