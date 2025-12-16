// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Battery Service profile implementation.
//!
//! Based on Bluetooth SIG Battery Service specification (org.bluetooth.service.battery_service).
//! Service UUID: 0x180F

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Battery Service UUID (16-bit)
pub const BATTERY_SERVICE_UUID: u16 = 0x180F;

/// Battery Level characteristic UUID (16-bit)
pub const BATTERY_LEVEL_UUID: u16 = 0x2A19;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// Default battery level value (100%)
const DEFAULT_BATTERY_LEVEL: u8 = 100;

/// Creates the Battery Service profile definition.
///
/// This profile includes:
/// - Battery Service (0x180F)
///   - Battery Level (0x2A19): Read, Notify (default: 100%)
///
/// The Battery Level characteristic represents the current battery level as a percentage
/// from 0% to 100%, where 0% means fully discharged and 100% means fully charged.
///
/// # Returns
/// A complete `ProfileDefinition` for the Battery Service profile.
pub fn battery_service_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        BATTERY_SERVICE_UUID,
        vec![
            // Battery Level - Read and Notify with default value of 100%
            CharacteristicDefinition::with_default_value(
                BATTERY_LEVEL_UUID,
                vec![PROPERTY_READ, PROPERTY_NOTIFY],
                vec![DEFAULT_BATTERY_LEVEL],
            ),
        ],
    )])
}

/// Helper to create a battery level value byte.
///
/// # Arguments
/// * `level` - Battery level from 0 to 100 (percentage)
///
/// # Returns
/// A single byte representing the battery level
///
/// # Panics
/// Panics if level is greater than 100
pub fn battery_level_value(level: u8) -> Vec<u8> {
    assert!(level <= 100, "Battery level must be 0-100");
    vec![level]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_service_profile_structure() {
        let profile = battery_service_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, BATTERY_SERVICE_UUID);

        // Should have one characteristic
        assert_eq!(service.characteristics.len(), 1);

        // Check Battery Level characteristic
        let battery_level = &service.characteristics[0];
        assert_eq!(battery_level.uuid, BATTERY_LEVEL_UUID);
        assert_eq!(
            battery_level.properties,
            vec![PROPERTY_READ, PROPERTY_NOTIFY]
        );
        assert_eq!(
            battery_level.default_value,
            Some(vec![DEFAULT_BATTERY_LEVEL])
        );
    }

    #[test]
    fn test_battery_level_value() {
        assert_eq!(battery_level_value(0), vec![0]);
        assert_eq!(battery_level_value(50), vec![50]);
        assert_eq!(battery_level_value(100), vec![100]);
    }

    #[test]
    #[should_panic(expected = "Battery level must be 0-100")]
    fn test_battery_level_value_too_high() {
        battery_level_value(101);
    }
}
