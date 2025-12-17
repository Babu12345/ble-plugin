// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Object Transfer Profile implementation.
//!
//! Based on Bluetooth SIG Object Transfer Service specification
//! (org.bluetooth.service.object_transfer).
//! Service UUID: 0x1825

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Object Transfer Service UUID (16-bit)
pub const OBJECT_TRANSFER_SERVICE_UUID: u16 = 0x1825;

/// OTS Feature characteristic UUID (16-bit)
pub const OTS_FEATURE_UUID: u16 = 0x2ABD;

/// Object Name characteristic UUID (16-bit)
pub const OBJECT_NAME_UUID: u16 = 0x2ABE;

/// Object Type characteristic UUID (16-bit)
pub const OBJECT_TYPE_UUID: u16 = 0x2ABF;

/// Object Size characteristic UUID (16-bit)
pub const OBJECT_SIZE_UUID: u16 = 0x2AC0;

/// Object Properties characteristic UUID (16-bit)
pub const OBJECT_PROPERTIES_UUID: u16 = 0x2AC4;

/// Object Action Control Point characteristic UUID (16-bit)
pub const OBJECT_ACTION_CONTROL_POINT_UUID: u16 = 0x2AC5;

/// Object List Control Point characteristic UUID (16-bit)
pub const OBJECT_LIST_CONTROL_POINT_UUID: u16 = 0x2AC6;

/// BLE property for Read
const PROPERTY_READ: i32 = 1;

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2;

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8;

/// Object Transfer Service Features
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum OtsFeature {
    /// OACP Create Op Code Supported (bit 0)
    OacpCreateSupported = 0x00000001,
    /// OACP Delete Op Code Supported (bit 1)
    OacpDeleteSupported = 0x00000002,
    /// OACP Calculate Checksum Op Code Supported (bit 2)
    OacpChecksumSupported = 0x00000004,
    /// OACP Execute Op Code Supported (bit 3)
    OacpExecuteSupported = 0x00000008,
    /// OACP Read Op Code Supported (bit 4)
    OacpReadSupported = 0x00000010,
    /// OACP Write Op Code Supported (bit 5)
    OacpWriteSupported = 0x00000020,
    /// Appending Additional Data to Objects Supported (bit 6)
    AppendingSupported = 0x00000040,
    /// Truncation of Objects Supported (bit 7)
    TruncationSupported = 0x00000080,
    /// Patching of Objects Supported (bit 8)
    PatchingSupported = 0x00000100,
    /// OACP Abort Op Code Supported (bit 9)
    OacpAbortSupported = 0x00000200,
}

impl OtsFeature {
    /// Convert to u32 value
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Object Action Control Point operation codes
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OacpOpCode {
    /// Create object
    Create = 0x01,
    /// Delete object
    Delete = 0x02,
    /// Calculate checksum
    CalculateChecksum = 0x03,
    /// Execute object
    Execute = 0x04,
    /// Read object
    Read = 0x05,
    /// Write object
    Write = 0x06,
    /// Abort operation
    Abort = 0x07,
}

impl OacpOpCode {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Object Transfer Profile definition.
///
/// This profile enables file and firmware transfer over BLE:
/// - Object Transfer Service (0x1825)
///   - OTS Feature (0x2ABD): Read (supported features)
///   - Object Name (0x2ABE): Read, Write (file name)
///   - Object Type (0x2ABF): Read (file type/MIME type)
///   - Object Size (0x2AC0): Read (current and allocated size)
///   - Object Properties (0x2AC4): Read (file properties)
///   - Object Action Control Point (0x2AC5): Write, Indicate (transfer operations)
///   - Object List Control Point (0x2AC6): Write, Indicate (list management)
///
/// Critical for:
/// - Over-the-Air (OTA) firmware updates
/// - File transfer between devices
/// - Remote device management
/// - Configuration file transfers
///
/// # Returns
/// A complete `ProfileDefinition` for the Object Transfer Profile.
pub fn object_transfer_profile() -> ProfileDefinition {
    // Default features: basic read/write/create/delete operations
    let default_features = OtsFeature::OacpCreateSupported.as_u32()
        | OtsFeature::OacpDeleteSupported.as_u32()
        | OtsFeature::OacpReadSupported.as_u32()
        | OtsFeature::OacpWriteSupported.as_u32()
        | OtsFeature::OacpAbortSupported.as_u32();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        OBJECT_TRANSFER_SERVICE_UUID,
        vec![
            // OTS Feature - Read (supported features bitmask)
            CharacteristicDefinition::with_default_value(
                OTS_FEATURE_UUID,
                vec![PROPERTY_READ],
                default_features.to_le_bytes().to_vec(),
            ),
            // Object Name - Read, Write (file name)
            CharacteristicDefinition::new(OBJECT_NAME_UUID, vec![PROPERTY_READ, PROPERTY_WRITE]),
            // Object Type - Read (file type/MIME type)
            CharacteristicDefinition::new(OBJECT_TYPE_UUID, vec![PROPERTY_READ]),
            // Object Size - Read (current and allocated size)
            CharacteristicDefinition::new(OBJECT_SIZE_UUID, vec![PROPERTY_READ]),
            // Object Properties - Read (file properties)
            CharacteristicDefinition::new(OBJECT_PROPERTIES_UUID, vec![PROPERTY_READ]),
            // Object Action Control Point - Write, Indicate (transfer operations)
            CharacteristicDefinition::new(
                OBJECT_ACTION_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
            // Object List Control Point - Write, Indicate (list management)
            CharacteristicDefinition::new(
                OBJECT_LIST_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_transfer_profile_structure() {
        let profile = object_transfer_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, OBJECT_TRANSFER_SERVICE_UUID);

        // Should have seven characteristics
        assert_eq!(service.characteristics.len(), 7);

        // Check OTS Feature characteristic
        let feature = &service.characteristics[0];
        assert_eq!(feature.uuid, OTS_FEATURE_UUID);
        assert_eq!(feature.properties, vec![PROPERTY_READ]);
        assert!(feature.default_value.is_some());
    }

    #[test]
    fn test_ots_feature_values() {
        assert_eq!(OtsFeature::OacpCreateSupported.as_u32(), 0x00000001);
        assert_eq!(OtsFeature::OacpDeleteSupported.as_u32(), 0x00000002);
        assert_eq!(OtsFeature::OacpReadSupported.as_u32(), 0x00000010);
        assert_eq!(OtsFeature::OacpWriteSupported.as_u32(), 0x00000020);
        assert_eq!(OtsFeature::OacpAbortSupported.as_u32(), 0x00000200);
    }

    #[test]
    fn test_oacp_op_code_values() {
        assert_eq!(OacpOpCode::Create.as_u8(), 0x01);
        assert_eq!(OacpOpCode::Delete.as_u8(), 0x02);
        assert_eq!(OacpOpCode::Read.as_u8(), 0x05);
        assert_eq!(OacpOpCode::Write.as_u8(), 0x06);
        assert_eq!(OacpOpCode::Abort.as_u8(), 0x07);
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = object_transfer_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&OTS_FEATURE_UUID));
        assert!(uuids.contains(&OBJECT_NAME_UUID));
        assert!(uuids.contains(&OBJECT_TYPE_UUID));
        assert!(uuids.contains(&OBJECT_SIZE_UUID));
        assert!(uuids.contains(&OBJECT_PROPERTIES_UUID));
        assert!(uuids.contains(&OBJECT_ACTION_CONTROL_POINT_UUID));
        assert!(uuids.contains(&OBJECT_LIST_CONTROL_POINT_UUID));
    }

    #[test]
    fn test_default_features() {
        let profile = object_transfer_profile();
        let service = &profile.services[0];
        let feature = &service.characteristics[0];

        let default_value = feature.default_value.as_ref().unwrap();
        let features = u32::from_le_bytes([
            default_value[0],
            default_value[1],
            default_value[2],
            default_value[3],
        ]);

        // Check that default features are set correctly
        assert_ne!(features & OtsFeature::OacpCreateSupported.as_u32(), 0);
        assert_ne!(features & OtsFeature::OacpDeleteSupported.as_u32(), 0);
        assert_ne!(features & OtsFeature::OacpReadSupported.as_u32(), 0);
        assert_ne!(features & OtsFeature::OacpWriteSupported.as_u32(), 0);
        assert_ne!(features & OtsFeature::OacpAbortSupported.as_u32(), 0);
    }
}
