// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Proximity Profile implementation.
//!
//! Based on Bluetooth SIG Proximity Profile specification
//! (org.bluetooth.profile.prx).
//!
//! This profile consists of three services:
//! - Link Loss Service (0x1803)
//! - Immediate Alert Service (0x1802)
//! - Tx Power Service (0x1804)

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Link Loss Service UUID (16-bit)
pub const LINK_LOSS_SERVICE_UUID: u16 = 0x1803;

/// Immediate Alert Service UUID (16-bit)
pub const IMMEDIATE_ALERT_SERVICE_UUID: u16 = 0x1802;

/// Tx Power Service UUID (16-bit)
pub const TX_POWER_SERVICE_UUID: u16 = 0x1804;

/// Alert Level characteristic UUID (16-bit)
/// Used in both Link Loss and Immediate Alert services
pub const ALERT_LEVEL_UUID: u16 = 0x2A06;

/// Tx Power Level characteristic UUID (16-bit)
pub const TX_POWER_LEVEL_UUID: u16 = 0x2A07;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2; // BleProperties::Write

/// Alert level values as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AlertLevel {
    /// No alert
    NoAlert = 0,
    /// Mild alert
    MildAlert = 1,
    /// High alert
    HighAlert = 2,
}

impl AlertLevel {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Proximity Profile definition.
///
/// This profile includes three services:
/// - Link Loss Service (0x1803)
///   - Alert Level (0x2A06): Read, Write (default: No Alert)
/// - Immediate Alert Service (0x1802)
///   - Alert Level (0x2A06): Write (no default, write-only)
/// - Tx Power Service (0x1804)
///   - Tx Power Level (0x2A07): Read
///
/// # Returns
/// A complete `ProfileDefinition` for the Proximity Profile.
pub fn proximity_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![
        // Link Loss Service - Alert when connection is lost
        ServiceDefinition::new(
            LINK_LOSS_SERVICE_UUID,
            vec![CharacteristicDefinition::with_default_value(
                ALERT_LEVEL_UUID,
                vec![PROPERTY_READ, PROPERTY_WRITE],
                vec![AlertLevel::NoAlert.as_u8()],
            )],
        ),
        // Immediate Alert Service - Trigger alert immediately
        ServiceDefinition::new(
            IMMEDIATE_ALERT_SERVICE_UUID,
            vec![CharacteristicDefinition::new(
                ALERT_LEVEL_UUID,
                vec![PROPERTY_WRITE],
            )],
        ),
        // Tx Power Service - Report transmit power for distance estimation
        ServiceDefinition::new(
            TX_POWER_SERVICE_UUID,
            vec![CharacteristicDefinition::new(
                TX_POWER_LEVEL_UUID,
                vec![PROPERTY_READ],
            )],
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proximity_profile_structure() {
        let profile = proximity_profile();

        // Should have exactly three services
        assert_eq!(profile.services.len(), 3);

        // Check Link Loss Service
        let link_loss = &profile.services[0];
        assert_eq!(link_loss.uuid, LINK_LOSS_SERVICE_UUID);
        assert_eq!(link_loss.characteristics.len(), 1);
        assert_eq!(link_loss.characteristics[0].uuid, ALERT_LEVEL_UUID);
        assert_eq!(
            link_loss.characteristics[0].properties,
            vec![PROPERTY_READ, PROPERTY_WRITE]
        );
        assert_eq!(
            link_loss.characteristics[0].default_value,
            Some(vec![AlertLevel::NoAlert.as_u8()])
        );

        // Check Immediate Alert Service
        let immediate_alert = &profile.services[1];
        assert_eq!(immediate_alert.uuid, IMMEDIATE_ALERT_SERVICE_UUID);
        assert_eq!(immediate_alert.characteristics.len(), 1);
        assert_eq!(immediate_alert.characteristics[0].uuid, ALERT_LEVEL_UUID);
        assert_eq!(
            immediate_alert.characteristics[0].properties,
            vec![PROPERTY_WRITE]
        );
        assert!(immediate_alert.characteristics[0].default_value.is_none());

        // Check Tx Power Service
        let tx_power = &profile.services[2];
        assert_eq!(tx_power.uuid, TX_POWER_SERVICE_UUID);
        assert_eq!(tx_power.characteristics.len(), 1);
        assert_eq!(tx_power.characteristics[0].uuid, TX_POWER_LEVEL_UUID);
        assert_eq!(tx_power.characteristics[0].properties, vec![PROPERTY_READ]);
        assert!(tx_power.characteristics[0].default_value.is_none());
    }

    #[test]
    fn test_alert_level_values() {
        assert_eq!(AlertLevel::NoAlert.as_u8(), 0);
        assert_eq!(AlertLevel::MildAlert.as_u8(), 1);
        assert_eq!(AlertLevel::HighAlert.as_u8(), 2);
    }

    #[test]
    fn test_service_uuids() {
        let profile = proximity_profile();
        let uuids: Vec<u16> = profile.services.iter().map(|s| s.uuid).collect();

        assert!(uuids.contains(&LINK_LOSS_SERVICE_UUID));
        assert!(uuids.contains(&IMMEDIATE_ALERT_SERVICE_UUID));
        assert!(uuids.contains(&TX_POWER_SERVICE_UUID));
    }
}
