// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Cycling Speed and Cadence Service profile implementation.
//!
//! Based on Bluetooth SIG Cycling Speed and Cadence Service specification
//! (org.bluetooth.service.cycling_speed_and_cadence).
//! Service UUID: 0x1816

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Cycling Speed and Cadence Service UUID (16-bit)
pub const CYCLING_SPEED_CADENCE_SERVICE_UUID: u16 = 0x1816;

/// CSC Measurement characteristic UUID (16-bit)
pub const CSC_MEASUREMENT_UUID: u16 = 0x2A5B;

/// CSC Feature characteristic UUID (16-bit)
pub const CSC_FEATURE_UUID: u16 = 0x2A5C;

/// Sensor Location characteristic UUID (16-bit)
pub const SENSOR_LOCATION_UUID: u16 = 0x2A5D;

/// SC Control Point characteristic UUID (16-bit)
pub const SC_CONTROL_POINT_UUID: u16 = 0x2A55;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2; // BleProperties::Write

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8; // BleProperties::Indicate

/// Sensor location values as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SensorLocation {
    /// Other location
    Other = 0,
    /// Top of shoe
    TopOfShoe = 1,
    /// In shoe
    InShoe = 2,
    /// Hip
    Hip = 3,
    /// Front wheel
    FrontWheel = 4,
    /// Left crank
    LeftCrank = 5,
    /// Right crank
    RightCrank = 6,
    /// Left pedal
    LeftPedal = 7,
    /// Right pedal
    RightPedal = 8,
    /// Front hub
    FrontHub = 9,
    /// Rear dropout
    RearDropout = 10,
    /// Chainstay
    Chainstay = 11,
    /// Rear wheel
    RearWheel = 12,
    /// Rear hub
    RearHub = 13,
    /// Chest
    Chest = 14,
}

impl SensorLocation {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Cycling Speed and Cadence Service profile definition.
///
/// This profile includes:
/// - Cycling Speed and Cadence Service (0x1816)
///   - CSC Measurement (0x2A5B): Notify (speed and cadence data)
///   - CSC Feature (0x2A5C): Read (supported features bitmap)
///   - Sensor Location (0x2A5D): Read (default: Rear Wheel)
///   - SC Control Point (0x2A55): Write, Indicate (calibration/configuration)
///
/// # Returns
/// A complete `ProfileDefinition` for the Cycling Speed and Cadence Service profile.
pub fn cycling_speed_cadence_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        CYCLING_SPEED_CADENCE_SERVICE_UUID,
        vec![
            // CSC Measurement - Notify (contains speed and cadence data)
            CharacteristicDefinition::new(CSC_MEASUREMENT_UUID, vec![PROPERTY_NOTIFY]),
            // CSC Feature - Read (indicates what features are supported)
            CharacteristicDefinition::new(CSC_FEATURE_UUID, vec![PROPERTY_READ]),
            // Sensor Location - Read with default value (Rear Wheel for bike computer)
            CharacteristicDefinition::with_default_value(
                SENSOR_LOCATION_UUID,
                vec![PROPERTY_READ],
                vec![SensorLocation::RearWheel.as_u8()],
            ),
            // SC Control Point - Write and Indicate (for calibration/configuration)
            CharacteristicDefinition::new(
                SC_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycling_speed_cadence_profile_structure() {
        let profile = cycling_speed_cadence_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, CYCLING_SPEED_CADENCE_SERVICE_UUID);

        // Should have four characteristics
        assert_eq!(service.characteristics.len(), 4);

        // Check CSC Measurement characteristic
        let csc_measurement = &service.characteristics[0];
        assert_eq!(csc_measurement.uuid, CSC_MEASUREMENT_UUID);
        assert_eq!(csc_measurement.properties, vec![PROPERTY_NOTIFY]);
        assert!(csc_measurement.default_value.is_none());

        // Check CSC Feature characteristic
        let csc_feature = &service.characteristics[1];
        assert_eq!(csc_feature.uuid, CSC_FEATURE_UUID);
        assert_eq!(csc_feature.properties, vec![PROPERTY_READ]);
        assert!(csc_feature.default_value.is_none());

        // Check Sensor Location characteristic
        let sensor_location = &service.characteristics[2];
        assert_eq!(sensor_location.uuid, SENSOR_LOCATION_UUID);
        assert_eq!(sensor_location.properties, vec![PROPERTY_READ]);
        assert_eq!(
            sensor_location.default_value,
            Some(vec![SensorLocation::RearWheel.as_u8()])
        );

        // Check SC Control Point characteristic
        let control_point = &service.characteristics[3];
        assert_eq!(control_point.uuid, SC_CONTROL_POINT_UUID);
        assert_eq!(
            control_point.properties,
            vec![PROPERTY_WRITE, PROPERTY_INDICATE]
        );
        assert!(control_point.default_value.is_none());
    }

    #[test]
    fn test_sensor_location_values() {
        assert_eq!(SensorLocation::Other.as_u8(), 0);
        assert_eq!(SensorLocation::TopOfShoe.as_u8(), 1);
        assert_eq!(SensorLocation::InShoe.as_u8(), 2);
        assert_eq!(SensorLocation::Hip.as_u8(), 3);
        assert_eq!(SensorLocation::FrontWheel.as_u8(), 4);
        assert_eq!(SensorLocation::LeftCrank.as_u8(), 5);
        assert_eq!(SensorLocation::RightCrank.as_u8(), 6);
        assert_eq!(SensorLocation::LeftPedal.as_u8(), 7);
        assert_eq!(SensorLocation::RightPedal.as_u8(), 8);
        assert_eq!(SensorLocation::FrontHub.as_u8(), 9);
        assert_eq!(SensorLocation::RearDropout.as_u8(), 10);
        assert_eq!(SensorLocation::Chainstay.as_u8(), 11);
        assert_eq!(SensorLocation::RearWheel.as_u8(), 12);
        assert_eq!(SensorLocation::RearHub.as_u8(), 13);
        assert_eq!(SensorLocation::Chest.as_u8(), 14);
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = cycling_speed_cadence_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&CSC_MEASUREMENT_UUID));
        assert!(uuids.contains(&CSC_FEATURE_UUID));
        assert!(uuids.contains(&SENSOR_LOCATION_UUID));
        assert!(uuids.contains(&SC_CONTROL_POINT_UUID));
    }
}
