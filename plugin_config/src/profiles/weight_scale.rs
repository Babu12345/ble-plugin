// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Weight Scale Profile implementation.
//!
//! Based on Bluetooth SIG Weight Scale Service specification
//! (org.bluetooth.service.weight_scale).
//! Service UUID: 0x181D

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Weight Scale Service UUID (16-bit)
pub const WEIGHT_SCALE_SERVICE_UUID: u16 = 0x181D;

/// Weight Measurement characteristic UUID (16-bit)
pub const WEIGHT_MEASUREMENT_UUID: u16 = 0x2A9D;

/// Weight Scale Feature characteristic UUID (16-bit)
pub const WEIGHT_SCALE_FEATURE_UUID: u16 = 0x2A9E;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8; // BleProperties::Indicate

/// Weight Scale Feature flags as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum WeightScaleFeatureFlags {
    /// Time Stamp Supported
    TimeStamp = 0x0001,
    /// Multiple Users Supported
    MultipleUsers = 0x0002,
    /// BMI Supported
    BmiSupported = 0x0004,
    /// Weight Display Resolution: 0.5 kg or 1 lb
    DisplayResolution05kg = 0x0008,
    /// Weight Display Resolution: 0.2 kg or 0.5 lb
    DisplayResolution02kg = 0x0010,
    /// Weight Display Resolution: 0.1 kg or 0.2 lb
    DisplayResolution01kg = 0x0018,
    /// Weight Display Resolution: 0.05 kg or 0.1 lb
    DisplayResolution005kg = 0x0020,
    /// Weight Display Resolution: 0.02 kg or 0.05 lb
    DisplayResolution002kg = 0x0028,
    /// Weight Display Resolution: 0.01 kg or 0.02 lb
    DisplayResolution001kg = 0x0030,
    /// Weight Display Resolution: 0.005 kg or 0.01 lb
    DisplayResolution0005kg = 0x0038,
    /// Height Display Resolution: 0.01 meter or 1 inch
    HeightResolution001m = 0x0080,
    /// Height Display Resolution: 0.005 meter or 0.5 inch
    HeightResolution0005m = 0x0100,
    /// Height Display Resolution: 0.001 meter or 0.1 inch
    HeightResolution0001m = 0x0180,
}

impl WeightScaleFeatureFlags {
    /// Convert to u32 value
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Creates the Weight Scale Profile definition.
///
/// This profile includes:
/// - Weight Scale Service (0x181D)
///   - Weight Measurement (0x2A9D): Indicate (weight, BMI, timestamp)
///   - Weight Scale Feature (0x2A9E): Read (supported features bitmap)
///
/// # Returns
/// A complete `ProfileDefinition` for the Weight Scale Profile.
pub fn weight_scale_profile() -> ProfileDefinition {
    // Default feature set: timestamp, BMI, and 0.1 kg resolution
    let default_features: u32 = WeightScaleFeatureFlags::TimeStamp.as_u32()
        | WeightScaleFeatureFlags::BmiSupported.as_u32()
        | WeightScaleFeatureFlags::DisplayResolution01kg.as_u32();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        WEIGHT_SCALE_SERVICE_UUID,
        vec![
            // Weight Measurement - Indicate (reliable notifications for weight readings)
            CharacteristicDefinition::new(WEIGHT_MEASUREMENT_UUID, vec![PROPERTY_INDICATE]),
            // Weight Scale Feature - Read (indicates supported features)
            CharacteristicDefinition::with_default_value(
                WEIGHT_SCALE_FEATURE_UUID,
                vec![PROPERTY_READ],
                default_features.to_le_bytes().to_vec(),
            ),
        ],
    )])
}

/// Creates an extended Weight Scale Profile with multi-user support.
///
/// This profile includes the same characteristics as the basic profile but
/// with multi-user support enabled in the feature flags.
///
/// # Returns
/// A complete `ProfileDefinition` for the extended Weight Scale Profile.
pub fn weight_scale_profile_multi_user() -> ProfileDefinition {
    // Extended feature set: timestamp, multi-user, BMI, and 0.1 kg resolution
    let extended_features: u32 = WeightScaleFeatureFlags::TimeStamp.as_u32()
        | WeightScaleFeatureFlags::MultipleUsers.as_u32()
        | WeightScaleFeatureFlags::BmiSupported.as_u32()
        | WeightScaleFeatureFlags::DisplayResolution01kg.as_u32();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        WEIGHT_SCALE_SERVICE_UUID,
        vec![
            // Weight Measurement - Indicate
            CharacteristicDefinition::new(WEIGHT_MEASUREMENT_UUID, vec![PROPERTY_INDICATE]),
            // Weight Scale Feature - Read (with multi-user support)
            CharacteristicDefinition::with_default_value(
                WEIGHT_SCALE_FEATURE_UUID,
                vec![PROPERTY_READ],
                extended_features.to_le_bytes().to_vec(),
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weight_scale_profile_structure() {
        let profile = weight_scale_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, WEIGHT_SCALE_SERVICE_UUID);

        // Should have two characteristics
        assert_eq!(service.characteristics.len(), 2);

        // Check Weight Measurement characteristic
        let measurement = &service.characteristics[0];
        assert_eq!(measurement.uuid, WEIGHT_MEASUREMENT_UUID);
        assert_eq!(measurement.properties, vec![PROPERTY_INDICATE]);
        assert!(measurement.default_value.is_none());

        // Check Weight Scale Feature characteristic
        let feature = &service.characteristics[1];
        assert_eq!(feature.uuid, WEIGHT_SCALE_FEATURE_UUID);
        assert_eq!(feature.properties, vec![PROPERTY_READ]);
        assert!(feature.default_value.is_some());
    }

    #[test]
    fn test_weight_scale_profile_multi_user_structure() {
        let profile = weight_scale_profile_multi_user();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, WEIGHT_SCALE_SERVICE_UUID);

        // Should have two characteristics
        assert_eq!(service.characteristics.len(), 2);

        // Verify the feature value includes multi-user support
        let feature = &service.characteristics[1];
        let expected_features: u32 = WeightScaleFeatureFlags::TimeStamp.as_u32()
            | WeightScaleFeatureFlags::MultipleUsers.as_u32()
            | WeightScaleFeatureFlags::BmiSupported.as_u32()
            | WeightScaleFeatureFlags::DisplayResolution01kg.as_u32();

        assert_eq!(
            feature.default_value,
            Some(expected_features.to_le_bytes().to_vec())
        );
    }

    #[test]
    fn test_weight_scale_feature_flags() {
        assert_eq!(WeightScaleFeatureFlags::TimeStamp.as_u32(), 0x0001);
        assert_eq!(WeightScaleFeatureFlags::MultipleUsers.as_u32(), 0x0002);
        assert_eq!(WeightScaleFeatureFlags::BmiSupported.as_u32(), 0x0004);
        assert_eq!(
            WeightScaleFeatureFlags::DisplayResolution05kg.as_u32(),
            0x0008
        );
        assert_eq!(
            WeightScaleFeatureFlags::DisplayResolution02kg.as_u32(),
            0x0010
        );
        assert_eq!(
            WeightScaleFeatureFlags::DisplayResolution01kg.as_u32(),
            0x0018
        );
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = weight_scale_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&WEIGHT_MEASUREMENT_UUID));
        assert!(uuids.contains(&WEIGHT_SCALE_FEATURE_UUID));
    }

    #[test]
    fn test_default_feature_value() {
        let profile = weight_scale_profile();
        let service = &profile.services[0];
        let feature_char = &service.characteristics[1];

        // Should have timestamp, BMI, and 0.1 kg resolution by default
        let expected: u32 = WeightScaleFeatureFlags::TimeStamp.as_u32()
            | WeightScaleFeatureFlags::BmiSupported.as_u32()
            | WeightScaleFeatureFlags::DisplayResolution01kg.as_u32();

        assert_eq!(
            feature_char.default_value,
            Some(expected.to_le_bytes().to_vec())
        );
    }
}
