// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Audio Stream Control Profile implementation.
//!
//! Based on Bluetooth SIG Audio Stream Control Service specification
//! (org.bluetooth.service.audio_stream_control).
//! Service UUID: 0x184E

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Audio Stream Control Service UUID (16-bit)
pub const AUDIO_STREAM_CONTROL_SERVICE_UUID: u16 = 0x184E;

/// Sink ASE characteristic UUID (16-bit)
pub const SINK_ASE_UUID: u16 = 0x2BC4;

/// Source ASE characteristic UUID (16-bit)
pub const SOURCE_ASE_UUID: u16 = 0x2BC5;

/// ASE Control Point characteristic UUID (16-bit)
pub const ASE_CONTROL_POINT_UUID: u16 = 0x2BC6;

/// BLE property for Read
const PROPERTY_READ: i32 = 1;

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4;

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2;

/// BLE property for Write Without Response
const PROPERTY_WRITE_NO_RSP: i32 = 16;

/// Audio Stream Endpoint (ASE) states
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AseState {
    /// Idle state
    Idle = 0x00,
    /// Codec Configured state
    CodecConfigured = 0x01,
    /// QoS Configured state
    QosConfigured = 0x02,
    /// Enabling state
    Enabling = 0x03,
    /// Streaming state
    Streaming = 0x04,
    /// Disabling state
    Disabling = 0x05,
    /// Releasing state
    Releasing = 0x06,
}

impl AseState {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// ASE Control Point operation codes
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AseControlOpCode {
    /// Config Codec
    ConfigCodec = 0x01,
    /// Config QoS
    ConfigQos = 0x02,
    /// Enable
    Enable = 0x03,
    /// Receiver Start Ready
    ReceiverStartReady = 0x04,
    /// Disable
    Disable = 0x05,
    /// Receiver Stop Ready
    ReceiverStopReady = 0x06,
    /// Update Metadata
    UpdateMetadata = 0x07,
    /// Release
    Release = 0x08,
}

impl AseControlOpCode {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Audio codec types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AudioCodecType {
    /// LC3 codec
    Lc3 = 0x06,
    /// Vendor specific codec
    VendorSpecific = 0xFF,
}

impl AudioCodecType {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Audio Stream Control Profile definition.
///
/// This profile enables LE Audio streaming control (part of LE Audio standard):
/// - Audio Stream Control Service (0x184E)
///   - Sink ASE (0x2BC4): Read, Notify (audio sink endpoint state)
///   - Source ASE (0x2BC5): Read, Notify (audio source endpoint state)
///   - ASE Control Point (0x2BC6): Write, Write Without Response, Notify (stream control)
///
/// Used for:
/// - Wireless headphones and earbuds
/// - Hearing aids
/// - Wireless speakers
/// - Multi-stream audio (music + voice calls)
/// - Broadcast audio
///
/// Part of the LE Audio standard, succeeding Classic Bluetooth audio.
///
/// # Returns
/// A complete `ProfileDefinition` for the Audio Stream Control Profile.
pub fn audio_stream_control_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        AUDIO_STREAM_CONTROL_SERVICE_UUID,
        vec![
            // Sink ASE - Read, Notify (audio sink endpoint state)
            CharacteristicDefinition::new(SINK_ASE_UUID, vec![PROPERTY_READ, PROPERTY_NOTIFY]),
            // Source ASE - Read, Notify (audio source endpoint state)
            CharacteristicDefinition::new(SOURCE_ASE_UUID, vec![PROPERTY_READ, PROPERTY_NOTIFY]),
            // ASE Control Point - Write, Write Without Response, Notify (stream control)
            CharacteristicDefinition::new(
                ASE_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_WRITE_NO_RSP, PROPERTY_NOTIFY],
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_stream_control_profile_structure() {
        let profile = audio_stream_control_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, AUDIO_STREAM_CONTROL_SERVICE_UUID);

        // Should have three characteristics
        assert_eq!(service.characteristics.len(), 3);

        // Check Sink ASE characteristic
        let sink_ase = &service.characteristics[0];
        assert_eq!(sink_ase.uuid, SINK_ASE_UUID);
        assert_eq!(sink_ase.properties, vec![PROPERTY_READ, PROPERTY_NOTIFY]);

        // Check Source ASE characteristic
        let source_ase = &service.characteristics[1];
        assert_eq!(source_ase.uuid, SOURCE_ASE_UUID);
        assert_eq!(source_ase.properties, vec![PROPERTY_READ, PROPERTY_NOTIFY]);

        // Check ASE Control Point characteristic
        let control_point = &service.characteristics[2];
        assert_eq!(control_point.uuid, ASE_CONTROL_POINT_UUID);
        assert_eq!(
            control_point.properties,
            vec![PROPERTY_WRITE, PROPERTY_WRITE_NO_RSP, PROPERTY_NOTIFY]
        );
    }

    #[test]
    fn test_ase_state_values() {
        assert_eq!(AseState::Idle.as_u8(), 0x00);
        assert_eq!(AseState::CodecConfigured.as_u8(), 0x01);
        assert_eq!(AseState::QosConfigured.as_u8(), 0x02);
        assert_eq!(AseState::Enabling.as_u8(), 0x03);
        assert_eq!(AseState::Streaming.as_u8(), 0x04);
        assert_eq!(AseState::Releasing.as_u8(), 0x06);
    }

    #[test]
    fn test_ase_control_op_code_values() {
        assert_eq!(AseControlOpCode::ConfigCodec.as_u8(), 0x01);
        assert_eq!(AseControlOpCode::ConfigQos.as_u8(), 0x02);
        assert_eq!(AseControlOpCode::Enable.as_u8(), 0x03);
        assert_eq!(AseControlOpCode::Disable.as_u8(), 0x05);
        assert_eq!(AseControlOpCode::Release.as_u8(), 0x08);
    }

    #[test]
    fn test_audio_codec_type_values() {
        assert_eq!(AudioCodecType::Lc3.as_u8(), 0x06);
        assert_eq!(AudioCodecType::VendorSpecific.as_u8(), 0xFF);
    }

    #[test]
    fn test_all_required_characteristics_present() {
        let profile = audio_stream_control_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&SINK_ASE_UUID));
        assert!(uuids.contains(&SOURCE_ASE_UUID));
        assert!(uuids.contains(&ASE_CONTROL_POINT_UUID));
    }
}
