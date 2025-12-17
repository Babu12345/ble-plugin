// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Running Speed and Cadence Profile implementation.
//!
//! Based on Bluetooth SIG Running Speed and Cadence Service specification
//! (org.bluetooth.service.running_speed_and_cadence).
//! Service UUID: 0x1814

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Running Speed and Cadence Service UUID (16-bit)
pub const RUNNING_SPEED_CADENCE_SERVICE_UUID: u16 = 0x1814;

/// RSC Measurement characteristic UUID (16-bit)
pub const RSC_MEASUREMENT_UUID: u16 = 0x2A53;

/// RSC Feature characteristic UUID (16-bit)
pub const RSC_FEATURE_UUID: u16 = 0x2A54;

/// Sensor Location characteristic UUID (16-bit)
pub const SENSOR_LOCATION_UUID: u16 = 0x2A5D;

/// SC Control Point characteristic UUID (16-bit)
pub const SC_CONTROL_POINT_UUID: u16 = 0x2A55;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2; // BleProperties::Write

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8; // BleProperties::Indicate

/// RSC Feature flags as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum RscFeature {
    /// Instantaneous Stride Length Measurement Supported (bit 0)
    InstantaneousStrideLengthSupported = 0x0001,
    /// Total Distance Measurement Supported (bit 1)
    TotalDistanceSupported = 0x0002,
    /// Walking or Running Status Supported (bit 2)
    WalkingOrRunningStatusSupported = 0x0004,
    /// Calibration Procedure Supported (bit 3)
    CalibrationProcedureSupported = 0x0008,
    /// Multiple Sensor Locations Supported (bit 4)
    MultipleSensorLocationsSupported = 0x0010,
}

impl RscFeature {
    /// Convert to u16 value
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Sensor location values for Running Speed and Cadence as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SensorLocation {
    /// Other
    Other = 0,
    /// Top of shoe
    TopOfShoe = 1,
    /// In shoe
    InShoe = 2,
    /// Hip
    Hip = 3,
    /// Front Wheel
    FrontWheel = 4,
    /// Left Crank
    LeftCrank = 5,
    /// Right Crank
    RightCrank = 6,
    /// Left Pedal
    LeftPedal = 7,
    /// Right Pedal
    RightPedal = 8,
    /// Front Hub
    FrontHub = 9,
    /// Rear Dropout
    RearDropout = 10,
    /// Chainstay
    Chainstay = 11,
    /// Rear Wheel
    RearWheel = 12,
    /// Rear Hub
    RearHub = 13,
    /// Chest
    Chest = 14,
    /// Spider
    Spider = 15,
    /// Chain Ring
    ChainRing = 16,
}

impl SensorLocation {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Running Speed and Cadence Profile definition.
///
/// This profile includes:
/// - Running Speed and Cadence Service (0x1814)
///   - RSC Measurement (0x2A53): Notify (speed, cadence, stride length, distance)
///   - RSC Feature (0x2A54): Read (supported features)
///   - Sensor Location (0x2A5D): Read (sensor placement)
///   - SC Control Point (0x2A55): Write, Indicate (calibration, control)
///
/// # Returns
/// A complete `ProfileDefinition` for the Running Speed and Cadence Profile.
pub fn running_speed_cadence_profile() -> ProfileDefinition {
    // Default features: stride length, total distance, walking/running status
    let default_features = RscFeature::InstantaneousStrideLengthSupported.as_u16()
        | RscFeature::TotalDistanceSupported.as_u16()
        | RscFeature::WalkingOrRunningStatusSupported.as_u16();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        RUNNING_SPEED_CADENCE_SERVICE_UUID,
        vec![
            // RSC Measurement - Notify (speed, cadence, stride, distance)
            CharacteristicDefinition::new(RSC_MEASUREMENT_UUID, vec![PROPERTY_NOTIFY]),
            // RSC Feature - Read (supported features bitmask)
            CharacteristicDefinition::with_default_value(
                RSC_FEATURE_UUID,
                vec![PROPERTY_READ],
                default_features.to_le_bytes().to_vec(),
            ),
            // Sensor Location - Read (where sensor is mounted)
            CharacteristicDefinition::with_default_value(
                SENSOR_LOCATION_UUID,
                vec![PROPERTY_READ],
                vec![SensorLocation::TopOfShoe.as_u8()],
            ),
            // SC Control Point - Write, Indicate (commands and responses)
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
    fn test_running_speed_cadence_profile_structure() {
        let profile = running_speed_cadence_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, RUNNING_SPEED_CADENCE_SERVICE_UUID);

        // Should have four characteristics
        assert_eq!(service.characteristics.len(), 4);

        // Check RSC Measurement characteristic
        let measurement = &service.characteristics[0];
        assert_eq!(measurement.uuid, RSC_MEASUREMENT_UUID);
        assert_eq!(measurement.properties, vec![PROPERTY_NOTIFY]);
        assert!(measurement.default_value.is_none());

        // Check RSC Feature characteristic
        let feature = &service.characteristics[1];
        assert_eq!(feature.uuid, RSC_FEATURE_UUID);
        assert_eq!(feature.properties, vec![PROPERTY_READ]);
        assert!(feature.default_value.is_some());

        // Check Sensor Location characteristic
        let location = &service.characteristics[2];
        assert_eq!(location.uuid, SENSOR_LOCATION_UUID);
        assert_eq!(location.properties, vec![PROPERTY_READ]);
        assert_eq!(
            location.default_value,
            Some(vec![SensorLocation::TopOfShoe.as_u8()])
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
    fn test_rsc_feature_values() {
        assert_eq!(
            RscFeature::InstantaneousStrideLengthSupported.as_u16(),
            0x0001
        );
        assert_eq!(RscFeature::TotalDistanceSupported.as_u16(), 0x0002);
        assert_eq!(
            RscFeature::WalkingOrRunningStatusSupported.as_u16(),
            0x0004
        );
        assert_eq!(RscFeature::CalibrationProcedureSupported.as_u16(), 0x0008);
        assert_eq!(
            RscFeature::MultipleSensorLocationsSupported.as_u16(),
            0x0010
        );
    }

    #[test]
    fn test_sensor_location_values() {
        assert_eq!(SensorLocation::Other.as_u8(), 0);
        assert_eq!(SensorLocation::TopOfShoe.as_u8(), 1);
        assert_eq!(SensorLocation::InShoe.as_u8(), 2);
        assert_eq!(SensorLocation::Hip.as_u8(), 3);
        assert_eq!(SensorLocation::Chest.as_u8(), 14);
    }

    #[test]
    fn test_default_feature_value() {
        let profile = running_speed_cadence_profile();
        let service = &profile.services[0];
        let feature = &service.characteristics[1];

        let default_features = RscFeature::InstantaneousStrideLengthSupported.as_u16()
            | RscFeature::TotalDistanceSupported.as_u16()
            | RscFeature::WalkingOrRunningStatusSupported.as_u16();

        assert_eq!(
            feature.default_value,
            Some(default_features.to_le_bytes().to_vec())
        );
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = running_speed_cadence_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&RSC_MEASUREMENT_UUID));
        assert!(uuids.contains(&RSC_FEATURE_UUID));
        assert!(uuids.contains(&SENSOR_LOCATION_UUID));
        assert!(uuids.contains(&SC_CONTROL_POINT_UUID));
    }
}
