// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Device Information Service profile implementation.
//!
//! Based on Bluetooth SIG Device Information Service specification
//! (org.bluetooth.service.device_information).
//! Service UUID: 0x180A

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Device Information Service UUID (16-bit)
pub const DEVICE_INFORMATION_SERVICE_UUID: u16 = 0x180A;

/// Manufacturer Name String characteristic UUID (16-bit)
pub const MANUFACTURER_NAME_UUID: u16 = 0x2A29;

/// Model Number String characteristic UUID (16-bit)
pub const MODEL_NUMBER_UUID: u16 = 0x2A24;

/// Serial Number String characteristic UUID (16-bit)
pub const SERIAL_NUMBER_UUID: u16 = 0x2A25;

/// Hardware Revision String characteristic UUID (16-bit)
pub const HARDWARE_REVISION_UUID: u16 = 0x2A27;

/// Firmware Revision String characteristic UUID (16-bit)
pub const FIRMWARE_REVISION_UUID: u16 = 0x2A26;

/// Software Revision String characteristic UUID (16-bit)
pub const SOFTWARE_REVISION_UUID: u16 = 0x2A28;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// Default manufacturer name
const DEFAULT_MANUFACTURER: &str = "Wanyeki Technologies";

/// Default model number
const DEFAULT_MODEL_NUMBER: &str = "BLE-Plugin-v1";

/// Default firmware revision
const DEFAULT_FIRMWARE_REVISION: &str = "1.0.0";

/// Creates the Device Information Service profile definition.
///
/// This profile includes:
/// - Device Information Service (0x180A)
///   - Manufacturer Name (0x2A29): Read (default: "Wanyeki Technologies")
///   - Model Number (0x2A24): Read (default: "BLE-Plugin-v1")
///   - Firmware Revision (0x2A26): Read (default: "1.0.0")
///
/// All characteristics are read-only and contain default values that can be
/// updated later via the `configure_characteristic_read` command.
///
/// # Returns
/// A complete `ProfileDefinition` for the Device Information Service profile.
pub fn device_info_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        DEVICE_INFORMATION_SERVICE_UUID,
        vec![
            // Manufacturer Name - Read with default value
            CharacteristicDefinition::with_default_value(
                MANUFACTURER_NAME_UUID,
                vec![PROPERTY_READ],
                DEFAULT_MANUFACTURER.as_bytes().to_vec(),
            ),
            // Model Number - Read with default value
            CharacteristicDefinition::with_default_value(
                MODEL_NUMBER_UUID,
                vec![PROPERTY_READ],
                DEFAULT_MODEL_NUMBER.as_bytes().to_vec(),
            ),
            // Firmware Revision - Read with default value
            CharacteristicDefinition::with_default_value(
                FIRMWARE_REVISION_UUID,
                vec![PROPERTY_READ],
                DEFAULT_FIRMWARE_REVISION.as_bytes().to_vec(),
            ),
        ],
    )])
}

/// Creates an extended Device Information Service profile with additional characteristics.
///
/// This profile includes all characteristics from the basic profile plus:
///   - Serial Number (0x2A25): Read (default: "000000000000")
///   - Hardware Revision (0x2A27): Read (default: "1.0")
///   - Software Revision (0x2A28): Read (default: "1.0.0")
///
/// # Returns
/// A complete `ProfileDefinition` for the extended Device Information Service profile.
pub fn device_info_profile_extended() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        DEVICE_INFORMATION_SERVICE_UUID,
        vec![
            // Manufacturer Name
            CharacteristicDefinition::with_default_value(
                MANUFACTURER_NAME_UUID,
                vec![PROPERTY_READ],
                DEFAULT_MANUFACTURER.as_bytes().to_vec(),
            ),
            // Model Number
            CharacteristicDefinition::with_default_value(
                MODEL_NUMBER_UUID,
                vec![PROPERTY_READ],
                DEFAULT_MODEL_NUMBER.as_bytes().to_vec(),
            ),
            // Serial Number
            CharacteristicDefinition::with_default_value(
                SERIAL_NUMBER_UUID,
                vec![PROPERTY_READ],
                b"000000000000".to_vec(),
            ),
            // Hardware Revision
            CharacteristicDefinition::with_default_value(
                HARDWARE_REVISION_UUID,
                vec![PROPERTY_READ],
                b"1.0".to_vec(),
            ),
            // Firmware Revision
            CharacteristicDefinition::with_default_value(
                FIRMWARE_REVISION_UUID,
                vec![PROPERTY_READ],
                DEFAULT_FIRMWARE_REVISION.as_bytes().to_vec(),
            ),
            // Software Revision
            CharacteristicDefinition::with_default_value(
                SOFTWARE_REVISION_UUID,
                vec![PROPERTY_READ],
                b"1.0.0".to_vec(),
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_profile_structure() {
        let profile = device_info_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, DEVICE_INFORMATION_SERVICE_UUID);

        // Should have three characteristics
        assert_eq!(service.characteristics.len(), 3);

        // Check Manufacturer Name characteristic
        let manufacturer = &service.characteristics[0];
        assert_eq!(manufacturer.uuid, MANUFACTURER_NAME_UUID);
        assert_eq!(manufacturer.properties, vec![PROPERTY_READ]);
        assert_eq!(
            manufacturer.default_value,
            Some(DEFAULT_MANUFACTURER.as_bytes().to_vec())
        );

        // Check Model Number characteristic
        let model = &service.characteristics[1];
        assert_eq!(model.uuid, MODEL_NUMBER_UUID);
        assert_eq!(model.properties, vec![PROPERTY_READ]);
        assert_eq!(
            model.default_value,
            Some(DEFAULT_MODEL_NUMBER.as_bytes().to_vec())
        );

        // Check Firmware Revision characteristic
        let firmware = &service.characteristics[2];
        assert_eq!(firmware.uuid, FIRMWARE_REVISION_UUID);
        assert_eq!(firmware.properties, vec![PROPERTY_READ]);
        assert_eq!(
            firmware.default_value,
            Some(DEFAULT_FIRMWARE_REVISION.as_bytes().to_vec())
        );
    }

    #[test]
    fn test_device_info_profile_extended_structure() {
        let profile = device_info_profile_extended();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, DEVICE_INFORMATION_SERVICE_UUID);

        // Should have six characteristics
        assert_eq!(service.characteristics.len(), 6);

        // Verify all UUIDs are present
        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&MANUFACTURER_NAME_UUID));
        assert!(uuids.contains(&MODEL_NUMBER_UUID));
        assert!(uuids.contains(&SERIAL_NUMBER_UUID));
        assert!(uuids.contains(&HARDWARE_REVISION_UUID));
        assert!(uuids.contains(&FIRMWARE_REVISION_UUID));
        assert!(uuids.contains(&SOFTWARE_REVISION_UUID));
    }

    #[test]
    fn test_all_characteristics_have_default_values() {
        let profile = device_info_profile();
        for characteristic in profile.services[0].characteristics.iter() {
            assert!(
                characteristic.default_value.is_some(),
                "Characteristic {} should have a default value",
                characteristic.uuid
            );
        }
    }
}
