// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Reconnection Configuration Profile implementation.
//!
//! Based on Bluetooth SIG Reconnection Configuration Service specification
//! (org.bluetooth.service.reconnection_configuration).
//! Service UUID: 0x1829

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Reconnection Configuration Service UUID (16-bit)
pub const RECONNECTION_CONFIGURATION_SERVICE_UUID: u16 = 0x1829;

/// RC Features characteristic UUID (16-bit)
pub const RC_FEATURES_UUID: u16 = 0x2B1D;

/// RC Settings characteristic UUID (16-bit)
pub const RC_SETTINGS_UUID: u16 = 0x2B1E;

/// Reconnection Configuration Control Point characteristic UUID (16-bit)
pub const RC_CONTROL_POINT_UUID: u16 = 0x2B1F;

/// BLE property for Read
const PROPERTY_READ: i32 = 1;

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2;

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8;

/// Reconnection Configuration Features
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum RcFeature {
    /// E2E-CRC Supported (bit 0)
    E2eCrcSupported = 0x01,
    /// Bluetooth Address Switching Supported (bit 1)
    AddressSwitchingSupported = 0x02,
    /// On-Demand Switching Supported (bit 2)
    OnDemandSwitchingSupported = 0x04,
    /// Whitelist Support (bit 3)
    WhitelistSupported = 0x08,
}

impl RcFeature {
    /// Convert to u8 value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Reconnection Configuration Control Point operation codes
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum RcOpCode {
    /// Get Reconnection Configuration Settings
    GetSettings = 0x01,
    /// Set Reconnection Configuration Settings
    SetSettings = 0x02,
    /// Upgrade to Lesc Only Mode
    UpgradeToLescOnly = 0x03,
    /// Switch OOB Pairing to Lesc OOB
    SwitchToLescOob = 0x04,
}

impl RcOpCode {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Reconnection Configuration Profile definition.
///
/// This profile enables optimized reconnection configuration for BLE devices:
/// - Reconnection Configuration Service (0x1829)
///   - RC Features (0x2B1D): Read (supported reconnection features)
///   - RC Settings (0x2B1E): Read, Write (reconnection parameters)
///   - RC Control Point (0x2B1F): Write, Indicate (configuration commands)
///
/// This profile is critical for:
/// - Power optimization by reducing reconnection overhead
/// - Improved user experience with faster reconnections
/// - Security enhancement with LESC (LE Secure Connections)
///
/// # Returns
/// A complete `ProfileDefinition` for the Reconnection Configuration Profile.
pub fn reconnection_configuration_profile() -> ProfileDefinition {
    // Default features: E2E-CRC and Address Switching supported
    let default_features = RcFeature::E2eCrcSupported.as_u8()
        | RcFeature::AddressSwitchingSupported.as_u8();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        RECONNECTION_CONFIGURATION_SERVICE_UUID,
        vec![
            // RC Features - Read (supported reconnection features)
            CharacteristicDefinition::with_default_value(
                RC_FEATURES_UUID,
                vec![PROPERTY_READ],
                vec![default_features],
            ),
            // RC Settings - Read, Write (reconnection parameters)
            CharacteristicDefinition::new(RC_SETTINGS_UUID, vec![PROPERTY_READ, PROPERTY_WRITE]),
            // RC Control Point - Write, Indicate (configuration commands)
            CharacteristicDefinition::new(
                RC_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnection_configuration_profile_structure() {
        let profile = reconnection_configuration_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, RECONNECTION_CONFIGURATION_SERVICE_UUID);

        // Should have three characteristics
        assert_eq!(service.characteristics.len(), 3);

        // Check RC Features characteristic
        let features = &service.characteristics[0];
        assert_eq!(features.uuid, RC_FEATURES_UUID);
        assert_eq!(features.properties, vec![PROPERTY_READ]);
        assert!(features.default_value.is_some());

        // Check RC Settings characteristic
        let settings = &service.characteristics[1];
        assert_eq!(settings.uuid, RC_SETTINGS_UUID);
        assert_eq!(settings.properties, vec![PROPERTY_READ, PROPERTY_WRITE]);

        // Check RC Control Point characteristic
        let control_point = &service.characteristics[2];
        assert_eq!(control_point.uuid, RC_CONTROL_POINT_UUID);
        assert_eq!(
            control_point.properties,
            vec![PROPERTY_WRITE, PROPERTY_INDICATE]
        );
    }

    #[test]
    fn test_rc_feature_values() {
        assert_eq!(RcFeature::E2eCrcSupported.as_u8(), 0x01);
        assert_eq!(RcFeature::AddressSwitchingSupported.as_u8(), 0x02);
        assert_eq!(RcFeature::OnDemandSwitchingSupported.as_u8(), 0x04);
        assert_eq!(RcFeature::WhitelistSupported.as_u8(), 0x08);
    }

    #[test]
    fn test_rc_op_code_values() {
        assert_eq!(RcOpCode::GetSettings.as_u8(), 0x01);
        assert_eq!(RcOpCode::SetSettings.as_u8(), 0x02);
        assert_eq!(RcOpCode::UpgradeToLescOnly.as_u8(), 0x03);
        assert_eq!(RcOpCode::SwitchToLescOob.as_u8(), 0x04);
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = reconnection_configuration_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&RC_FEATURES_UUID));
        assert!(uuids.contains(&RC_SETTINGS_UUID));
        assert!(uuids.contains(&RC_CONTROL_POINT_UUID));
    }

    #[test]
    fn test_default_features() {
        let profile = reconnection_configuration_profile();
        let service = &profile.services[0];
        let features = &service.characteristics[0];

        let default_value = features.default_value.as_ref().unwrap();
        let features_byte = default_value[0];

        // Check that default features are set correctly
        assert_ne!(features_byte & RcFeature::E2eCrcSupported.as_u8(), 0);
        assert_ne!(features_byte & RcFeature::AddressSwitchingSupported.as_u8(), 0);
    }
}
