// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Insulin Delivery Profile implementation.
//!
//! Based on Bluetooth SIG Insulin Delivery Service specification
//! (org.bluetooth.service.insulin_delivery).
//! Service UUID: 0x183A

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Insulin Delivery Service UUID (16-bit)
pub const INSULIN_DELIVERY_SERVICE_UUID: u16 = 0x183A;

/// IDD Status Changed characteristic UUID (16-bit)
pub const IDD_STATUS_CHANGED_UUID: u16 = 0x2B20;

/// IDD Status characteristic UUID (16-bit)
pub const IDD_STATUS_UUID: u16 = 0x2B21;

/// IDD Annunciation Status characteristic UUID (16-bit)
pub const IDD_ANNUNCIATION_STATUS_UUID: u16 = 0x2B22;

/// IDD Features characteristic UUID (16-bit)
pub const IDD_FEATURES_UUID: u16 = 0x2B23;

/// IDD Status Reader Control Point characteristic UUID (16-bit)
pub const IDD_STATUS_READER_CONTROL_POINT_UUID: u16 = 0x2B24;

/// IDD Command Control Point characteristic UUID (16-bit)
pub const IDD_COMMAND_CONTROL_POINT_UUID: u16 = 0x2B25;

/// IDD Command Data characteristic UUID (16-bit)
pub const IDD_COMMAND_DATA_UUID: u16 = 0x2B26;

/// IDD Record Access Control Point characteristic UUID (16-bit)
pub const IDD_RECORD_ACCESS_CONTROL_POINT_UUID: u16 = 0x2B27;

/// IDD History Data characteristic UUID (16-bit)
pub const IDD_HISTORY_DATA_UUID: u16 = 0x2B28;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2; // BleProperties::Write

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8; // BleProperties::Indicate

/// IDD Feature flags as defined by Bluetooth SIG
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum IddFeature {
    /// Basal Rate Delivery Supported (bit 0)
    BasalRateDeliverySupported = 0x0001,
    /// TBR Supported (bit 1)
    TbrSupported = 0x0002,
    /// Bolus Delivery Supported (bit 2)
    BolusDeliverySupported = 0x0004,
    /// Bolus Template Supported (bit 3)
    BolusTemplateSupported = 0x0008,
    /// Service Supported (bit 4)
    ServiceSupported = 0x0010,
    /// Pump Status Supported (bit 5)
    PumpStatusSupported = 0x0020,
    /// Device Specific Alerts Supported (bit 6)
    DeviceSpecificAlertsSupported = 0x0040,
    /// Multiple Bonds Supported (bit 7)
    MultipleBondsSupported = 0x0080,
}

impl IddFeature {
    /// Convert to u16 value
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Insulin Delivery Device Status values
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IddStatus {
    /// Device ready
    Ready = 0,
    /// Priming
    Priming = 1,
    /// Delivery Active
    DeliveryActive = 2,
    /// Delivery Stopped
    DeliveryStopped = 3,
    /// Cartridge Empty
    CartridgeEmpty = 4,
    /// Cartridge Low
    CartridgeLow = 5,
    /// Battery Low
    BatteryLow = 6,
}

impl IddStatus {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Insulin Delivery Profile definition.
///
/// This profile includes:
/// - Insulin Delivery Service (0x183A)
///   - IDD Status Changed (0x2B20): Indicate (status change notifications)
///   - IDD Status (0x2B21): Read (current device status)
///   - IDD Annunciation Status (0x2B22): Read (alerts and warnings)
///   - IDD Features (0x2B23): Read (supported features)
///   - IDD Status Reader Control Point (0x2B24): Write, Indicate (status queries)
///   - IDD Command Control Point (0x2B25): Write, Indicate (delivery commands)
///   - IDD Command Data (0x2B26): Write (command parameters)
///   - IDD Record Access Control Point (0x2B27): Write, Indicate (history access)
///   - IDD History Data (0x2B28): Notify (historical delivery data)
///
/// # Returns
/// A complete `ProfileDefinition` for the Insulin Delivery Profile.
pub fn insulin_delivery_profile() -> ProfileDefinition {
    // Default features: basal rate, bolus, pump status
    let default_features = IddFeature::BasalRateDeliverySupported.as_u16()
        | IddFeature::BolusDeliverySupported.as_u16()
        | IddFeature::PumpStatusSupported.as_u16()
        | IddFeature::DeviceSpecificAlertsSupported.as_u16();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        INSULIN_DELIVERY_SERVICE_UUID,
        vec![
            // IDD Status Changed - Indicate (notify when status changes)
            CharacteristicDefinition::new(IDD_STATUS_CHANGED_UUID, vec![PROPERTY_INDICATE]),
            // IDD Status - Read (current device status)
            CharacteristicDefinition::with_default_value(
                IDD_STATUS_UUID,
                vec![PROPERTY_READ],
                vec![IddStatus::Ready.as_u8()],
            ),
            // IDD Annunciation Status - Read (alerts, warnings, errors)
            CharacteristicDefinition::new(IDD_ANNUNCIATION_STATUS_UUID, vec![PROPERTY_READ]),
            // IDD Features - Read (supported features bitmask)
            CharacteristicDefinition::with_default_value(
                IDD_FEATURES_UUID,
                vec![PROPERTY_READ],
                default_features.to_le_bytes().to_vec(),
            ),
            // IDD Status Reader Control Point - Write, Indicate (query status details)
            CharacteristicDefinition::new(
                IDD_STATUS_READER_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
            // IDD Command Control Point - Write, Indicate (insulin delivery commands)
            CharacteristicDefinition::new(
                IDD_COMMAND_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
            // IDD Command Data - Write (parameters for commands)
            CharacteristicDefinition::new(IDD_COMMAND_DATA_UUID, vec![PROPERTY_WRITE]),
            // IDD Record Access Control Point - Write, Indicate (access delivery history)
            CharacteristicDefinition::new(
                IDD_RECORD_ACCESS_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
            // IDD History Data - Notify (historical insulin delivery data)
            CharacteristicDefinition::new(IDD_HISTORY_DATA_UUID, vec![PROPERTY_NOTIFY]),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insulin_delivery_profile_structure() {
        let profile = insulin_delivery_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, INSULIN_DELIVERY_SERVICE_UUID);

        // Should have nine characteristics
        assert_eq!(service.characteristics.len(), 9);

        // Check IDD Status Changed characteristic
        let status_changed = &service.characteristics[0];
        assert_eq!(status_changed.uuid, IDD_STATUS_CHANGED_UUID);
        assert_eq!(status_changed.properties, vec![PROPERTY_INDICATE]);

        // Check IDD Status characteristic
        let status = &service.characteristics[1];
        assert_eq!(status.uuid, IDD_STATUS_UUID);
        assert_eq!(status.properties, vec![PROPERTY_READ]);
        assert_eq!(status.default_value, Some(vec![IddStatus::Ready.as_u8()]));

        // Check IDD Features characteristic
        let features = &service.characteristics[3];
        assert_eq!(features.uuid, IDD_FEATURES_UUID);
        assert!(features.default_value.is_some());
    }

    #[test]
    fn test_idd_feature_values() {
        assert_eq!(IddFeature::BasalRateDeliverySupported.as_u16(), 0x0001);
        assert_eq!(IddFeature::TbrSupported.as_u16(), 0x0002);
        assert_eq!(IddFeature::BolusDeliverySupported.as_u16(), 0x0004);
        assert_eq!(IddFeature::BolusTemplateSupported.as_u16(), 0x0008);
        assert_eq!(IddFeature::PumpStatusSupported.as_u16(), 0x0020);
        assert_eq!(IddFeature::MultipleBondsSupported.as_u16(), 0x0080);
    }

    #[test]
    fn test_idd_status_values() {
        assert_eq!(IddStatus::Ready.as_u8(), 0);
        assert_eq!(IddStatus::Priming.as_u8(), 1);
        assert_eq!(IddStatus::DeliveryActive.as_u8(), 2);
        assert_eq!(IddStatus::DeliveryStopped.as_u8(), 3);
        assert_eq!(IddStatus::CartridgeEmpty.as_u8(), 4);
        assert_eq!(IddStatus::BatteryLow.as_u8(), 6);
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = insulin_delivery_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&IDD_STATUS_CHANGED_UUID));
        assert!(uuids.contains(&IDD_STATUS_UUID));
        assert!(uuids.contains(&IDD_ANNUNCIATION_STATUS_UUID));
        assert!(uuids.contains(&IDD_FEATURES_UUID));
        assert!(uuids.contains(&IDD_STATUS_READER_CONTROL_POINT_UUID));
        assert!(uuids.contains(&IDD_COMMAND_CONTROL_POINT_UUID));
        assert!(uuids.contains(&IDD_COMMAND_DATA_UUID));
        assert!(uuids.contains(&IDD_RECORD_ACCESS_CONTROL_POINT_UUID));
        assert!(uuids.contains(&IDD_HISTORY_DATA_UUID));
    }
}
