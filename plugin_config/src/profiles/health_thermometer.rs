// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Health Thermometer Service profile implementation.
//!
//! Based on Bluetooth SIG Health Thermometer Service specification
//! (org.bluetooth.service.health_thermometer).
//! Service UUID: 0x1809

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Health Thermometer Service UUID (16-bit)
pub const HEALTH_THERMOMETER_SERVICE_UUID: u16 = 0x1809;

/// Temperature Measurement characteristic UUID (16-bit)
pub const TEMPERATURE_MEASUREMENT_UUID: u16 = 0x2A1C;

/// Temperature Type characteristic UUID (16-bit)
pub const TEMPERATURE_TYPE_UUID: u16 = 0x2A1D;

/// Intermediate Temperature characteristic UUID (16-bit)
pub const INTERMEDIATE_TEMPERATURE_UUID: u16 = 0x2A1E;

/// Measurement Interval characteristic UUID (16-bit)
pub const MEASUREMENT_INTERVAL_UUID: u16 = 0x2A21;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2; // BleProperties::Write

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8; // BleProperties::Indicate

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// Temperature type values as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TemperatureType {
    /// Armpit temperature
    Armpit = 1,
    /// Body temperature (general)
    Body = 2,
    /// Ear temperature (usually eardrum)
    Ear = 3,
    /// Finger temperature
    Finger = 4,
    /// Gastrointestinal Tract temperature
    GastroIntestinalTract = 5,
    /// Mouth temperature
    Mouth = 6,
    /// Rectum temperature
    Rectum = 7,
    /// Toe temperature
    Toe = 8,
    /// Tympanum (eardrum) temperature
    Tympanum = 9,
}

impl TemperatureType {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Health Thermometer Service profile definition.
///
/// This profile includes:
/// - Health Thermometer Service (0x1809)
///   - Temperature Measurement (0x2A1C): Indicate
///   - Temperature Type (0x2A1D): Read (default: Body)
///   - Measurement Interval (0x2A21): Read, Write, Indicate
///
/// # Returns
/// A complete `ProfileDefinition` for the Health Thermometer Service profile.
pub fn health_thermometer_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        HEALTH_THERMOMETER_SERVICE_UUID,
        vec![
            // Temperature Measurement - Indicate (reliable notifications)
            CharacteristicDefinition::new(TEMPERATURE_MEASUREMENT_UUID, vec![PROPERTY_INDICATE]),
            // Temperature Type - Read with default value (Body)
            CharacteristicDefinition::with_default_value(
                TEMPERATURE_TYPE_UUID,
                vec![PROPERTY_READ],
                vec![TemperatureType::Body.as_u8()],
            ),
            // Measurement Interval - Read, Write, Indicate
            CharacteristicDefinition::new(
                MEASUREMENT_INTERVAL_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
        ],
    )])
}

/// Creates an extended Health Thermometer Service profile with intermediate temperature.
///
/// This profile includes all characteristics from the basic profile plus:
///   - Intermediate Temperature (0x2A1E): Notify
///
/// The intermediate temperature characteristic is used to send temperature readings
/// during the measurement process, before the final stable reading is available.
///
/// # Returns
/// A complete `ProfileDefinition` for the extended Health Thermometer Service profile.
pub fn health_thermometer_profile_extended() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        HEALTH_THERMOMETER_SERVICE_UUID,
        vec![
            // Temperature Measurement - Indicate
            CharacteristicDefinition::new(TEMPERATURE_MEASUREMENT_UUID, vec![PROPERTY_INDICATE]),
            // Temperature Type - Read with default value (Body)
            CharacteristicDefinition::with_default_value(
                TEMPERATURE_TYPE_UUID,
                vec![PROPERTY_READ],
                vec![TemperatureType::Body.as_u8()],
            ),
            // Intermediate Temperature - Notify
            CharacteristicDefinition::new(
                INTERMEDIATE_TEMPERATURE_UUID,
                vec![PROPERTY_NOTIFY],
            ),
            // Measurement Interval - Read, Write, Indicate
            CharacteristicDefinition::new(
                MEASUREMENT_INTERVAL_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_thermometer_profile_structure() {
        let profile = health_thermometer_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, HEALTH_THERMOMETER_SERVICE_UUID);

        // Should have three characteristics
        assert_eq!(service.characteristics.len(), 3);

        // Check Temperature Measurement characteristic
        let temp_measurement = &service.characteristics[0];
        assert_eq!(temp_measurement.uuid, TEMPERATURE_MEASUREMENT_UUID);
        assert_eq!(temp_measurement.properties, vec![PROPERTY_INDICATE]);
        assert!(temp_measurement.default_value.is_none());

        // Check Temperature Type characteristic
        let temp_type = &service.characteristics[1];
        assert_eq!(temp_type.uuid, TEMPERATURE_TYPE_UUID);
        assert_eq!(temp_type.properties, vec![PROPERTY_READ]);
        assert_eq!(
            temp_type.default_value,
            Some(vec![TemperatureType::Body.as_u8()])
        );

        // Check Measurement Interval characteristic
        let interval = &service.characteristics[2];
        assert_eq!(interval.uuid, MEASUREMENT_INTERVAL_UUID);
        assert_eq!(
            interval.properties,
            vec![PROPERTY_READ, PROPERTY_WRITE, PROPERTY_INDICATE]
        );
        assert!(interval.default_value.is_none());
    }

    #[test]
    fn test_health_thermometer_profile_extended_structure() {
        let profile = health_thermometer_profile_extended();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, HEALTH_THERMOMETER_SERVICE_UUID);

        // Should have four characteristics
        assert_eq!(service.characteristics.len(), 4);

        // Verify all UUIDs are present
        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&TEMPERATURE_MEASUREMENT_UUID));
        assert!(uuids.contains(&TEMPERATURE_TYPE_UUID));
        assert!(uuids.contains(&INTERMEDIATE_TEMPERATURE_UUID));
        assert!(uuids.contains(&MEASUREMENT_INTERVAL_UUID));
    }

    #[test]
    fn test_temperature_type_values() {
        assert_eq!(TemperatureType::Armpit.as_u8(), 1);
        assert_eq!(TemperatureType::Body.as_u8(), 2);
        assert_eq!(TemperatureType::Ear.as_u8(), 3);
        assert_eq!(TemperatureType::Finger.as_u8(), 4);
        assert_eq!(TemperatureType::GastroIntestinalTract.as_u8(), 5);
        assert_eq!(TemperatureType::Mouth.as_u8(), 6);
        assert_eq!(TemperatureType::Rectum.as_u8(), 7);
        assert_eq!(TemperatureType::Toe.as_u8(), 8);
        assert_eq!(TemperatureType::Tympanum.as_u8(), 9);
    }
}
