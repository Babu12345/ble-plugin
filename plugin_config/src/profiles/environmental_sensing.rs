// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Environmental Sensing Service profile implementation.
//!
//! Based on Bluetooth SIG Environmental Sensing Service specification
//! (org.bluetooth.service.environmental_sensing).
//! Service UUID: 0x181A

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Environmental Sensing Service UUID (16-bit)
pub const ENVIRONMENTAL_SENSING_SERVICE_UUID: u16 = 0x181A;

/// Temperature characteristic UUID (16-bit)
pub const TEMPERATURE_UUID: u16 = 0x2A6E;

/// Humidity characteristic UUID (16-bit)
pub const HUMIDITY_UUID: u16 = 0x2A6F;

/// Pressure characteristic UUID (16-bit)
pub const PRESSURE_UUID: u16 = 0x2A6D;

/// UV Index characteristic UUID (16-bit)
pub const UV_INDEX_UUID: u16 = 0x2A76;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// Creates the Environmental Sensing Service profile definition.
///
/// This profile includes:
/// - Environmental Sensing Service (0x181A)
///   - Temperature (0x2A6E): Read, Notify
///   - Humidity (0x2A6F): Read, Notify
///   - Pressure (0x2A6D): Read, Notify
///
/// All characteristics support both reading current values and receiving
/// notifications when values change.
///
/// # Returns
/// A complete `ProfileDefinition` for the Environmental Sensing Service profile.
pub fn environmental_sensing_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        ENVIRONMENTAL_SENSING_SERVICE_UUID,
        vec![
            // Temperature - Read and Notify
            CharacteristicDefinition::new(TEMPERATURE_UUID, vec![PROPERTY_READ, PROPERTY_NOTIFY]),
            // Humidity - Read and Notify
            CharacteristicDefinition::new(HUMIDITY_UUID, vec![PROPERTY_READ, PROPERTY_NOTIFY]),
            // Pressure - Read and Notify
            CharacteristicDefinition::new(PRESSURE_UUID, vec![PROPERTY_READ, PROPERTY_NOTIFY]),
        ],
    )])
}

/// Creates an extended Environmental Sensing Service profile with UV Index.
///
/// This profile includes all characteristics from the basic profile plus:
///   - UV Index (0x2A76): Read, Notify
///
/// # Returns
/// A complete `ProfileDefinition` for the extended Environmental Sensing Service profile.
pub fn environmental_sensing_profile_extended() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        ENVIRONMENTAL_SENSING_SERVICE_UUID,
        vec![
            // Temperature - Read and Notify
            CharacteristicDefinition::new(TEMPERATURE_UUID, vec![PROPERTY_READ, PROPERTY_NOTIFY]),
            // Humidity - Read and Notify
            CharacteristicDefinition::new(HUMIDITY_UUID, vec![PROPERTY_READ, PROPERTY_NOTIFY]),
            // Pressure - Read and Notify
            CharacteristicDefinition::new(PRESSURE_UUID, vec![PROPERTY_READ, PROPERTY_NOTIFY]),
            // UV Index - Read and Notify
            CharacteristicDefinition::new(UV_INDEX_UUID, vec![PROPERTY_READ, PROPERTY_NOTIFY]),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environmental_sensing_profile_structure() {
        let profile = environmental_sensing_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, ENVIRONMENTAL_SENSING_SERVICE_UUID);

        // Should have three characteristics
        assert_eq!(service.characteristics.len(), 3);

        // Check Temperature characteristic
        let temperature = &service.characteristics[0];
        assert_eq!(temperature.uuid, TEMPERATURE_UUID);
        assert_eq!(
            temperature.properties,
            vec![PROPERTY_READ, PROPERTY_NOTIFY]
        );
        assert!(temperature.default_value.is_none());

        // Check Humidity characteristic
        let humidity = &service.characteristics[1];
        assert_eq!(humidity.uuid, HUMIDITY_UUID);
        assert_eq!(humidity.properties, vec![PROPERTY_READ, PROPERTY_NOTIFY]);
        assert!(humidity.default_value.is_none());

        // Check Pressure characteristic
        let pressure = &service.characteristics[2];
        assert_eq!(pressure.uuid, PRESSURE_UUID);
        assert_eq!(pressure.properties, vec![PROPERTY_READ, PROPERTY_NOTIFY]);
        assert!(pressure.default_value.is_none());
    }

    #[test]
    fn test_environmental_sensing_profile_extended_structure() {
        let profile = environmental_sensing_profile_extended();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, ENVIRONMENTAL_SENSING_SERVICE_UUID);

        // Should have four characteristics
        assert_eq!(service.characteristics.len(), 4);

        // Verify all UUIDs are present
        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&TEMPERATURE_UUID));
        assert!(uuids.contains(&HUMIDITY_UUID));
        assert!(uuids.contains(&PRESSURE_UUID));
        assert!(uuids.contains(&UV_INDEX_UUID));
    }

    #[test]
    fn test_all_characteristics_support_read_and_notify() {
        let profile = environmental_sensing_profile();
        for characteristic in profile.services[0].characteristics.iter() {
            assert_eq!(
                characteristic.properties,
                vec![PROPERTY_READ, PROPERTY_NOTIFY],
                "Characteristic {} should support Read and Notify",
                characteristic.uuid
            );
        }
    }
}
