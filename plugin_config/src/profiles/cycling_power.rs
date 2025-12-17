// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Cycling Power Profile implementation.
//!
//! Based on Bluetooth SIG Cycling Power Service specification
//! (org.bluetooth.service.cycling_power).
//! Service UUID: 0x1818

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Cycling Power Service UUID (16-bit)
pub const CYCLING_POWER_SERVICE_UUID: u16 = 0x1818;

/// Cycling Power Measurement characteristic UUID (16-bit)
pub const CYCLING_POWER_MEASUREMENT_UUID: u16 = 0x2A63;

/// Cycling Power Feature characteristic UUID (16-bit)
pub const CYCLING_POWER_FEATURE_UUID: u16 = 0x2A65;

/// Sensor Location characteristic UUID (16-bit)
pub const SENSOR_LOCATION_UUID: u16 = 0x2A5D;

/// Cycling Power Control Point characteristic UUID (16-bit)
pub const CYCLING_POWER_CONTROL_POINT_UUID: u16 = 0x2A66;

/// Cycling Power Vector characteristic UUID (16-bit)
pub const CYCLING_POWER_VECTOR_UUID: u16 = 0x2A64;

/// BLE property for Read
const PROPERTY_READ: i32 = 1;

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4;

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8;

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2;

/// Cycling Power Feature flags
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum CyclingPowerFeature {
    /// Pedal Power Balance Supported (bit 0)
    PedalPowerBalanceSupported = 0x00000001,
    /// Accumulated Torque Supported (bit 1)
    AccumulatedTorqueSupported = 0x00000002,
    /// Wheel Revolution Data Supported (bit 2)
    WheelRevolutionDataSupported = 0x00000004,
    /// Crank Revolution Data Supported (bit 3)
    CrankRevolutionDataSupported = 0x00000008,
    /// Extreme Magnitudes Supported (bit 4)
    ExtremeMagnitudesSupported = 0x00000010,
    /// Extreme Angles Supported (bit 5)
    ExtremeAnglesSupported = 0x00000020,
    /// Top and Bottom Dead Spot Angles Supported (bit 6)
    DeadSpotAnglesSupported = 0x00000040,
    /// Accumulated Energy Supported (bit 7)
    AccumulatedEnergySupported = 0x00000080,
    /// Offset Compensation Indicator Supported (bit 8)
    OffsetCompensationSupported = 0x00000100,
    /// Offset Compensation Supported (bit 9)
    OffsetCompensationSupported2 = 0x00000200,
    /// Cycling Power Measurement Characteristic Content Masking Supported (bit 10)
    ContentMaskingSupported = 0x00000400,
    /// Multiple Sensor Locations Supported (bit 11)
    MultipleSensorLocationsSupported = 0x00000800,
    /// Crank Length Adjustment Supported (bit 12)
    CrankLengthAdjustmentSupported = 0x00001000,
    /// Chain Length Adjustment Supported (bit 13)
    ChainLengthAdjustmentSupported = 0x00002000,
    /// Chain Weight Adjustment Supported (bit 14)
    ChainWeightAdjustmentSupported = 0x00004000,
    /// Span Length Adjustment Supported (bit 15)
    SpanLengthAdjustmentSupported = 0x00008000,
}

impl CyclingPowerFeature {
    /// Convert to u32 value
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Sensor Location values
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

/// Creates the Cycling Power Profile definition.
///
/// This profile provides power measurement data for cycling:
/// - Cycling Power Service (0x1818)
///   - Cycling Power Measurement (0x2A63): Notify (power data in watts)
///   - Cycling Power Feature (0x2A65): Read (supported features bitmask)
///   - Sensor Location (0x2A5D): Read (where the sensor is mounted)
///   - Cycling Power Control Point (0x2A66): Write, Indicate (configuration)
///   - Cycling Power Vector (0x2A64): Notify (force/torque vector data)
///
/// # Returns
/// A complete `ProfileDefinition` for the Cycling Power Profile.
pub fn cycling_power_profile() -> ProfileDefinition {
    // Default features: basic power measurement with crank revolution data
    let default_features = CyclingPowerFeature::CrankRevolutionDataSupported.as_u32()
        | CyclingPowerFeature::AccumulatedEnergySupported.as_u32();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        CYCLING_POWER_SERVICE_UUID,
        vec![
            // Cycling Power Measurement - Notify (power in watts)
            CharacteristicDefinition::new(CYCLING_POWER_MEASUREMENT_UUID, vec![PROPERTY_NOTIFY]),
            // Cycling Power Feature - Read (supported features bitmask)
            CharacteristicDefinition::with_default_value(
                CYCLING_POWER_FEATURE_UUID,
                vec![PROPERTY_READ],
                default_features.to_le_bytes().to_vec(),
            ),
            // Sensor Location - Read (where sensor is mounted)
            CharacteristicDefinition::with_default_value(
                SENSOR_LOCATION_UUID,
                vec![PROPERTY_READ],
                vec![SensorLocation::LeftCrank.as_u8()],
            ),
            // Cycling Power Control Point - Write, Indicate (configuration commands)
            CharacteristicDefinition::new(
                CYCLING_POWER_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
            // Cycling Power Vector - Notify (force/torque vector data)
            CharacteristicDefinition::new(CYCLING_POWER_VECTOR_UUID, vec![PROPERTY_NOTIFY]),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycling_power_profile_structure() {
        let profile = cycling_power_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, CYCLING_POWER_SERVICE_UUID);

        // Should have five characteristics
        assert_eq!(service.characteristics.len(), 5);

        // Check Cycling Power Measurement characteristic
        let measurement = &service.characteristics[0];
        assert_eq!(measurement.uuid, CYCLING_POWER_MEASUREMENT_UUID);
        assert_eq!(measurement.properties, vec![PROPERTY_NOTIFY]);

        // Check Cycling Power Feature characteristic
        let feature = &service.characteristics[1];
        assert_eq!(feature.uuid, CYCLING_POWER_FEATURE_UUID);
        assert_eq!(feature.properties, vec![PROPERTY_READ]);
        assert!(feature.default_value.is_some());

        // Check Sensor Location characteristic
        let location = &service.characteristics[2];
        assert_eq!(location.uuid, SENSOR_LOCATION_UUID);
        assert_eq!(location.properties, vec![PROPERTY_READ]);
        assert_eq!(
            location.default_value,
            Some(vec![SensorLocation::LeftCrank.as_u8()])
        );
    }

    #[test]
    fn test_cycling_power_feature_values() {
        assert_eq!(
            CyclingPowerFeature::PedalPowerBalanceSupported.as_u32(),
            0x00000001
        );
        assert_eq!(
            CyclingPowerFeature::CrankRevolutionDataSupported.as_u32(),
            0x00000008
        );
        assert_eq!(
            CyclingPowerFeature::AccumulatedEnergySupported.as_u32(),
            0x00000080
        );
        assert_eq!(
            CyclingPowerFeature::MultipleSensorLocationsSupported.as_u32(),
            0x00000800
        );
    }

    #[test]
    fn test_sensor_location_values() {
        assert_eq!(SensorLocation::LeftCrank.as_u8(), 5);
        assert_eq!(SensorLocation::RightCrank.as_u8(), 6);
        assert_eq!(SensorLocation::LeftPedal.as_u8(), 7);
        assert_eq!(SensorLocation::RightPedal.as_u8(), 8);
        assert_eq!(SensorLocation::Spider.as_u8(), 15);
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = cycling_power_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&CYCLING_POWER_MEASUREMENT_UUID));
        assert!(uuids.contains(&CYCLING_POWER_FEATURE_UUID));
        assert!(uuids.contains(&SENSOR_LOCATION_UUID));
        assert!(uuids.contains(&CYCLING_POWER_CONTROL_POINT_UUID));
        assert!(uuids.contains(&CYCLING_POWER_VECTOR_UUID));
    }

    #[test]
    fn test_default_feature_value() {
        let profile = cycling_power_profile();
        let service = &profile.services[0];
        let feature = &service.characteristics[1];

        let default_value = feature.default_value.as_ref().unwrap();
        let features = u32::from_le_bytes([
            default_value[0],
            default_value[1],
            default_value[2],
            default_value[3],
        ]);

        // Check that default features are set correctly
        assert_ne!(
            features & CyclingPowerFeature::CrankRevolutionDataSupported.as_u32(),
            0
        );
        assert_ne!(
            features & CyclingPowerFeature::AccumulatedEnergySupported.as_u32(),
            0
        );
    }
}
