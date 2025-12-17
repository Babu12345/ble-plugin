// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Blood Pressure Profile implementation.
//!
//! Based on Bluetooth SIG Blood Pressure Service specification
//! (org.bluetooth.service.blood_pressure).
//! Service UUID: 0x1810

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Blood Pressure Service UUID (16-bit)
pub const BLOOD_PRESSURE_SERVICE_UUID: u16 = 0x1810;

/// Blood Pressure Measurement characteristic UUID (16-bit)
pub const BLOOD_PRESSURE_MEASUREMENT_UUID: u16 = 0x2A35;

/// Intermediate Cuff Pressure characteristic UUID (16-bit)
pub const INTERMEDIATE_CUFF_PRESSURE_UUID: u16 = 0x2A36;

/// Blood Pressure Feature characteristic UUID (16-bit)
pub const BLOOD_PRESSURE_FEATURE_UUID: u16 = 0x2A49;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8; // BleProperties::Indicate

/// Blood Pressure Feature flags as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum BloodPressureFeatureFlags {
    /// Body Movement Detection Support
    BodyMovementDetection = 0x0001,
    /// Cuff Fit Detection Support
    CuffFitDetection = 0x0002,
    /// Irregular Pulse Detection Support
    IrregularPulseDetection = 0x0004,
    /// Pulse Rate Range Detection Support
    PulseRateRangeDetection = 0x0008,
    /// Measurement Position Detection Support
    MeasurementPositionDetection = 0x0010,
    /// Multiple Bond Support
    MultipleBond = 0x0020,
    /// E2E-CRC Support
    E2ECrcSupport = 0x0040,
    /// User Data Service Support
    UserDataService = 0x0080,
    /// User Facing Time Support
    UserFacingTime = 0x0100,
}

impl BloodPressureFeatureFlags {
    /// Convert to u16 value
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Creates the Blood Pressure Profile definition.
///
/// This profile includes:
/// - Blood Pressure Service (0x1810)
///   - Blood Pressure Measurement (0x2A35): Indicate (systolic, diastolic, MAP)
///   - Blood Pressure Feature (0x2A49): Read (supported features bitmap)
///
/// # Returns
/// A complete `ProfileDefinition` for the Blood Pressure Profile.
pub fn blood_pressure_profile() -> ProfileDefinition {
    // Default feature set: irregular pulse detection
    let default_features: u16 = BloodPressureFeatureFlags::IrregularPulseDetection.as_u16();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        BLOOD_PRESSURE_SERVICE_UUID,
        vec![
            // Blood Pressure Measurement - Indicate (reliable notifications for BP readings)
            CharacteristicDefinition::new(
                BLOOD_PRESSURE_MEASUREMENT_UUID,
                vec![PROPERTY_INDICATE],
            ),
            // Blood Pressure Feature - Read (indicates supported features)
            CharacteristicDefinition::with_default_value(
                BLOOD_PRESSURE_FEATURE_UUID,
                vec![PROPERTY_READ],
                default_features.to_le_bytes().to_vec(),
            ),
        ],
    )])
}

/// Creates an extended Blood Pressure Profile with intermediate cuff pressure.
///
/// This profile includes all characteristics from the basic profile plus:
///   - Intermediate Cuff Pressure (0x2A36): Notify
///
/// The intermediate cuff pressure characteristic is used to send cuff pressure
/// readings during the measurement process, before the final BP reading is available.
///
/// # Returns
/// A complete `ProfileDefinition` for the extended Blood Pressure Profile.
pub fn blood_pressure_profile_extended() -> ProfileDefinition {
    // Default feature set: irregular pulse detection
    let default_features: u16 = BloodPressureFeatureFlags::IrregularPulseDetection.as_u16();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        BLOOD_PRESSURE_SERVICE_UUID,
        vec![
            // Blood Pressure Measurement - Indicate
            CharacteristicDefinition::new(
                BLOOD_PRESSURE_MEASUREMENT_UUID,
                vec![PROPERTY_INDICATE],
            ),
            // Intermediate Cuff Pressure - Notify (cuff pressure during measurement)
            CharacteristicDefinition::new(
                INTERMEDIATE_CUFF_PRESSURE_UUID,
                vec![PROPERTY_NOTIFY],
            ),
            // Blood Pressure Feature - Read
            CharacteristicDefinition::with_default_value(
                BLOOD_PRESSURE_FEATURE_UUID,
                vec![PROPERTY_READ],
                default_features.to_le_bytes().to_vec(),
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blood_pressure_profile_structure() {
        let profile = blood_pressure_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, BLOOD_PRESSURE_SERVICE_UUID);

        // Should have two characteristics
        assert_eq!(service.characteristics.len(), 2);

        // Check Blood Pressure Measurement characteristic
        let measurement = &service.characteristics[0];
        assert_eq!(measurement.uuid, BLOOD_PRESSURE_MEASUREMENT_UUID);
        assert_eq!(measurement.properties, vec![PROPERTY_INDICATE]);
        assert!(measurement.default_value.is_none());

        // Check Blood Pressure Feature characteristic
        let feature = &service.characteristics[1];
        assert_eq!(feature.uuid, BLOOD_PRESSURE_FEATURE_UUID);
        assert_eq!(feature.properties, vec![PROPERTY_READ]);
        assert!(feature.default_value.is_some());
    }

    #[test]
    fn test_blood_pressure_profile_extended_structure() {
        let profile = blood_pressure_profile_extended();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, BLOOD_PRESSURE_SERVICE_UUID);

        // Should have three characteristics
        assert_eq!(service.characteristics.len(), 3);

        // Verify all UUIDs are present
        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&BLOOD_PRESSURE_MEASUREMENT_UUID));
        assert!(uuids.contains(&INTERMEDIATE_CUFF_PRESSURE_UUID));
        assert!(uuids.contains(&BLOOD_PRESSURE_FEATURE_UUID));
    }

    #[test]
    fn test_blood_pressure_feature_flags() {
        assert_eq!(
            BloodPressureFeatureFlags::BodyMovementDetection.as_u16(),
            0x0001
        );
        assert_eq!(
            BloodPressureFeatureFlags::CuffFitDetection.as_u16(),
            0x0002
        );
        assert_eq!(
            BloodPressureFeatureFlags::IrregularPulseDetection.as_u16(),
            0x0004
        );
        assert_eq!(
            BloodPressureFeatureFlags::PulseRateRangeDetection.as_u16(),
            0x0008
        );
        assert_eq!(
            BloodPressureFeatureFlags::MeasurementPositionDetection.as_u16(),
            0x0010
        );
        assert_eq!(BloodPressureFeatureFlags::MultipleBond.as_u16(), 0x0020);
        assert_eq!(BloodPressureFeatureFlags::E2ECrcSupport.as_u16(), 0x0040);
        assert_eq!(
            BloodPressureFeatureFlags::UserDataService.as_u16(),
            0x0080
        );
        assert_eq!(
            BloodPressureFeatureFlags::UserFacingTime.as_u16(),
            0x0100
        );
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = blood_pressure_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&BLOOD_PRESSURE_MEASUREMENT_UUID));
        assert!(uuids.contains(&BLOOD_PRESSURE_FEATURE_UUID));
    }

    #[test]
    fn test_default_feature_value() {
        let profile = blood_pressure_profile();
        let service = &profile.services[0];
        let feature_char = &service.characteristics[1];

        // Should have irregular pulse detection enabled by default
        let expected: u16 = BloodPressureFeatureFlags::IrregularPulseDetection.as_u16();

        assert_eq!(
            feature_char.default_value,
            Some(expected.to_le_bytes().to_vec())
        );
    }
}
