// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Phone Alert Status Profile implementation.
//!
//! Based on Bluetooth SIG Phone Alert Status Service specification
//! (org.bluetooth.service.phone_alert_status).
//! Service UUID: 0x180E

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Phone Alert Status Service UUID (16-bit)
pub const PHONE_ALERT_STATUS_SERVICE_UUID: u16 = 0x180E;

/// Alert Status characteristic UUID (16-bit)
pub const ALERT_STATUS_UUID: u16 = 0x2A3F;

/// Ringer Setting characteristic UUID (16-bit)
pub const RINGER_SETTING_UUID: u16 = 0x2A41;

/// Ringer Control Point characteristic UUID (16-bit)
pub const RINGER_CONTROL_POINT_UUID: u16 = 0x2A40;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// BLE property for Write Without Response
const PROPERTY_WRITE_WITHOUT_RESPONSE: i32 = 16; // BleProperties::WriteWithoutResponse

/// Alert Status flags as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AlertStatusFlags {
    /// Ringer State: Active (bit 0)
    RingerActive = 0x01,
    /// Vibrator State: Active (bit 1)
    VibratorActive = 0x02,
    /// Display Alert Status: Active (bit 2)
    DisplayAlertActive = 0x04,
}

impl AlertStatusFlags {
    /// Convert to u8 value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Ringer Setting values as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum RingerSetting {
    /// Ringer Silent
    Silent = 0,
    /// Ringer Normal
    Normal = 1,
}

impl RingerSetting {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Ringer Control Point command values
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum RingerControlCommand {
    /// Silent Mode
    SilentMode = 1,
    /// Mute Once
    MuteOnce = 2,
    /// Cancel Silent Mode
    CancelSilentMode = 3,
}

impl RingerControlCommand {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Phone Alert Status Profile definition.
///
/// This profile includes:
/// - Phone Alert Status Service (0x180E)
///   - Alert Status (0x2A3F): Read, Notify (ringer, vibrator, display status)
///   - Ringer Setting (0x2A41): Read, Notify (silent or normal)
///   - Ringer Control Point (0x2A40): Write Without Response (control commands)
///
/// # Returns
/// A complete `ProfileDefinition` for the Phone Alert Status Profile.
pub fn phone_alert_status_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        PHONE_ALERT_STATUS_SERVICE_UUID,
        vec![
            // Alert Status - Read, Notify (indicates if ringer, vibrator, or display is active)
            CharacteristicDefinition::with_default_value(
                ALERT_STATUS_UUID,
                vec![PROPERTY_READ, PROPERTY_NOTIFY],
                vec![0x00], // Default: no alerts active
            ),
            // Ringer Setting - Read, Notify (indicates ringer state: silent or normal)
            CharacteristicDefinition::with_default_value(
                RINGER_SETTING_UUID,
                vec![PROPERTY_READ, PROPERTY_NOTIFY],
                vec![RingerSetting::Normal.as_u8()], // Default: normal ringer
            ),
            // Ringer Control Point - Write Without Response (commands to control ringer)
            CharacteristicDefinition::new(
                RINGER_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE_WITHOUT_RESPONSE],
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_alert_status_profile_structure() {
        let profile = phone_alert_status_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, PHONE_ALERT_STATUS_SERVICE_UUID);

        // Should have three characteristics
        assert_eq!(service.characteristics.len(), 3);

        // Check Alert Status characteristic
        let alert_status = &service.characteristics[0];
        assert_eq!(alert_status.uuid, ALERT_STATUS_UUID);
        assert_eq!(
            alert_status.properties,
            vec![PROPERTY_READ, PROPERTY_NOTIFY]
        );
        assert_eq!(alert_status.default_value, Some(vec![0x00]));

        // Check Ringer Setting characteristic
        let ringer_setting = &service.characteristics[1];
        assert_eq!(ringer_setting.uuid, RINGER_SETTING_UUID);
        assert_eq!(
            ringer_setting.properties,
            vec![PROPERTY_READ, PROPERTY_NOTIFY]
        );
        assert_eq!(
            ringer_setting.default_value,
            Some(vec![RingerSetting::Normal.as_u8()])
        );

        // Check Ringer Control Point characteristic
        let control_point = &service.characteristics[2];
        assert_eq!(control_point.uuid, RINGER_CONTROL_POINT_UUID);
        assert_eq!(
            control_point.properties,
            vec![PROPERTY_WRITE_WITHOUT_RESPONSE]
        );
        assert!(control_point.default_value.is_none());
    }

    #[test]
    fn test_alert_status_flags() {
        assert_eq!(AlertStatusFlags::RingerActive.as_u8(), 0x01);
        assert_eq!(AlertStatusFlags::VibratorActive.as_u8(), 0x02);
        assert_eq!(AlertStatusFlags::DisplayAlertActive.as_u8(), 0x04);
    }

    #[test]
    fn test_ringer_setting_values() {
        assert_eq!(RingerSetting::Silent.as_u8(), 0);
        assert_eq!(RingerSetting::Normal.as_u8(), 1);
    }

    #[test]
    fn test_ringer_control_command_values() {
        assert_eq!(RingerControlCommand::SilentMode.as_u8(), 1);
        assert_eq!(RingerControlCommand::MuteOnce.as_u8(), 2);
        assert_eq!(RingerControlCommand::CancelSilentMode.as_u8(), 3);
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = phone_alert_status_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&ALERT_STATUS_UUID));
        assert!(uuids.contains(&RINGER_SETTING_UUID));
        assert!(uuids.contains(&RINGER_CONTROL_POINT_UUID));
    }
}
