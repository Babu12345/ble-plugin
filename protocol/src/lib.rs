//! Defines the host and plug-in protocols that they must adhere to in order to
//! transfer data and commands between each other. Note when this library references Host it refers to
//! the the device that is accessing the capabilitiese of the plugin. It does not refer to the USB host protocol.
//! This is because it's technically correct for the plugin to implement the USB host protocol for "hosts" that implement
//! the USB device protocol only

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

pub mod errors;
pub mod host;
mod io;
pub mod io_types;
pub mod plugin;
pub use io::*;

/// This is the maximum size of the name of the peripheral
pub const MAX_NAME_SIZE: usize = 30;
/// Represents the default transfer size. For FS it's 8, 16, 32, or 64 bytes. For HS the max is 512 bytes
pub const DEFAULT_PACKET_SIZE: usize = 256;

#[cfg(test)]
mod tests {
    use crate::io_types::*;
    use crate::{
        io::{
            DATA_BYTES_LENGTH_IN_BYTES, MESSAGE_HEADER_SIZE, MESSAGE_MAGIC, MESSAGE_MAGIC_BYTES,
            MESSAGE_TYPE_ID_BYTES,
        },
        MessageType, MessageTypeId, DEFAULT_PACKET_SIZE, IO,
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
        assert_eq!(MESSAGE_MAGIC, 0xDEAD, "Magic number should be 0xDEAD");
        assert_eq!(MESSAGE_MAGIC_BYTES, 2, "Magic number should be 2 bytes");
        assert_eq!(MESSAGE_TYPE_ID_BYTES, 1, "Message type ID should be 1 byte");
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
        assert_eq!(MessageTypeId::HostCommandConfigurePeripheral as u8, 0x01);
        assert_eq!(MessageTypeId::HostCommandConfigureService as u8, 0x02);
        assert_eq!(
            MessageTypeId::HostCommandConfigureCharacteristic as u8,
            0x03
        );
        assert_eq!(
            MessageTypeId::HostCommandConfigureCharacteristicRead as u8,
            0x04
        );
        assert_eq!(MessageTypeId::HostCommandGetServiceInfo as u8, 0x05);
        assert_eq!(MessageTypeId::HostCommandGetCharacteristicInfo as u8, 0x06);
        assert_eq!(MessageTypeId::HostCommandStartAdvertisement as u8, 0x07);
        assert_eq!(
            MessageTypeId::HostCommandNotifyCharacteristicValue as u8,
            0x08
        );
        assert_eq!(MessageTypeId::PluginData as u8, 0x10);
        assert_eq!(MessageTypeId::PluginConfigurationError as u8, 0x11);
        assert_eq!(MessageTypeId::PluginServiceInfoResponse as u8, 0x12);
        assert_eq!(MessageTypeId::PluginCharacteristicInfoResponse as u8, 0x13);
    }

    #[test]
    fn test_message_type_implementations() {
        // Test that each message type has correct MessageType implementation
        assert_eq!(
            HostCommandConfigurePeripheral::message_type_id() as u8,
            MessageTypeId::HostCommandConfigurePeripheral as u8
        );
        assert_eq!(
            HostCommandConfigureService::message_type_id() as u8,
            MessageTypeId::HostCommandConfigureService as u8
        );
        assert_eq!(
            HostCommandConfigureCharacteristic::message_type_id() as u8,
            MessageTypeId::HostCommandConfigureCharacteristic as u8
        );
        assert_eq!(
            HostCommandConfigureCharacteristicRead::message_type_id() as u8,
            MessageTypeId::HostCommandConfigureCharacteristicRead as u8
        );
        assert_eq!(
            HostCommandGetServiceInfo::message_type_id() as u8,
            MessageTypeId::HostCommandGetServiceInfo as u8
        );
        assert_eq!(
            HostCommandGetCharacteristicInfo::message_type_id() as u8,
            MessageTypeId::HostCommandGetCharacteristicInfo as u8
        );
        assert_eq!(
            HostCommandStartAdvertisement::message_type_id() as u8,
            MessageTypeId::HostCommandStartAdvertisement as u8
        );
        assert_eq!(
            HostCommandNotifyCharacteristicValue::message_type_id() as u8,
            MessageTypeId::HostCommandNotifyCharacteristicValue as u8
        );
        assert_eq!(
            PluginConfigurationError::message_type_id() as u8,
            MessageTypeId::PluginConfigurationError as u8
        );
        assert_eq!(
            PluginServiceInfoResponse::message_type_id() as u8,
            MessageTypeId::PluginServiceInfoResponse as u8
        );
        assert_eq!(
            PluginCharacteristicInfoResponse::message_type_id() as u8,
            MessageTypeId::PluginCharacteristicInfoResponse as u8
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_host_command_serialization_with_header() {
        use heapless::String;
        use uuid::Uuid;

        // Test HostCommandConfigurePeripheral serialization
        let cmd = HostCommandConfigurePeripheral {
            name: String::try_from("TestDevice").expect("Should create string"),
            uuid: Uuid::nil(), // Use nil UUID for testing
        };

        let serialized: [u8; DEFAULT_PACKET_SIZE] =
            cmd.to_bytes().expect("Should serialize successfully");

        // Verify magic number in first 2 bytes
        let magic = u16::from_le_bytes([serialized[0], serialized[1]]);
        assert_eq!(
            magic, MESSAGE_MAGIC,
            "Magic number should be present in header"
        );

        // Verify message type ID in byte 2
        let type_id = serialized[MESSAGE_MAGIC_BYTES];
        assert_eq!(
            type_id,
            MessageTypeId::HostCommandConfigurePeripheral as u8,
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
        use heapless::Vec;
        use uuid::Uuid;

        // Test PluginServiceInfoResponse serialization
        let mut char_uuids: Vec<Uuid, 16> = Vec::new();
        char_uuids.push(Uuid::nil()).ok();
        char_uuids.push(Uuid::max()).ok();

        let response = PluginServiceInfoResponse {
            service_uuid: Uuid::nil(),
            characteristic_uuids: char_uuids,
            exists: true,
        };

        let serialized: [u8; DEFAULT_PACKET_SIZE] =
            response.to_bytes().expect("Should serialize successfully");

        // Verify magic number
        let magic = u16::from_le_bytes([serialized[0], serialized[1]]);
        assert_eq!(magic, MESSAGE_MAGIC, "Magic number should be present");

        // Verify message type ID
        let type_id = serialized[MESSAGE_MAGIC_BYTES];
        assert_eq!(
            type_id,
            MessageTypeId::PluginServiceInfoResponse as u8,
            "Message type ID should be correct for plugin response"
        );
    }

    #[test]
    fn test_message_header_roundtrip() {
        use uuid::Uuid;

        // Create a command
        let original_cmd = HostCommandGetServiceInfo { uuid: Uuid::nil() };

        // Serialize with header
        let mut buffer = [0u8; DEFAULT_PACKET_SIZE];
        original_cmd
            .to_bytes_in_slice(&mut buffer)
            .expect("Should serialize");

        // Verify header structure before deserialization
        let magic = u16::from_le_bytes([buffer[0], buffer[1]]);
        assert_eq!(magic, MESSAGE_MAGIC);

        let type_id = buffer[MESSAGE_MAGIC_BYTES];
        assert_eq!(type_id, MessageTypeId::HostCommandGetServiceInfo as u8);

        // Deserialize and verify round-trip
        let deserialized_cmd = HostCommandGetServiceInfo::from_bytes(&buffer)
            .expect("Should deserialize successfully");

        assert_eq!(
            original_cmd.uuid, deserialized_cmd.uuid,
            "Round-trip should preserve data"
        );
    }

    #[test]
    fn test_invalid_magic_number_detection() {
        // Create a buffer with invalid magic number
        let mut invalid_buffer = [0u8; DEFAULT_PACKET_SIZE];

        // Set incorrect magic (should be 0xDEAD but we'll use 0xBEEF)
        invalid_buffer[0] = 0xEF; // 0xBEEF in little-endian
        invalid_buffer[1] = 0xBE;
        invalid_buffer[2] = MessageTypeId::HostCommandGetServiceInfo as u8;
        invalid_buffer[3] = 10; // length
        invalid_buffer[4] = 0;

        // Should fail to deserialize due to invalid magic
        let result = HostCommandGetServiceInfo::from_bytes(&invalid_buffer);
        assert!(
            result.is_err(),
            "Should reject message with invalid magic number"
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
    fn test_message_type_id_uniqueness() {
        use std::collections::HashSet;

        // Collect all message type ID values
        let type_ids = vec![
            MessageTypeId::HostCommandConfigurePeripheral as u8,
            MessageTypeId::HostCommandConfigureService as u8,
            MessageTypeId::HostCommandConfigureCharacteristic as u8,
            MessageTypeId::HostCommandConfigureCharacteristicRead as u8,
            MessageTypeId::HostCommandGetServiceInfo as u8,
            MessageTypeId::HostCommandGetCharacteristicInfo as u8,
            MessageTypeId::HostCommandStartAdvertisement as u8,
            MessageTypeId::HostCommandNotifyCharacteristicValue as u8,
            MessageTypeId::PluginData as u8,
            MessageTypeId::PluginConfigurationError as u8,
            MessageTypeId::PluginServiceInfoResponse as u8,
            MessageTypeId::PluginCharacteristicInfoResponse as u8,
        ];

        // Verify all IDs are unique
        let unique_ids: HashSet<u8> = type_ids.iter().cloned().collect();
        assert_eq!(
            type_ids.len(),
            unique_ids.len(),
            "All message type IDs should be unique"
        );
    }

    #[test]
    fn test_message_header_size_calculation() {
        // Verify that MESSAGE_HEADER_SIZE calculation is correct
        let expected_size = 2 + 1 + 2; // magic + type_id + length
        assert_eq!(MESSAGE_HEADER_SIZE, expected_size);
        assert_eq!(MESSAGE_HEADER_SIZE, 5);
    }

    #[test]
    fn test_invalid_message_length() {
        // Create a buffer with an invalid (too large) length field
        let mut invalid_buffer = [0u8; DEFAULT_PACKET_SIZE];

        // Set correct magic and type ID
        invalid_buffer[0] = (MESSAGE_MAGIC & 0xFF) as u8;
        invalid_buffer[1] = ((MESSAGE_MAGIC >> 8) & 0xFF) as u8;
        invalid_buffer[2] = MessageTypeId::HostCommandGetServiceInfo as u8;

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
    fn test_message_type_id_ranges() {
        // Verify host commands are in 0x01-0x0F range
        assert!((MessageTypeId::HostCommandConfigurePeripheral as u8) < 0x10);
        assert!((MessageTypeId::HostCommandConfigureService as u8) < 0x10);
        assert!((MessageTypeId::HostCommandConfigureCharacteristic as u8) < 0x10);
        assert!((MessageTypeId::HostCommandConfigureCharacteristicRead as u8) < 0x10);
        assert!((MessageTypeId::HostCommandGetServiceInfo as u8) < 0x10);
        assert!((MessageTypeId::HostCommandGetCharacteristicInfo as u8) < 0x10);
        assert!((MessageTypeId::HostCommandStartAdvertisement as u8) < 0x10);
        assert!((MessageTypeId::HostCommandNotifyCharacteristicValue as u8) < 0x10);

        // Verify plugin responses are in 0x10+ range
        assert!((MessageTypeId::PluginData as u8) >= 0x10);
        assert!((MessageTypeId::PluginConfigurationError as u8) >= 0x10);
        assert!((MessageTypeId::PluginServiceInfoResponse as u8) >= 0x10);
        assert!((MessageTypeId::PluginCharacteristicInfoResponse as u8) >= 0x10);
    }
}
