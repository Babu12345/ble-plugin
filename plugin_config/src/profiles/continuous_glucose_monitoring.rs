// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Continuous Glucose Monitoring (CGM) Profile implementation.
//!
//! Based on Bluetooth SIG Continuous Glucose Monitoring Service specification
//! (org.bluetooth.service.continuous_glucose_monitoring).
//! Service UUID: 0x181F

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Continuous Glucose Monitoring Service UUID (16-bit)
pub const CONTINUOUS_GLUCOSE_MONITORING_SERVICE_UUID: u16 = 0x181F;

/// CGM Measurement characteristic UUID (16-bit)
pub const CGM_MEASUREMENT_UUID: u16 = 0x2AA7;

/// CGM Feature characteristic UUID (16-bit)
pub const CGM_FEATURE_UUID: u16 = 0x2AA8;

/// CGM Status characteristic UUID (16-bit)
pub const CGM_STATUS_UUID: u16 = 0x2AA9;

/// CGM Session Start Time characteristic UUID (16-bit)
pub const CGM_SESSION_START_TIME_UUID: u16 = 0x2AAA;

/// CGM Session Run Time characteristic UUID (16-bit)
pub const CGM_SESSION_RUN_TIME_UUID: u16 = 0x2AAB;

/// Record Access Control Point characteristic UUID (16-bit)
pub const RECORD_ACCESS_CONTROL_POINT_UUID: u16 = 0x2A52;

/// CGM Specific Ops Control Point characteristic UUID (16-bit)
pub const CGM_SPECIFIC_OPS_CONTROL_POINT_UUID: u16 = 0x2AAC;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2; // BleProperties::Write

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8; // BleProperties::Indicate

/// CGM Feature flags as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum CgmFeature {
    /// Calibration Supported (bit 0)
    CalibrationSupported = 0x00000001,
    /// Patient High/Low Alerts Supported (bit 1)
    PatientHighLowAlertsSupported = 0x00000002,
    /// Hypo Alerts Supported (bit 2)
    HypoAlertsSupported = 0x00000004,
    /// Hyper Alerts Supported (bit 3)
    HyperAlertsSupported = 0x00000008,
    /// Rate of Increase/Decrease Alerts Supported (bit 4)
    RateOfIncreaseDecreaseAlertsSupported = 0x00000010,
    /// Device Specific Alert Supported (bit 5)
    DeviceSpecificAlertSupported = 0x00000020,
    /// Sensor Malfunction Detection Supported (bit 6)
    SensorMalfunctionDetectionSupported = 0x00000040,
    /// Sensor Temperature High-Low Detection Supported (bit 7)
    SensorTemperatureHighLowDetectionSupported = 0x00000080,
    /// Sensor Result High-Low Detection Supported (bit 8)
    SensorResultHighLowDetectionSupported = 0x00000100,
    /// Low Battery Detection Supported (bit 9)
    LowBatteryDetectionSupported = 0x00000200,
    /// Sensor Type Error Detection Supported (bit 10)
    SensorTypeErrorDetectionSupported = 0x00000400,
    /// General Device Fault Supported (bit 11)
    GeneralDeviceFaultSupported = 0x00000800,
    /// E2E-CRC Supported (bit 12)
    E2ECrcSupported = 0x00001000,
    /// Multiple Bond Supported (bit 13)
    MultipleBondSupported = 0x00002000,
    /// Multiple Sessions Supported (bit 14)
    MultipleSessionsSupported = 0x00004000,
    /// CGM Trend Information Supported (bit 15)
    CgmTrendInformationSupported = 0x00008000,
    /// CGM Quality Supported (bit 16)
    CgmQualitySupported = 0x00010000,
}

impl CgmFeature {
    /// Convert to u32 value
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// CGM Type values
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CgmType {
    /// Capillary Whole blood
    CapillaryWholeBlood = 1,
    /// Capillary Plasma
    CapillaryPlasma = 2,
    /// Venous Whole blood
    VenousWholeBlood = 3,
    /// Venous Plasma
    VenousPlasma = 4,
    /// Arterial Whole blood
    ArterialWholeBlood = 5,
    /// Arterial Plasma
    ArterialPlasma = 6,
    /// Undetermined Whole blood
    UndeterminedWholeBlood = 7,
    /// Undetermined Plasma
    UndeterminedPlasma = 8,
    /// Interstitial Fluid (ISF)
    InterstitialFluid = 9,
    /// Control Solution
    ControlSolution = 10,
}

impl CgmType {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Continuous Glucose Monitoring Profile definition.
///
/// This profile includes:
/// - Continuous Glucose Monitoring Service (0x181F)
///   - CGM Measurement (0x2AA7): Notify (glucose concentration, trend, quality)
///   - CGM Feature (0x2AA8): Read (supported features)
///   - CGM Status (0x2AA9): Read (sensor status, warnings, calibration)
///   - CGM Session Start Time (0x2AAA): Read, Write (session timing)
///   - CGM Session Run Time (0x2AAB): Read (session duration)
///   - Record Access Control Point (0x2A52): Write, Indicate (stored data access)
///   - CGM Specific Ops Control Point (0x2AAC): Write, Indicate (CGM operations)
///
/// # Returns
/// A complete `ProfileDefinition` for the Continuous Glucose Monitoring Profile.
pub fn continuous_glucose_monitoring_profile() -> ProfileDefinition {
    // Default features: trend, quality, alerts, multiple sessions
    let default_features = CgmFeature::CgmTrendInformationSupported.as_u32()
        | CgmFeature::CgmQualitySupported.as_u32()
        | CgmFeature::PatientHighLowAlertsSupported.as_u32()
        | CgmFeature::HypoAlertsSupported.as_u32()
        | CgmFeature::HyperAlertsSupported.as_u32()
        | CgmFeature::MultipleSessionsSupported.as_u32();

    // Default type: Interstitial Fluid (most common for CGM)
    let default_type = vec![
        CgmType::InterstitialFluid.as_u8(),
        0x00, // Sample Location: Not specified
    ];

    ProfileDefinition::new(vec![ServiceDefinition::new(
        CONTINUOUS_GLUCOSE_MONITORING_SERVICE_UUID,
        vec![
            // CGM Measurement - Notify (real-time glucose readings with trend and quality)
            CharacteristicDefinition::new(CGM_MEASUREMENT_UUID, vec![PROPERTY_NOTIFY]),
            // CGM Feature - Read (feature flags and CGM type/sample location)
            CharacteristicDefinition::with_default_value(
                CGM_FEATURE_UUID,
                vec![PROPERTY_READ],
                {
                    let mut bytes = default_features.to_le_bytes().to_vec();
                    bytes.extend_from_slice(&default_type);
                    bytes
                },
            ),
            // CGM Status - Read (sensor status and calibration info)
            CharacteristicDefinition::new(CGM_STATUS_UUID, vec![PROPERTY_READ]),
            // CGM Session Start Time - Read, Write (when CGM session started)
            CharacteristicDefinition::new(
                CGM_SESSION_START_TIME_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
            ),
            // CGM Session Run Time - Read (how long session has been running)
            CharacteristicDefinition::new(CGM_SESSION_RUN_TIME_UUID, vec![PROPERTY_READ]),
            // Record Access Control Point - Write, Indicate (access historical data)
            CharacteristicDefinition::new(
                RECORD_ACCESS_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
            // CGM Specific Ops Control Point - Write, Indicate (CGM-specific commands)
            CharacteristicDefinition::new(
                CGM_SPECIFIC_OPS_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continuous_glucose_monitoring_profile_structure() {
        let profile = continuous_glucose_monitoring_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, CONTINUOUS_GLUCOSE_MONITORING_SERVICE_UUID);

        // Should have seven characteristics
        assert_eq!(service.characteristics.len(), 7);

        // Check CGM Measurement characteristic
        let measurement = &service.characteristics[0];
        assert_eq!(measurement.uuid, CGM_MEASUREMENT_UUID);
        assert_eq!(measurement.properties, vec![PROPERTY_NOTIFY]);

        // Check CGM Feature characteristic
        let feature = &service.characteristics[1];
        assert_eq!(feature.uuid, CGM_FEATURE_UUID);
        assert_eq!(feature.properties, vec![PROPERTY_READ]);
        assert!(feature.default_value.is_some());

        // Check CGM Status characteristic
        let status = &service.characteristics[2];
        assert_eq!(status.uuid, CGM_STATUS_UUID);
        assert_eq!(status.properties, vec![PROPERTY_READ]);
    }

    #[test]
    fn test_cgm_feature_values() {
        assert_eq!(CgmFeature::CalibrationSupported.as_u32(), 0x00000001);
        assert_eq!(
            CgmFeature::PatientHighLowAlertsSupported.as_u32(),
            0x00000002
        );
        assert_eq!(CgmFeature::HypoAlertsSupported.as_u32(), 0x00000004);
        assert_eq!(CgmFeature::HyperAlertsSupported.as_u32(), 0x00000008);
        assert_eq!(
            CgmFeature::CgmTrendInformationSupported.as_u32(),
            0x00008000
        );
        assert_eq!(CgmFeature::CgmQualitySupported.as_u32(), 0x00010000);
    }

    #[test]
    fn test_cgm_type_values() {
        assert_eq!(CgmType::CapillaryWholeBlood.as_u8(), 1);
        assert_eq!(CgmType::VenousPlasma.as_u8(), 4);
        assert_eq!(CgmType::InterstitialFluid.as_u8(), 9);
        assert_eq!(CgmType::ControlSolution.as_u8(), 10);
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = continuous_glucose_monitoring_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&CGM_MEASUREMENT_UUID));
        assert!(uuids.contains(&CGM_FEATURE_UUID));
        assert!(uuids.contains(&CGM_STATUS_UUID));
        assert!(uuids.contains(&CGM_SESSION_START_TIME_UUID));
        assert!(uuids.contains(&CGM_SESSION_RUN_TIME_UUID));
        assert!(uuids.contains(&RECORD_ACCESS_CONTROL_POINT_UUID));
        assert!(uuids.contains(&CGM_SPECIFIC_OPS_CONTROL_POINT_UUID));
    }
}
