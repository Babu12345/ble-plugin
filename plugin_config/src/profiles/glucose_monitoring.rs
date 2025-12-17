// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Glucose Monitoring Profile implementation.
//!
//! Based on Bluetooth SIG Glucose Service specification
//! (org.bluetooth.service.glucose).
//! Service UUID: 0x1808

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Glucose Service UUID (16-bit)
pub const GLUCOSE_SERVICE_UUID: u16 = 0x1808;

/// Glucose Measurement characteristic UUID (16-bit)
pub const GLUCOSE_MEASUREMENT_UUID: u16 = 0x2A18;

/// Glucose Measurement Context characteristic UUID (16-bit)
pub const GLUCOSE_MEASUREMENT_CONTEXT_UUID: u16 = 0x2A34;

/// Glucose Feature characteristic UUID (16-bit)
pub const GLUCOSE_FEATURE_UUID: u16 = 0x2A51;

/// Record Access Control Point characteristic UUID (16-bit)
pub const RECORD_ACCESS_CONTROL_POINT_UUID: u16 = 0x2A52;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2; // BleProperties::Write

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8; // BleProperties::Indicate

/// Glucose Feature flags as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum GlucoseFeatureFlags {
    /// Low Battery Detection During Measurement Supported
    LowBatteryDetection = 0x0001,
    /// Sensor Malfunction Detection Supported
    SensorMalfunction = 0x0002,
    /// Sensor Sample Size Supported
    SampleSize = 0x0004,
    /// Sensor Strip Insertion Error Detection Supported
    StripInsertionError = 0x0008,
    /// Sensor Strip Type Error Detection Supported
    StripTypeError = 0x0010,
    /// Sensor Result High-Low Detection Supported
    ResultHighLowDetection = 0x0020,
    /// Sensor Temperature High-Low Detection Supported
    TemperatureHighLowDetection = 0x0040,
    /// Sensor Read Interrupt Detection Supported
    ReadInterruptDetection = 0x0080,
    /// General Device Fault Supported
    GeneralDeviceFault = 0x0100,
    /// Time Fault Supported
    TimeFault = 0x0200,
    /// Multiple Bond Supported
    MultipleBond = 0x0400,
}

impl GlucoseFeatureFlags {
    /// Convert to u16 value
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Creates the Glucose Monitoring Profile definition.
///
/// This profile includes:
/// - Glucose Service (0x1808)
///   - Glucose Measurement (0x2A18): Notify (glucose concentration data)
///   - Glucose Measurement Context (0x2A34): Notify (additional context info)
///   - Glucose Feature (0x2A51): Read (supported features bitmap)
///   - Record Access Control Point (0x2A52): Write, Indicate (database access)
///
/// # Returns
/// A complete `ProfileDefinition` for the Glucose Monitoring Profile.
pub fn glucose_monitoring_profile() -> ProfileDefinition {
    // Default feature set: basic measurement with low battery and malfunction detection
    let default_features: u16 = GlucoseFeatureFlags::LowBatteryDetection.as_u16()
        | GlucoseFeatureFlags::SensorMalfunction.as_u16();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        GLUCOSE_SERVICE_UUID,
        vec![
            // Glucose Measurement - Notify (glucose concentration, type, location, etc.)
            CharacteristicDefinition::new(GLUCOSE_MEASUREMENT_UUID, vec![PROPERTY_NOTIFY]),
            // Glucose Measurement Context - Notify (carbs, meal, exercise, medication, etc.)
            CharacteristicDefinition::new(
                GLUCOSE_MEASUREMENT_CONTEXT_UUID,
                vec![PROPERTY_NOTIFY],
            ),
            // Glucose Feature - Read (indicates supported features)
            CharacteristicDefinition::with_default_value(
                GLUCOSE_FEATURE_UUID,
                vec![PROPERTY_READ],
                default_features.to_le_bytes().to_vec(),
            ),
            // Record Access Control Point - Write, Indicate (transfer stored records)
            CharacteristicDefinition::new(
                RECORD_ACCESS_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glucose_monitoring_profile_structure() {
        let profile = glucose_monitoring_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, GLUCOSE_SERVICE_UUID);

        // Should have four characteristics
        assert_eq!(service.characteristics.len(), 4);

        // Check Glucose Measurement characteristic
        let measurement = &service.characteristics[0];
        assert_eq!(measurement.uuid, GLUCOSE_MEASUREMENT_UUID);
        assert_eq!(measurement.properties, vec![PROPERTY_NOTIFY]);
        assert!(measurement.default_value.is_none());

        // Check Glucose Measurement Context characteristic
        let context = &service.characteristics[1];
        assert_eq!(context.uuid, GLUCOSE_MEASUREMENT_CONTEXT_UUID);
        assert_eq!(context.properties, vec![PROPERTY_NOTIFY]);
        assert!(context.default_value.is_none());

        // Check Glucose Feature characteristic
        let feature = &service.characteristics[2];
        assert_eq!(feature.uuid, GLUCOSE_FEATURE_UUID);
        assert_eq!(feature.properties, vec![PROPERTY_READ]);
        assert!(feature.default_value.is_some());

        // Check Record Access Control Point characteristic
        let racp = &service.characteristics[3];
        assert_eq!(racp.uuid, RECORD_ACCESS_CONTROL_POINT_UUID);
        assert_eq!(racp.properties, vec![PROPERTY_WRITE, PROPERTY_INDICATE]);
        assert!(racp.default_value.is_none());
    }

    #[test]
    fn test_glucose_feature_flags() {
        assert_eq!(GlucoseFeatureFlags::LowBatteryDetection.as_u16(), 0x0001);
        assert_eq!(GlucoseFeatureFlags::SensorMalfunction.as_u16(), 0x0002);
        assert_eq!(GlucoseFeatureFlags::SampleSize.as_u16(), 0x0004);
        assert_eq!(GlucoseFeatureFlags::StripInsertionError.as_u16(), 0x0008);
        assert_eq!(GlucoseFeatureFlags::StripTypeError.as_u16(), 0x0010);
        assert_eq!(
            GlucoseFeatureFlags::ResultHighLowDetection.as_u16(),
            0x0020
        );
        assert_eq!(
            GlucoseFeatureFlags::TemperatureHighLowDetection.as_u16(),
            0x0040
        );
        assert_eq!(
            GlucoseFeatureFlags::ReadInterruptDetection.as_u16(),
            0x0080
        );
        assert_eq!(GlucoseFeatureFlags::GeneralDeviceFault.as_u16(), 0x0100);
        assert_eq!(GlucoseFeatureFlags::TimeFault.as_u16(), 0x0200);
        assert_eq!(GlucoseFeatureFlags::MultipleBond.as_u16(), 0x0400);
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = glucose_monitoring_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&GLUCOSE_MEASUREMENT_UUID));
        assert!(uuids.contains(&GLUCOSE_MEASUREMENT_CONTEXT_UUID));
        assert!(uuids.contains(&GLUCOSE_FEATURE_UUID));
        assert!(uuids.contains(&RECORD_ACCESS_CONTROL_POINT_UUID));
    }

    #[test]
    fn test_default_feature_value() {
        let profile = glucose_monitoring_profile();
        let service = &profile.services[0];
        let feature_char = &service.characteristics[2];

        // Should have low battery detection and sensor malfunction enabled
        let expected: u16 = GlucoseFeatureFlags::LowBatteryDetection.as_u16()
            | GlucoseFeatureFlags::SensorMalfunction.as_u16();

        assert_eq!(
            feature_char.default_value,
            Some(expected.to_le_bytes().to_vec())
        );
    }
}
