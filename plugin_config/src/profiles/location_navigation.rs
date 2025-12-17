// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Location and Navigation Profile implementation.
//!
//! Based on Bluetooth SIG Location and Navigation Service specification
//! (org.bluetooth.service.location_and_navigation).
//! Service UUID: 0x1819

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Location and Navigation Service UUID (16-bit)
pub const LOCATION_NAVIGATION_SERVICE_UUID: u16 = 0x1819;

/// LN Feature characteristic UUID (16-bit)
pub const LN_FEATURE_UUID: u16 = 0x2A6A;

/// Location and Speed characteristic UUID (16-bit)
pub const LOCATION_AND_SPEED_UUID: u16 = 0x2A67;

/// Position Quality characteristic UUID (16-bit)
pub const POSITION_QUALITY_UUID: u16 = 0x2A69;

/// LN Control Point characteristic UUID (16-bit)
pub const LN_CONTROL_POINT_UUID: u16 = 0x2A6B;

/// Navigation characteristic UUID (16-bit)
pub const NAVIGATION_UUID: u16 = 0x2A68;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2; // BleProperties::Write

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8; // BleProperties::Indicate

/// LN Feature flags as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum LnFeature {
    /// Instantaneous Speed Supported (bit 0)
    InstantaneousSpeedSupported = 0x00000001,
    /// Total Distance Supported (bit 1)
    TotalDistanceSupported = 0x00000002,
    /// Location Supported (bit 2)
    LocationSupported = 0x00000004,
    /// Elevation Supported (bit 3)
    ElevationSupported = 0x00000008,
    /// Heading Supported (bit 4)
    HeadingSupported = 0x00000010,
    /// Rolling Time Supported (bit 5)
    RollingTimeSupported = 0x00000020,
    /// UTC Time Supported (bit 6)
    UtcTimeSupported = 0x00000040,
    /// Remaining Distance Supported (bit 7)
    RemainingDistanceSupported = 0x00000080,
    /// Remaining Vertical Distance Supported (bit 8)
    RemainingVerticalDistanceSupported = 0x00000100,
    /// Estimated Time of Arrival Supported (bit 9)
    EstimatedTimeOfArrivalSupported = 0x00000200,
    /// Number of Beacons in Solution Supported (bit 10)
    NumberOfBeaconsInSolutionSupported = 0x00000400,
    /// Number of Beacons in View Supported (bit 11)
    NumberOfBeaconsInViewSupported = 0x00000800,
    /// Time to First Fix Supported (bit 12)
    TimeToFirstFixSupported = 0x00001000,
    /// Estimated Horizontal Position Error Supported (bit 13)
    EstimatedHorizontalPositionErrorSupported = 0x00002000,
    /// Estimated Vertical Position Error Supported (bit 14)
    EstimatedVerticalPositionErrorSupported = 0x00004000,
    /// Horizontal Dilution of Precision Supported (bit 15)
    HorizontalDilutionOfPrecisionSupported = 0x00008000,
    /// Vertical Dilution of Precision Supported (bit 16)
    VerticalDilutionOfPrecisionSupported = 0x00010000,
    /// Location and Speed Characteristic Content Masking Supported (bit 17)
    LocationAndSpeedContentMaskingSupported = 0x00020000,
    /// Fix Rate Setting Supported (bit 18)
    FixRateSettingSupported = 0x00040000,
    /// Elevation Setting Supported (bit 19)
    ElevationSettingSupported = 0x00080000,
    /// Position Status Supported (bit 20)
    PositionStatusSupported = 0x00100000,
}

impl LnFeature {
    /// Convert to u32 value
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Position Status values
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PositionStatus {
    /// No Position
    NoPosition = 0,
    /// Position OK
    PositionOk = 1,
    /// Estimated Position
    EstimatedPosition = 2,
    /// Last Known Position
    LastKnownPosition = 3,
}

impl PositionStatus {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Location and Navigation Profile definition.
///
/// This profile includes:
/// - Location and Navigation Service (0x1819)
///   - LN Feature (0x2A6A): Read (supported features)
///   - Location and Speed (0x2A67): Notify (position, speed, heading)
///   - Position Quality (0x2A69): Read (GPS quality indicators)
///   - LN Control Point (0x2A6B): Write, Indicate (commands and responses)
///   - Navigation (0x2A68): Notify (navigation instructions)
///
/// # Returns
/// A complete `ProfileDefinition` for the Location and Navigation Profile.
pub fn location_navigation_profile() -> ProfileDefinition {
    // Default features: location, speed, elevation, heading, UTC time
    let default_features = LnFeature::LocationSupported.as_u32()
        | LnFeature::InstantaneousSpeedSupported.as_u32()
        | LnFeature::ElevationSupported.as_u32()
        | LnFeature::HeadingSupported.as_u32()
        | LnFeature::UtcTimeSupported.as_u32()
        | LnFeature::TotalDistanceSupported.as_u32();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        LOCATION_NAVIGATION_SERVICE_UUID,
        vec![
            // LN Feature - Read (supported features bitmask)
            CharacteristicDefinition::with_default_value(
                LN_FEATURE_UUID,
                vec![PROPERTY_READ],
                default_features.to_le_bytes().to_vec(),
            ),
            // Location and Speed - Notify (real-time position and speed data)
            CharacteristicDefinition::new(LOCATION_AND_SPEED_UUID, vec![PROPERTY_NOTIFY]),
            // Position Quality - Read (GPS signal quality, HDOP, satellites)
            CharacteristicDefinition::new(POSITION_QUALITY_UUID, vec![PROPERTY_READ]),
            // LN Control Point - Write, Indicate (route selection, navigation control)
            CharacteristicDefinition::new(
                LN_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
            // Navigation - Notify (turn-by-turn directions, waypoints)
            CharacteristicDefinition::new(NAVIGATION_UUID, vec![PROPERTY_NOTIFY]),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_navigation_profile_structure() {
        let profile = location_navigation_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, LOCATION_NAVIGATION_SERVICE_UUID);

        // Should have five characteristics
        assert_eq!(service.characteristics.len(), 5);

        // Check LN Feature characteristic
        let feature = &service.characteristics[0];
        assert_eq!(feature.uuid, LN_FEATURE_UUID);
        assert_eq!(feature.properties, vec![PROPERTY_READ]);
        assert!(feature.default_value.is_some());
        assert_eq!(feature.default_value.as_ref().unwrap().len(), 4); // u32 = 4 bytes

        // Check Location and Speed characteristic
        let location_speed = &service.characteristics[1];
        assert_eq!(location_speed.uuid, LOCATION_AND_SPEED_UUID);
        assert_eq!(location_speed.properties, vec![PROPERTY_NOTIFY]);
        assert!(location_speed.default_value.is_none());

        // Check Position Quality characteristic
        let position_quality = &service.characteristics[2];
        assert_eq!(position_quality.uuid, POSITION_QUALITY_UUID);
        assert_eq!(position_quality.properties, vec![PROPERTY_READ]);
        assert!(position_quality.default_value.is_none());

        // Check LN Control Point characteristic
        let control_point = &service.characteristics[3];
        assert_eq!(control_point.uuid, LN_CONTROL_POINT_UUID);
        assert_eq!(
            control_point.properties,
            vec![PROPERTY_WRITE, PROPERTY_INDICATE]
        );
        assert!(control_point.default_value.is_none());

        // Check Navigation characteristic
        let navigation = &service.characteristics[4];
        assert_eq!(navigation.uuid, NAVIGATION_UUID);
        assert_eq!(navigation.properties, vec![PROPERTY_NOTIFY]);
        assert!(navigation.default_value.is_none());
    }

    #[test]
    fn test_ln_feature_values() {
        assert_eq!(
            LnFeature::InstantaneousSpeedSupported.as_u32(),
            0x00000001
        );
        assert_eq!(LnFeature::TotalDistanceSupported.as_u32(), 0x00000002);
        assert_eq!(LnFeature::LocationSupported.as_u32(), 0x00000004);
        assert_eq!(LnFeature::ElevationSupported.as_u32(), 0x00000008);
        assert_eq!(LnFeature::HeadingSupported.as_u32(), 0x00000010);
        assert_eq!(LnFeature::UtcTimeSupported.as_u32(), 0x00000040);
        assert_eq!(LnFeature::FixRateSettingSupported.as_u32(), 0x00040000);
    }

    #[test]
    fn test_position_status_values() {
        assert_eq!(PositionStatus::NoPosition.as_u8(), 0);
        assert_eq!(PositionStatus::PositionOk.as_u8(), 1);
        assert_eq!(PositionStatus::EstimatedPosition.as_u8(), 2);
        assert_eq!(PositionStatus::LastKnownPosition.as_u8(), 3);
    }

    #[test]
    fn test_default_feature_value() {
        let profile = location_navigation_profile();
        let service = &profile.services[0];
        let feature = &service.characteristics[0];

        let default_features = LnFeature::LocationSupported.as_u32()
            | LnFeature::InstantaneousSpeedSupported.as_u32()
            | LnFeature::ElevationSupported.as_u32()
            | LnFeature::HeadingSupported.as_u32()
            | LnFeature::UtcTimeSupported.as_u32()
            | LnFeature::TotalDistanceSupported.as_u32();

        assert_eq!(
            feature.default_value,
            Some(default_features.to_le_bytes().to_vec())
        );
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = location_navigation_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&LN_FEATURE_UUID));
        assert!(uuids.contains(&LOCATION_AND_SPEED_UUID));
        assert!(uuids.contains(&POSITION_QUALITY_UUID));
        assert!(uuids.contains(&LN_CONTROL_POINT_UUID));
        assert!(uuids.contains(&NAVIGATION_UUID));
    }
}
