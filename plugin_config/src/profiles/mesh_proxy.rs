// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Mesh Proxy Profile implementation.
//!
//! Based on Bluetooth SIG Mesh Proxy Service specification
//! (org.bluetooth.service.mesh_proxy).
//! Service UUID: 0x1828

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Mesh Proxy Service UUID (16-bit)
pub const MESH_PROXY_SERVICE_UUID: u16 = 0x1828;

/// Mesh Proxy Data In characteristic UUID (16-bit)
pub const MESH_PROXY_DATA_IN_UUID: u16 = 0x2ADD;

/// Mesh Proxy Data Out characteristic UUID (16-bit)
pub const MESH_PROXY_DATA_OUT_UUID: u16 = 0x2ADE;

/// BLE property for Write Without Response
const PROPERTY_WRITE_NO_RSP: i32 = 16;

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4;

/// Mesh Proxy PDU types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ProxyPduType {
    /// Network PDU
    NetworkPdu = 0x00,
    /// Mesh Beacon
    MeshBeacon = 0x01,
    /// Proxy Configuration
    ProxyConfiguration = 0x02,
    /// Provisioning PDU
    ProvisioningPdu = 0x03,
}

impl ProxyPduType {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Proxy Configuration OpCodes
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ProxyConfigOpCode {
    /// Set Filter Type
    SetFilterType = 0x00,
    /// Add Addresses to Filter
    AddAddressToFilter = 0x01,
    /// Remove Addresses from Filter
    RemoveAddressFromFilter = 0x02,
    /// Filter Status
    FilterStatus = 0x03,
}

impl ProxyConfigOpCode {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Proxy Filter Types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ProxyFilterType {
    /// Whitelist filter
    Whitelist = 0x00,
    /// Blacklist filter
    Blacklist = 0x01,
}

impl ProxyFilterType {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Mesh Proxy Profile definition.
///
/// This profile enables GATT bearer for Bluetooth Mesh network access:
/// - Mesh Proxy Service (0x1828)
///   - Mesh Proxy Data In (0x2ADD): Write Without Response (mesh messages to network)
///   - Mesh Proxy Data Out (0x2ADE): Notify (mesh messages from network)
///
/// Used for:
/// - Mobile app control of mesh networks
/// - Gateway devices connecting BLE to mesh
/// - Remote monitoring and control
/// - Mesh network diagnostics
///
/// Complements Mesh Provisioning (0x1827) for complete mesh support.
///
/// # Returns
/// A complete `ProfileDefinition` for the Mesh Proxy Profile.
pub fn mesh_proxy_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        MESH_PROXY_SERVICE_UUID,
        vec![
            // Mesh Proxy Data In - Write Without Response (mesh messages to network)
            CharacteristicDefinition::new(MESH_PROXY_DATA_IN_UUID, vec![PROPERTY_WRITE_NO_RSP]),
            // Mesh Proxy Data Out - Notify (mesh messages from network)
            CharacteristicDefinition::new(MESH_PROXY_DATA_OUT_UUID, vec![PROPERTY_NOTIFY]),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_proxy_profile_structure() {
        let profile = mesh_proxy_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, MESH_PROXY_SERVICE_UUID);

        // Should have two characteristics
        assert_eq!(service.characteristics.len(), 2);

        // Check Mesh Proxy Data In characteristic
        let data_in = &service.characteristics[0];
        assert_eq!(data_in.uuid, MESH_PROXY_DATA_IN_UUID);
        assert_eq!(data_in.properties, vec![PROPERTY_WRITE_NO_RSP]);

        // Check Mesh Proxy Data Out characteristic
        let data_out = &service.characteristics[1];
        assert_eq!(data_out.uuid, MESH_PROXY_DATA_OUT_UUID);
        assert_eq!(data_out.properties, vec![PROPERTY_NOTIFY]);
    }

    #[test]
    fn test_proxy_pdu_type_values() {
        assert_eq!(ProxyPduType::NetworkPdu.as_u8(), 0x00);
        assert_eq!(ProxyPduType::MeshBeacon.as_u8(), 0x01);
        assert_eq!(ProxyPduType::ProxyConfiguration.as_u8(), 0x02);
        assert_eq!(ProxyPduType::ProvisioningPdu.as_u8(), 0x03);
    }

    #[test]
    fn test_proxy_config_op_code_values() {
        assert_eq!(ProxyConfigOpCode::SetFilterType.as_u8(), 0x00);
        assert_eq!(ProxyConfigOpCode::AddAddressToFilter.as_u8(), 0x01);
        assert_eq!(ProxyConfigOpCode::RemoveAddressFromFilter.as_u8(), 0x02);
        assert_eq!(ProxyConfigOpCode::FilterStatus.as_u8(), 0x03);
    }

    #[test]
    fn test_proxy_filter_type_values() {
        assert_eq!(ProxyFilterType::Whitelist.as_u8(), 0x00);
        assert_eq!(ProxyFilterType::Blacklist.as_u8(), 0x01);
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = mesh_proxy_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&MESH_PROXY_DATA_IN_UUID));
        assert!(uuids.contains(&MESH_PROXY_DATA_OUT_UUID));
    }
}
