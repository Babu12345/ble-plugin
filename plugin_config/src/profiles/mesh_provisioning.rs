// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Mesh Provisioning Profile implementation.
//!
//! Based on Bluetooth SIG Mesh Provisioning Service specification
//! (org.bluetooth.service.mesh_provisioning).
//! Service UUID: 0x1827

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Mesh Provisioning Service UUID (16-bit)
pub const MESH_PROVISIONING_SERVICE_UUID: u16 = 0x1827;

/// Mesh Provisioning Data In characteristic UUID (16-bit)
pub const MESH_PROVISIONING_DATA_IN_UUID: u16 = 0x2ADB;

/// Mesh Provisioning Data Out characteristic UUID (16-bit)
pub const MESH_PROVISIONING_DATA_OUT_UUID: u16 = 0x2ADC;

/// BLE property for Write Without Response
const PROPERTY_WRITE_NO_RSP: i32 = 16;

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4;

/// Mesh Provisioning PDU types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ProvisioningPduType {
    /// Provisioning Invite
    Invite = 0x00,
    /// Provisioning Capabilities
    Capabilities = 0x01,
    /// Provisioning Start
    Start = 0x02,
    /// Provisioning Public Key
    PublicKey = 0x03,
    /// Provisioning Input Complete
    InputComplete = 0x04,
    /// Provisioning Confirmation
    Confirmation = 0x05,
    /// Provisioning Random
    Random = 0x06,
    /// Provisioning Data
    Data = 0x07,
    /// Provisioning Complete
    Complete = 0x08,
    /// Provisioning Failed
    Failed = 0x09,
}

impl ProvisioningPduType {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Mesh Provisioning Error codes
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ProvisioningError {
    /// Prohibited
    Prohibited = 0x00,
    /// Invalid PDU
    InvalidPdu = 0x01,
    /// Invalid Format
    InvalidFormat = 0x02,
    /// Unexpected PDU
    UnexpectedPdu = 0x03,
    /// Confirmation Failed
    ConfirmationFailed = 0x04,
    /// Out of Resources
    OutOfResources = 0x05,
    /// Decryption Failed
    DecryptionFailed = 0x06,
    /// Unexpected Error
    UnexpectedError = 0x07,
    /// Cannot Assign Addresses
    CannotAssignAddresses = 0x08,
}

impl ProvisioningError {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Mesh Provisioning Profile definition.
///
/// This profile enables adding new nodes to a Bluetooth Mesh network:
/// - Mesh Provisioning Service (0x1827)
///   - Mesh Provisioning Data In (0x2ADB): Write Without Response (provisioning commands from provisioner)
///   - Mesh Provisioning Data Out (0x2ADC): Notify (provisioning responses to provisioner)
///
/// Used for:
/// - Smart home device onboarding
/// - Building automation network setup
/// - Industrial IoT mesh networks
/// - Large-scale sensor networks
///
/// # Returns
/// A complete `ProfileDefinition` for the Mesh Provisioning Profile.
pub fn mesh_provisioning_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        MESH_PROVISIONING_SERVICE_UUID,
        vec![
            // Mesh Provisioning Data In - Write Without Response (provisioning commands)
            CharacteristicDefinition::new(
                MESH_PROVISIONING_DATA_IN_UUID,
                vec![PROPERTY_WRITE_NO_RSP],
            ),
            // Mesh Provisioning Data Out - Notify (provisioning responses)
            CharacteristicDefinition::new(MESH_PROVISIONING_DATA_OUT_UUID, vec![PROPERTY_NOTIFY]),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_provisioning_profile_structure() {
        let profile = mesh_provisioning_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, MESH_PROVISIONING_SERVICE_UUID);

        // Should have two characteristics
        assert_eq!(service.characteristics.len(), 2);

        // Check Mesh Provisioning Data In characteristic
        let data_in = &service.characteristics[0];
        assert_eq!(data_in.uuid, MESH_PROVISIONING_DATA_IN_UUID);
        assert_eq!(data_in.properties, vec![PROPERTY_WRITE_NO_RSP]);

        // Check Mesh Provisioning Data Out characteristic
        let data_out = &service.characteristics[1];
        assert_eq!(data_out.uuid, MESH_PROVISIONING_DATA_OUT_UUID);
        assert_eq!(data_out.properties, vec![PROPERTY_NOTIFY]);
    }

    #[test]
    fn test_provisioning_pdu_type_values() {
        assert_eq!(ProvisioningPduType::Invite.as_u8(), 0x00);
        assert_eq!(ProvisioningPduType::Capabilities.as_u8(), 0x01);
        assert_eq!(ProvisioningPduType::Start.as_u8(), 0x02);
        assert_eq!(ProvisioningPduType::PublicKey.as_u8(), 0x03);
        assert_eq!(ProvisioningPduType::Complete.as_u8(), 0x08);
        assert_eq!(ProvisioningPduType::Failed.as_u8(), 0x09);
    }

    #[test]
    fn test_provisioning_error_values() {
        assert_eq!(ProvisioningError::Prohibited.as_u8(), 0x00);
        assert_eq!(ProvisioningError::InvalidPdu.as_u8(), 0x01);
        assert_eq!(ProvisioningError::ConfirmationFailed.as_u8(), 0x04);
        assert_eq!(ProvisioningError::DecryptionFailed.as_u8(), 0x06);
        assert_eq!(ProvisioningError::CannotAssignAddresses.as_u8(), 0x08);
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = mesh_provisioning_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&MESH_PROVISIONING_DATA_IN_UUID));
        assert!(uuids.contains(&MESH_PROVISIONING_DATA_OUT_UUID));
    }
}
