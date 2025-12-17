// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Pulse Oximeter Profile implementation.
//!
//! Based on Bluetooth SIG Pulse Oximeter Service specification
//! (org.bluetooth.service.pulse_oximeter).
//! Service UUID: 0x1822

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Pulse Oximeter Service UUID (16-bit)
pub const PULSE_OXIMETER_SERVICE_UUID: u16 = 0x1822;

/// PLX Spot-Check Measurement characteristic UUID (16-bit)
pub const PLX_SPOT_CHECK_MEASUREMENT_UUID: u16 = 0x2A5E;

/// PLX Continuous Measurement characteristic UUID (16-bit)
pub const PLX_CONTINUOUS_MEASUREMENT_UUID: u16 = 0x2A5F;

/// PLX Features characteristic UUID (16-bit)
pub const PLX_FEATURES_UUID: u16 = 0x2A60;

/// Record Access Control Point characteristic UUID (16-bit)
pub const RECORD_ACCESS_CONTROL_POINT_UUID: u16 = 0x2A52;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2; // BleProperties::Write

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8; // BleProperties::Indicate

/// PLX Feature flags as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum PlxFeature {
    /// Measurement Status Support (bit 0)
    MeasurementStatusSupport = 0x0001,
    /// Device and Sensor Status Support (bit 1)
    DeviceAndSensorStatusSupport = 0x0002,
    /// Measurement Storage for Spot-check measurements (bit 2)
    SpotCheckMeasurementStorageSupport = 0x0004,
    /// Timestamp for Spot-check measurements (bit 3)
    TimestampSupport = 0x0008,
    /// SpO2PR-Spot-check (bit 4)
    SpO2PrSpotCheckSupport = 0x0010,
    /// SpO2PR-Continuous (bit 5)
    SpO2PrContinuousSupport = 0x0020,
    /// Pulse Amplitude Index (bit 6)
    PulseAmplitudeIndexSupport = 0x0040,
    /// Multiple Bonds Supported (bit 7)
    MultipleBondsSupport = 0x0080,
}

impl PlxFeature {
    /// Convert to u16 value
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Measurement status values
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum MeasurementStatus {
    /// Measurement ongoing (bit 5)
    MeasurementOngoing = 0x0020,
    /// Early Estimated Data (bit 6)
    EarlyEstimatedData = 0x0040,
    /// Validated Data (bit 7)
    ValidatedData = 0x0080,
    /// Fully Qualified Data (bit 8)
    FullyQualifiedData = 0x0100,
    /// Data from Measurement Storage (bit 9)
    DataFromMeasurementStorage = 0x0200,
    /// Data for Demonstration (bit 10)
    DataForDemonstration = 0x0400,
    /// Data for Testing (bit 11)
    DataForTesting = 0x0800,
    /// Calibration Ongoing (bit 12)
    CalibrationOngoing = 0x1000,
    /// Measurement Unavailable (bit 13)
    MeasurementUnavailable = 0x2000,
    /// Questionable Measurement Detected (bit 14)
    QuestionableMeasurementDetected = 0x4000,
    /// Invalid Measurement Detected (bit 15)
    InvalidMeasurementDetected = 0x8000,
}

impl MeasurementStatus {
    /// Convert to u16 value
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Creates the Pulse Oximeter Profile definition.
///
/// This profile includes:
/// - Pulse Oximeter Service (0x1822)
///   - PLX Spot-Check Measurement (0x2A5E): Indicate (SpO2, pulse rate, timestamp)
///   - PLX Continuous Measurement (0x2A5F): Notify (real-time SpO2, pulse rate)
///   - PLX Features (0x2A60): Read (supported features)
///   - Record Access Control Point (0x2A52): Write, Indicate (stored data access)
///
/// # Returns
/// A complete `ProfileDefinition` for the Pulse Oximeter Profile.
pub fn pulse_oximeter_profile() -> ProfileDefinition {
    // Default features: continuous measurement, spot-check, measurement status
    let default_features = PlxFeature::SpO2PrContinuousSupport.as_u16()
        | PlxFeature::SpO2PrSpotCheckSupport.as_u16()
        | PlxFeature::MeasurementStatusSupport.as_u16()
        | PlxFeature::PulseAmplitudeIndexSupport.as_u16();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        PULSE_OXIMETER_SERVICE_UUID,
        vec![
            // PLX Spot-Check Measurement - Indicate (on-demand measurements)
            CharacteristicDefinition::new(
                PLX_SPOT_CHECK_MEASUREMENT_UUID,
                vec![PROPERTY_INDICATE],
            ),
            // PLX Continuous Measurement - Notify (real-time streaming data)
            CharacteristicDefinition::new(PLX_CONTINUOUS_MEASUREMENT_UUID, vec![PROPERTY_NOTIFY]),
            // PLX Features - Read (supported features bitmask)
            CharacteristicDefinition::with_default_value(
                PLX_FEATURES_UUID,
                vec![PROPERTY_READ],
                default_features.to_le_bytes().to_vec(),
            ),
            // Record Access Control Point - Write, Indicate (access stored measurements)
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
    fn test_pulse_oximeter_profile_structure() {
        let profile = pulse_oximeter_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, PULSE_OXIMETER_SERVICE_UUID);

        // Should have four characteristics
        assert_eq!(service.characteristics.len(), 4);

        // Check PLX Spot-Check Measurement characteristic
        let spot_check = &service.characteristics[0];
        assert_eq!(spot_check.uuid, PLX_SPOT_CHECK_MEASUREMENT_UUID);
        assert_eq!(spot_check.properties, vec![PROPERTY_INDICATE]);
        assert!(spot_check.default_value.is_none());

        // Check PLX Continuous Measurement characteristic
        let continuous = &service.characteristics[1];
        assert_eq!(continuous.uuid, PLX_CONTINUOUS_MEASUREMENT_UUID);
        assert_eq!(continuous.properties, vec![PROPERTY_NOTIFY]);
        assert!(continuous.default_value.is_none());

        // Check PLX Features characteristic
        let features = &service.characteristics[2];
        assert_eq!(features.uuid, PLX_FEATURES_UUID);
        assert_eq!(features.properties, vec![PROPERTY_READ]);
        assert!(features.default_value.is_some());

        // Check Record Access Control Point characteristic
        let racp = &service.characteristics[3];
        assert_eq!(racp.uuid, RECORD_ACCESS_CONTROL_POINT_UUID);
        assert_eq!(racp.properties, vec![PROPERTY_WRITE, PROPERTY_INDICATE]);
        assert!(racp.default_value.is_none());
    }

    #[test]
    fn test_plx_feature_values() {
        assert_eq!(PlxFeature::MeasurementStatusSupport.as_u16(), 0x0001);
        assert_eq!(PlxFeature::DeviceAndSensorStatusSupport.as_u16(), 0x0002);
        assert_eq!(
            PlxFeature::SpotCheckMeasurementStorageSupport.as_u16(),
            0x0004
        );
        assert_eq!(PlxFeature::TimestampSupport.as_u16(), 0x0008);
        assert_eq!(PlxFeature::SpO2PrSpotCheckSupport.as_u16(), 0x0010);
        assert_eq!(PlxFeature::SpO2PrContinuousSupport.as_u16(), 0x0020);
        assert_eq!(PlxFeature::PulseAmplitudeIndexSupport.as_u16(), 0x0040);
        assert_eq!(PlxFeature::MultipleBondsSupport.as_u16(), 0x0080);
    }

    #[test]
    fn test_measurement_status_values() {
        assert_eq!(MeasurementStatus::MeasurementOngoing.as_u16(), 0x0020);
        assert_eq!(MeasurementStatus::EarlyEstimatedData.as_u16(), 0x0040);
        assert_eq!(MeasurementStatus::ValidatedData.as_u16(), 0x0080);
        assert_eq!(MeasurementStatus::FullyQualifiedData.as_u16(), 0x0100);
        assert_eq!(
            MeasurementStatus::InvalidMeasurementDetected.as_u16(),
            0x8000
        );
    }

    #[test]
    fn test_default_feature_value() {
        let profile = pulse_oximeter_profile();
        let service = &profile.services[0];
        let features = &service.characteristics[2];

        let default_features = PlxFeature::SpO2PrContinuousSupport.as_u16()
            | PlxFeature::SpO2PrSpotCheckSupport.as_u16()
            | PlxFeature::MeasurementStatusSupport.as_u16()
            | PlxFeature::PulseAmplitudeIndexSupport.as_u16();

        assert_eq!(
            features.default_value,
            Some(default_features.to_le_bytes().to_vec())
        );
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = pulse_oximeter_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&PLX_SPOT_CHECK_MEASUREMENT_UUID));
        assert!(uuids.contains(&PLX_CONTINUOUS_MEASUREMENT_UUID));
        assert!(uuids.contains(&PLX_FEATURES_UUID));
        assert!(uuids.contains(&RECORD_ACCESS_CONTROL_POINT_UUID));
    }
}
