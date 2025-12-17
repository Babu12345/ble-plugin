// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Bond Management Profile implementation.
//!
//! Based on Bluetooth SIG Bond Management Service specification
//! (org.bluetooth.service.bond_management).
//! Service UUID: 0x181E

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Bond Management Service UUID (16-bit)
pub const BOND_MANAGEMENT_SERVICE_UUID: u16 = 0x181E;

/// Bond Management Control Point characteristic UUID (16-bit)
pub const BOND_MANAGEMENT_CONTROL_POINT_UUID: u16 = 0x2AA4;

/// Bond Management Features characteristic UUID (16-bit)
pub const BOND_MANAGEMENT_FEATURES_UUID: u16 = 0x2AA5;

/// BLE property for Read
const PROPERTY_READ: i32 = 1;

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2;

/// Bond Management Features flags
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum BondManagementFeature {
    /// Delete bond of requesting device supported (bit 0)
    DeleteBondRequestingDeviceSupported = 0x00000001,
    /// Delete bond of requesting device with authorization code supported (bit 1)
    DeleteBondWithAuthCodeSupported = 0x00000002,
    /// Delete all bonds supported (bit 2)
    DeleteAllBondsSupported = 0x00000004,
    /// Delete all bonds with authorization code supported (bit 3)
    DeleteAllBondsWithAuthCodeSupported = 0x00000008,
    /// Delete all bonds except requesting device supported (bit 4)
    DeleteAllExceptRequestingDeviceSupported = 0x00000010,
    /// Delete all bonds except requesting device with authorization code supported (bit 5)
    DeleteAllExceptWithAuthCodeSupported = 0x00000020,
}

impl BondManagementFeature {
    /// Convert to u32 value
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Bond Management Control Point operation codes
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BondManagementOpCode {
    /// Delete bond of requesting device
    DeleteBondRequestingDevice = 0x03,
    /// Delete all bonds
    DeleteAllBonds = 0x06,
    /// Delete all bonds except requesting device
    DeleteAllExceptRequestingDevice = 0x09,
}

impl BondManagementOpCode {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Bond Management Profile definition.
///
/// This profile allows management of bonding information between devices:
/// - Bond Management Service (0x181E)
///   - Bond Management Control Point (0x2AA4): Write (delete bonds)
///   - Bond Management Features (0x2AA5): Read (supported operations)
///
/// # Returns
/// A complete `ProfileDefinition` for the Bond Management Profile.
pub fn bond_management_profile() -> ProfileDefinition {
    // Default features: support basic bond deletion operations
    let default_features = BondManagementFeature::DeleteBondRequestingDeviceSupported.as_u32()
        | BondManagementFeature::DeleteAllBondsSupported.as_u32()
        | BondManagementFeature::DeleteAllExceptRequestingDeviceSupported.as_u32();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        BOND_MANAGEMENT_SERVICE_UUID,
        vec![
            // Bond Management Control Point - Write (operations to manage bonds)
            CharacteristicDefinition::new(BOND_MANAGEMENT_CONTROL_POINT_UUID, vec![PROPERTY_WRITE]),
            // Bond Management Features - Read (supported features bitmask)
            CharacteristicDefinition::with_default_value(
                BOND_MANAGEMENT_FEATURES_UUID,
                vec![PROPERTY_READ],
                default_features.to_le_bytes().to_vec(),
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bond_management_profile_structure() {
        let profile = bond_management_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, BOND_MANAGEMENT_SERVICE_UUID);

        // Should have two characteristics
        assert_eq!(service.characteristics.len(), 2);

        // Check Bond Management Control Point characteristic
        let control_point = &service.characteristics[0];
        assert_eq!(control_point.uuid, BOND_MANAGEMENT_CONTROL_POINT_UUID);
        assert_eq!(control_point.properties, vec![PROPERTY_WRITE]);

        // Check Bond Management Features characteristic
        let features = &service.characteristics[1];
        assert_eq!(features.uuid, BOND_MANAGEMENT_FEATURES_UUID);
        assert_eq!(features.properties, vec![PROPERTY_READ]);
        assert!(features.default_value.is_some());
    }

    #[test]
    fn test_bond_management_feature_values() {
        assert_eq!(
            BondManagementFeature::DeleteBondRequestingDeviceSupported.as_u32(),
            0x00000001
        );
        assert_eq!(
            BondManagementFeature::DeleteBondWithAuthCodeSupported.as_u32(),
            0x00000002
        );
        assert_eq!(
            BondManagementFeature::DeleteAllBondsSupported.as_u32(),
            0x00000004
        );
        assert_eq!(
            BondManagementFeature::DeleteAllBondsWithAuthCodeSupported.as_u32(),
            0x00000008
        );
        assert_eq!(
            BondManagementFeature::DeleteAllExceptRequestingDeviceSupported.as_u32(),
            0x00000010
        );
    }

    #[test]
    fn test_bond_management_op_code_values() {
        assert_eq!(
            BondManagementOpCode::DeleteBondRequestingDevice.as_u8(),
            0x03
        );
        assert_eq!(BondManagementOpCode::DeleteAllBonds.as_u8(), 0x06);
        assert_eq!(
            BondManagementOpCode::DeleteAllExceptRequestingDevice.as_u8(),
            0x09
        );
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = bond_management_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&BOND_MANAGEMENT_CONTROL_POINT_UUID));
        assert!(uuids.contains(&BOND_MANAGEMENT_FEATURES_UUID));
    }

    #[test]
    fn test_default_features() {
        let profile = bond_management_profile();
        let service = &profile.services[0];
        let features = &service.characteristics[1];

        let default_value = features.default_value.as_ref().unwrap();
        let features_bits = u32::from_le_bytes([
            default_value[0],
            default_value[1],
            default_value[2],
            default_value[3],
        ]);

        // Check that default features are set correctly
        assert_ne!(features_bits & BondManagementFeature::DeleteBondRequestingDeviceSupported.as_u32(), 0);
        assert_ne!(features_bits & BondManagementFeature::DeleteAllBondsSupported.as_u32(), 0);
        assert_ne!(features_bits & BondManagementFeature::DeleteAllExceptRequestingDeviceSupported.as_u32(), 0);
    }
}
