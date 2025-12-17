// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! HID over GATT Profile (HoGP) implementation.
//!
//! Based on Bluetooth SIG HID over GATT Profile specification
//! (org.bluetooth.profile.hogp).
//! Service UUID: 0x1812

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// HID Service UUID (16-bit)
pub const HID_SERVICE_UUID: u16 = 0x1812;

/// HID Information characteristic UUID (16-bit)
pub const HID_INFORMATION_UUID: u16 = 0x2A4A;

/// Report Map characteristic UUID (16-bit)
pub const REPORT_MAP_UUID: u16 = 0x2A4B;

/// HID Control Point characteristic UUID (16-bit)
pub const HID_CONTROL_POINT_UUID: u16 = 0x2A4C;

/// Report characteristic UUID (16-bit)
pub const REPORT_UUID: u16 = 0x2A4D;

/// Protocol Mode characteristic UUID (16-bit)
pub const PROTOCOL_MODE_UUID: u16 = 0x2A4E;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2; // BleProperties::Write

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// BLE property for Write Without Response
const PROPERTY_WRITE_WITHOUT_RESPONSE: i32 = 16; // BleProperties::WriteWithoutResponse

/// HID Protocol Mode values as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ProtocolMode {
    /// Boot Protocol Mode
    Boot = 0,
    /// Report Protocol Mode
    Report = 1,
}

impl ProtocolMode {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the HID over GATT Profile definition.
///
/// This profile includes:
/// - HID Service (0x1812)
///   - HID Information (0x2A4A): Read (device info like country code, flags)
///   - Report Map (0x2A4B): Read (HID report descriptor)
///   - HID Control Point (0x2A4C): Write Without Response (suspend/resume)
///   - Report (0x2A4D): Read, Notify, Write (input/output/feature reports)
///   - Protocol Mode (0x2A4E): Read, Write Without Response (boot/report mode)
///
/// # Returns
/// A complete `ProfileDefinition` for the HID over GATT Profile.
pub fn hid_over_gatt_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        HID_SERVICE_UUID,
        vec![
            // HID Information - Read (contains bcdHID, bCountryCode, flags)
            CharacteristicDefinition::with_default_value(
                HID_INFORMATION_UUID,
                vec![PROPERTY_READ],
                vec![0x11, 0x01, 0x00, 0x03], // bcdHID=1.11, bCountryCode=0, Flags=3
            ),
            // Report Map - Read (HID report descriptor defining device capabilities)
            CharacteristicDefinition::new(REPORT_MAP_UUID, vec![PROPERTY_READ]),
            // HID Control Point - Write Without Response (suspend/resume commands)
            CharacteristicDefinition::new(
                HID_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE_WITHOUT_RESPONSE],
            ),
            // Report - Read, Notify, Write (HID reports for input/output data)
            CharacteristicDefinition::new(
                REPORT_UUID,
                vec![PROPERTY_READ, PROPERTY_NOTIFY, PROPERTY_WRITE],
            ),
            // Protocol Mode - Read, Write Without Response (boot vs report protocol)
            CharacteristicDefinition::with_default_value(
                PROTOCOL_MODE_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE_WITHOUT_RESPONSE],
                vec![ProtocolMode::Report.as_u8()], // Default to Report Protocol Mode
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hid_over_gatt_profile_structure() {
        let profile = hid_over_gatt_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, HID_SERVICE_UUID);

        // Should have five characteristics
        assert_eq!(service.characteristics.len(), 5);

        // Check HID Information characteristic
        let hid_info = &service.characteristics[0];
        assert_eq!(hid_info.uuid, HID_INFORMATION_UUID);
        assert_eq!(hid_info.properties, vec![PROPERTY_READ]);
        assert_eq!(
            hid_info.default_value,
            Some(vec![0x11, 0x01, 0x00, 0x03])
        );

        // Check Report Map characteristic
        let report_map = &service.characteristics[1];
        assert_eq!(report_map.uuid, REPORT_MAP_UUID);
        assert_eq!(report_map.properties, vec![PROPERTY_READ]);
        assert!(report_map.default_value.is_none());

        // Check HID Control Point characteristic
        let control_point = &service.characteristics[2];
        assert_eq!(control_point.uuid, HID_CONTROL_POINT_UUID);
        assert_eq!(
            control_point.properties,
            vec![PROPERTY_WRITE_WITHOUT_RESPONSE]
        );
        assert!(control_point.default_value.is_none());

        // Check Report characteristic
        let report = &service.characteristics[3];
        assert_eq!(report.uuid, REPORT_UUID);
        assert_eq!(
            report.properties,
            vec![PROPERTY_READ, PROPERTY_NOTIFY, PROPERTY_WRITE]
        );
        assert!(report.default_value.is_none());

        // Check Protocol Mode characteristic
        let protocol_mode = &service.characteristics[4];
        assert_eq!(protocol_mode.uuid, PROTOCOL_MODE_UUID);
        assert_eq!(
            protocol_mode.properties,
            vec![PROPERTY_READ, PROPERTY_WRITE_WITHOUT_RESPONSE]
        );
        assert_eq!(
            protocol_mode.default_value,
            Some(vec![ProtocolMode::Report.as_u8()])
        );
    }

    #[test]
    fn test_protocol_mode_values() {
        assert_eq!(ProtocolMode::Boot.as_u8(), 0);
        assert_eq!(ProtocolMode::Report.as_u8(), 1);
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = hid_over_gatt_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&HID_INFORMATION_UUID));
        assert!(uuids.contains(&REPORT_MAP_UUID));
        assert!(uuids.contains(&HID_CONTROL_POINT_UUID));
        assert!(uuids.contains(&REPORT_UUID));
        assert!(uuids.contains(&PROTOCOL_MODE_UUID));
    }
}
