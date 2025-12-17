// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Scan Parameters Profile implementation.
//!
//! Based on Bluetooth SIG Scan Parameters Service specification
//! (org.bluetooth.service.scan_parameters).
//! Service UUID: 0x1813

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Scan Parameters Service UUID (16-bit)
pub const SCAN_PARAMETERS_SERVICE_UUID: u16 = 0x1813;

/// Scan Interval Window characteristic UUID (16-bit)
pub const SCAN_INTERVAL_WINDOW_UUID: u16 = 0x2A4F;

/// Scan Refresh characteristic UUID (16-bit)
pub const SCAN_REFRESH_UUID: u16 = 0x2A31;

/// BLE property for Write Without Response
const PROPERTY_WRITE_NO_RSP: i32 = 16;

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4;

/// Creates the Scan Parameters Profile definition.
///
/// This profile allows a GATT Client to store the LE scan parameters it is using
/// on a GATT Server device for power optimization.
pub fn scan_parameters_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        SCAN_PARAMETERS_SERVICE_UUID,
        vec![
            CharacteristicDefinition::new(
                SCAN_INTERVAL_WINDOW_UUID,
                vec![PROPERTY_WRITE_NO_RSP],
            ),
            CharacteristicDefinition::new(SCAN_REFRESH_UUID, vec![PROPERTY_NOTIFY]),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_parameters_profile_structure() {
        let profile = scan_parameters_profile();
        assert_eq!(profile.services.len(), 1);
        assert_eq!(profile.services[0].uuid, SCAN_PARAMETERS_SERVICE_UUID);
        assert_eq!(profile.services[0].characteristics.len(), 2);

        let uuids: Vec<u16> = profile.services[0].characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&SCAN_INTERVAL_WINDOW_UUID));
        assert!(uuids.contains(&SCAN_REFRESH_UUID));
    }
}
