// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! User Data Profile implementation.
//!
//! Based on Bluetooth SIG User Data Service specification
//! (org.bluetooth.service.user_data).
//! Service UUID: 0x181C

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// User Data Service UUID (16-bit)
pub const USER_DATA_SERVICE_UUID: u16 = 0x181C;

/// First Name characteristic UUID (16-bit)
pub const FIRST_NAME_UUID: u16 = 0x2A8A;

/// Last Name characteristic UUID (16-bit)
pub const LAST_NAME_UUID: u16 = 0x2A90;

/// Email Address characteristic UUID (16-bit)
pub const EMAIL_ADDRESS_UUID: u16 = 0x2A87;

/// Age characteristic UUID (16-bit)
pub const AGE_UUID: u16 = 0x2A80;

/// Date of Birth characteristic UUID (16-bit)
pub const DATE_OF_BIRTH_UUID: u16 = 0x2A85;

/// Gender characteristic UUID (16-bit)
pub const GENDER_UUID: u16 = 0x2A8C;

/// Weight characteristic UUID (16-bit)
pub const WEIGHT_UUID: u16 = 0x2A98;

/// Height characteristic UUID (16-bit)
pub const HEIGHT_UUID: u16 = 0x2A8E;

/// VO2 Max characteristic UUID (16-bit)
pub const VO2_MAX_UUID: u16 = 0x2A96;

/// Heart Rate Max characteristic UUID (16-bit)
pub const HEART_RATE_MAX_UUID: u16 = 0x2A8D;

/// Resting Heart Rate characteristic UUID (16-bit)
pub const RESTING_HEART_RATE_UUID: u16 = 0x2A92;

/// Maximum Recommended Heart Rate characteristic UUID (16-bit)
pub const MAXIMUM_RECOMMENDED_HEART_RATE_UUID: u16 = 0x2A91;

/// Aerobic Threshold characteristic UUID (16-bit)
pub const AEROBIC_THRESHOLD_UUID: u16 = 0x2A7F;

/// Anaerobic Threshold characteristic UUID (16-bit)
pub const ANAEROBIC_THRESHOLD_UUID: u16 = 0x2A83;

/// Five Zone Heart Rate Limits characteristic UUID (16-bit)
pub const FIVE_ZONE_HEART_RATE_LIMITS_UUID: u16 = 0x2A8B;

/// Three Zone Heart Rate Limits characteristic UUID (16-bit)
pub const THREE_ZONE_HEART_RATE_LIMITS_UUID: u16 = 0x2A94;

/// Two Zone Heart Rate Limit characteristic UUID (16-bit)
pub const TWO_ZONE_HEART_RATE_LIMIT_UUID: u16 = 0x2A95;

/// Database Change Increment characteristic UUID (16-bit)
pub const DATABASE_CHANGE_INCREMENT_UUID: u16 = 0x2A99;

/// User Index characteristic UUID (16-bit)
pub const USER_INDEX_UUID: u16 = 0x2A9A;

/// User Control Point characteristic UUID (16-bit)
pub const USER_CONTROL_POINT_UUID: u16 = 0x2A9F;

/// Language characteristic UUID (16-bit)
pub const LANGUAGE_UUID: u16 = 0x2AA2;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2; // BleProperties::Write

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8; // BleProperties::Indicate

/// Gender values as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Gender {
    /// Male
    Male = 0,
    /// Female
    Female = 1,
    /// Unspecified
    Unspecified = 2,
}

impl Gender {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// User Control Point operation codes
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum UserControlPointOpCode {
    /// Register New User
    RegisterNewUser = 1,
    /// Consent
    Consent = 2,
    /// Delete User Data
    DeleteUserData = 3,
    /// List All Users (response only)
    ListAllUsers = 4,
    /// Delete Users (response only)
    DeleteUsers = 5,
    /// Response Code
    ResponseCode = 32,
}

impl UserControlPointOpCode {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the User Data Profile definition.
///
/// This profile includes:
/// - User Data Service (0x181C)
///   - Multiple user profile characteristics (name, age, gender, biometric data)
///   - Heart rate training zones
///   - User Control Point for multi-user management
///   - Database Change Increment for synchronization
///
/// # Returns
/// A complete `ProfileDefinition` for the User Data Profile.
pub fn user_data_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        USER_DATA_SERVICE_UUID,
        vec![
            // User identification
            CharacteristicDefinition::new(FIRST_NAME_UUID, vec![PROPERTY_READ, PROPERTY_WRITE]),
            CharacteristicDefinition::new(LAST_NAME_UUID, vec![PROPERTY_READ, PROPERTY_WRITE]),
            CharacteristicDefinition::new(
                EMAIL_ADDRESS_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
            ),
            // Demographic data
            CharacteristicDefinition::new(AGE_UUID, vec![PROPERTY_READ, PROPERTY_WRITE]),
            CharacteristicDefinition::new(
                DATE_OF_BIRTH_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
            ),
            CharacteristicDefinition::with_default_value(
                GENDER_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
                vec![Gender::Unspecified.as_u8()],
            ),
            // Physical characteristics
            CharacteristicDefinition::new(WEIGHT_UUID, vec![PROPERTY_READ, PROPERTY_WRITE]),
            CharacteristicDefinition::new(HEIGHT_UUID, vec![PROPERTY_READ, PROPERTY_WRITE]),
            // Fitness metrics
            CharacteristicDefinition::new(VO2_MAX_UUID, vec![PROPERTY_READ, PROPERTY_WRITE]),
            CharacteristicDefinition::new(
                HEART_RATE_MAX_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
            ),
            CharacteristicDefinition::new(
                RESTING_HEART_RATE_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
            ),
            CharacteristicDefinition::new(
                MAXIMUM_RECOMMENDED_HEART_RATE_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
            ),
            CharacteristicDefinition::new(
                AEROBIC_THRESHOLD_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
            ),
            CharacteristicDefinition::new(
                ANAEROBIC_THRESHOLD_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
            ),
            // Heart rate training zones
            CharacteristicDefinition::new(
                FIVE_ZONE_HEART_RATE_LIMITS_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
            ),
            CharacteristicDefinition::new(
                THREE_ZONE_HEART_RATE_LIMITS_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
            ),
            CharacteristicDefinition::new(
                TWO_ZONE_HEART_RATE_LIMIT_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
            ),
            // Multi-user support
            CharacteristicDefinition::with_default_value(
                DATABASE_CHANGE_INCREMENT_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE, PROPERTY_NOTIFY],
                vec![0, 0, 0, 0], // 32-bit counter, starts at 0
            ),
            CharacteristicDefinition::with_default_value(
                USER_INDEX_UUID,
                vec![PROPERTY_READ],
                vec![0xFF], // 0xFF = unknown user
            ),
            CharacteristicDefinition::new(
                USER_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
            // Localization
            CharacteristicDefinition::new(LANGUAGE_UUID, vec![PROPERTY_READ, PROPERTY_WRITE]),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_data_profile_structure() {
        let profile = user_data_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, USER_DATA_SERVICE_UUID);

        // Should have 21 characteristics
        assert_eq!(service.characteristics.len(), 21);
    }

    #[test]
    fn test_gender_values() {
        assert_eq!(Gender::Male.as_u8(), 0);
        assert_eq!(Gender::Female.as_u8(), 1);
        assert_eq!(Gender::Unspecified.as_u8(), 2);
    }

    #[test]
    fn test_user_control_point_op_codes() {
        assert_eq!(UserControlPointOpCode::RegisterNewUser.as_u8(), 1);
        assert_eq!(UserControlPointOpCode::Consent.as_u8(), 2);
        assert_eq!(UserControlPointOpCode::DeleteUserData.as_u8(), 3);
        assert_eq!(UserControlPointOpCode::ListAllUsers.as_u8(), 4);
        assert_eq!(UserControlPointOpCode::DeleteUsers.as_u8(), 5);
        assert_eq!(UserControlPointOpCode::ResponseCode.as_u8(), 32);
    }

    #[test]
    fn test_user_index_default() {
        let profile = user_data_profile();
        let service = &profile.services[0];

        // Find User Index characteristic
        let user_index = service
            .characteristics
            .iter()
            .find(|c| c.uuid == USER_INDEX_UUID)
            .unwrap();

        assert_eq!(user_index.default_value, Some(vec![0xFF]));
    }

    #[test]
    fn test_database_change_increment_default() {
        let profile = user_data_profile();
        let service = &profile.services[0];

        // Find Database Change Increment characteristic
        let db_change = service
            .characteristics
            .iter()
            .find(|c| c.uuid == DATABASE_CHANGE_INCREMENT_UUID)
            .unwrap();

        assert_eq!(db_change.default_value, Some(vec![0, 0, 0, 0]));
        assert!(db_change.properties.contains(&PROPERTY_NOTIFY));
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = user_data_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();

        // Check core characteristics
        assert!(uuids.contains(&FIRST_NAME_UUID));
        assert!(uuids.contains(&LAST_NAME_UUID));
        assert!(uuids.contains(&AGE_UUID));
        assert!(uuids.contains(&GENDER_UUID));
        assert!(uuids.contains(&WEIGHT_UUID));
        assert!(uuids.contains(&HEIGHT_UUID));
        assert!(uuids.contains(&USER_CONTROL_POINT_UUID));
        assert!(uuids.contains(&DATABASE_CHANGE_INCREMENT_UUID));
        assert!(uuids.contains(&USER_INDEX_UUID));
    }

    #[test]
    fn test_fitness_characteristics_present() {
        let profile = user_data_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();

        assert!(uuids.contains(&VO2_MAX_UUID));
        assert!(uuids.contains(&HEART_RATE_MAX_UUID));
        assert!(uuids.contains(&RESTING_HEART_RATE_UUID));
        assert!(uuids.contains(&AEROBIC_THRESHOLD_UUID));
        assert!(uuids.contains(&ANAEROBIC_THRESHOLD_UUID));
        assert!(uuids.contains(&FIVE_ZONE_HEART_RATE_LIMITS_UUID));
    }
}
