// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Current Time Service profile implementation.
//!
//! Based on Bluetooth SIG Current Time Service specification
//! (org.bluetooth.service.current_time).
//! Service UUID: 0x1805

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Current Time Service UUID (16-bit)
pub const CURRENT_TIME_SERVICE_UUID: u16 = 0x1805;

/// Current Time characteristic UUID (16-bit)
pub const CURRENT_TIME_UUID: u16 = 0x2A2B;

/// Local Time Information characteristic UUID (16-bit)
pub const LOCAL_TIME_INFORMATION_UUID: u16 = 0x2A0F;

/// Reference Time Information characteristic UUID (16-bit)
pub const REFERENCE_TIME_INFORMATION_UUID: u16 = 0x2A14;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2; // BleProperties::Write

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// Creates the Current Time Service profile definition.
///
/// This profile includes:
/// - Current Time Service (0x1805)
///   - Current Time (0x2A2B): Read, Notify, Write
///   - Local Time Information (0x2A0F): Read, Write (timezone offset, DST)
///   - Reference Time Information (0x2A14): Read (time source accuracy)
///
/// The Current Time characteristic contains the current date and time,
/// and can be updated by writing to it. Clients can subscribe to notifications
/// to receive time updates.
///
/// # Returns
/// A complete `ProfileDefinition` for the Current Time Service profile.
pub fn current_time_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        CURRENT_TIME_SERVICE_UUID,
        vec![
            // Current Time - Read, Notify, Write (date and time)
            CharacteristicDefinition::new(
                CURRENT_TIME_UUID,
                vec![PROPERTY_READ, PROPERTY_NOTIFY, PROPERTY_WRITE],
            ),
            // Local Time Information - Read, Write (timezone and DST offset)
            CharacteristicDefinition::new(
                LOCAL_TIME_INFORMATION_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
            ),
            // Reference Time Information - Read (time source and accuracy)
            CharacteristicDefinition::new(REFERENCE_TIME_INFORMATION_UUID, vec![PROPERTY_READ]),
        ],
    )])
}

/// Creates a simplified Current Time Service profile with only essential characteristics.
///
/// This profile includes only:
/// - Current Time Service (0x1805)
///   - Current Time (0x2A2B): Read, Notify, Write
///
/// This simplified version is suitable for devices that only need basic time
/// synchronization without timezone or time source information.
///
/// # Returns
/// A simplified `ProfileDefinition` for the Current Time Service profile.
pub fn current_time_profile_simple() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        CURRENT_TIME_SERVICE_UUID,
        vec![
            // Current Time - Read, Notify, Write (date and time only)
            CharacteristicDefinition::new(
                CURRENT_TIME_UUID,
                vec![PROPERTY_READ, PROPERTY_NOTIFY, PROPERTY_WRITE],
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_time_profile_structure() {
        let profile = current_time_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, CURRENT_TIME_SERVICE_UUID);

        // Should have three characteristics
        assert_eq!(service.characteristics.len(), 3);

        // Check Current Time characteristic
        let current_time = &service.characteristics[0];
        assert_eq!(current_time.uuid, CURRENT_TIME_UUID);
        assert_eq!(
            current_time.properties,
            vec![PROPERTY_READ, PROPERTY_NOTIFY, PROPERTY_WRITE]
        );
        assert!(current_time.default_value.is_none());

        // Check Local Time Information characteristic
        let local_time_info = &service.characteristics[1];
        assert_eq!(local_time_info.uuid, LOCAL_TIME_INFORMATION_UUID);
        assert_eq!(
            local_time_info.properties,
            vec![PROPERTY_READ, PROPERTY_WRITE]
        );
        assert!(local_time_info.default_value.is_none());

        // Check Reference Time Information characteristic
        let ref_time_info = &service.characteristics[2];
        assert_eq!(ref_time_info.uuid, REFERENCE_TIME_INFORMATION_UUID);
        assert_eq!(ref_time_info.properties, vec![PROPERTY_READ]);
        assert!(ref_time_info.default_value.is_none());
    }

    #[test]
    fn test_current_time_profile_simple_structure() {
        let profile = current_time_profile_simple();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, CURRENT_TIME_SERVICE_UUID);

        // Should have only one characteristic
        assert_eq!(service.characteristics.len(), 1);

        // Check Current Time characteristic
        let current_time = &service.characteristics[0];
        assert_eq!(current_time.uuid, CURRENT_TIME_UUID);
        assert_eq!(
            current_time.properties,
            vec![PROPERTY_READ, PROPERTY_NOTIFY, PROPERTY_WRITE]
        );
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = current_time_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&CURRENT_TIME_UUID));
        assert!(uuids.contains(&LOCAL_TIME_INFORMATION_UUID));
        assert!(uuids.contains(&REFERENCE_TIME_INFORMATION_UUID));
    }
}
