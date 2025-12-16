// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Heart Rate Service profile implementation.
//!
//! Based on Bluetooth SIG Heart Rate Service specification (org.bluetooth.service.heart_rate).
//! Service UUID: 0x180D

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Heart Rate Service UUID (16-bit)
pub const HEART_RATE_SERVICE_UUID: u16 = 0x180D;

/// Heart Rate Measurement characteristic UUID (16-bit)
pub const HEART_RATE_MEASUREMENT_UUID: u16 = 0x2A37;

/// Body Sensor Location characteristic UUID (16-bit)
pub const BODY_SENSOR_LOCATION_UUID: u16 = 0x2A38;

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// Body sensor location values as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BodySensorLocation {
    /// Sensor location: Other
    Other = 0,
    /// Sensor location: Chest
    Chest = 1,
    /// Sensor location: Wrist
    Wrist = 2,
    /// Sensor location: Finger
    Finger = 3,
    /// Sensor location: Hand
    Hand = 4,
    /// Sensor location: Ear Lobe
    EarLobe = 5,
    /// Sensor location: Foot
    Foot = 6,
}

impl BodySensorLocation {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Heart Rate Monitor profile definition.
///
/// This profile includes:
/// - Heart Rate Service (0x180D)
///   - Heart Rate Measurement (0x2A37): Notify
///   - Body Sensor Location (0x2A38): Read (default: Wrist)
///
/// # Returns
/// A complete `ProfileDefinition` for the Heart Rate Monitor profile.
pub fn heart_rate_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        HEART_RATE_SERVICE_UUID,
        vec![
            // Heart Rate Measurement - Notify only
            CharacteristicDefinition::new(HEART_RATE_MEASUREMENT_UUID, vec![PROPERTY_NOTIFY]),
            // Body Sensor Location - Read with default value (Wrist)
            CharacteristicDefinition::with_default_value(
                BODY_SENSOR_LOCATION_UUID,
                vec![PROPERTY_READ],
                vec![BodySensorLocation::Wrist.as_u8()],
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heart_rate_profile_structure() {
        let profile = heart_rate_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, HEART_RATE_SERVICE_UUID);

        // Should have two characteristics
        assert_eq!(service.characteristics.len(), 2);

        // Check Heart Rate Measurement characteristic
        let hr_measurement = &service.characteristics[0];
        assert_eq!(hr_measurement.uuid, HEART_RATE_MEASUREMENT_UUID);
        assert_eq!(hr_measurement.properties, vec![PROPERTY_NOTIFY]);
        assert!(hr_measurement.default_value.is_none());

        // Check Body Sensor Location characteristic
        let body_sensor = &service.characteristics[1];
        assert_eq!(body_sensor.uuid, BODY_SENSOR_LOCATION_UUID);
        assert_eq!(body_sensor.properties, vec![PROPERTY_READ]);
        assert_eq!(
            body_sensor.default_value,
            Some(vec![BodySensorLocation::Wrist.as_u8()])
        );
    }

    #[test]
    fn test_body_sensor_location_values() {
        assert_eq!(BodySensorLocation::Other.as_u8(), 0);
        assert_eq!(BodySensorLocation::Chest.as_u8(), 1);
        assert_eq!(BodySensorLocation::Wrist.as_u8(), 2);
        assert_eq!(BodySensorLocation::Finger.as_u8(), 3);
        assert_eq!(BodySensorLocation::Hand.as_u8(), 4);
        assert_eq!(BodySensorLocation::EarLobe.as_u8(), 5);
        assert_eq!(BodySensorLocation::Foot.as_u8(), 6);
    }
}
