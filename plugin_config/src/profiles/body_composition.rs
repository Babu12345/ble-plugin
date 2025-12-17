// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Body Composition Profile implementation.
//!
//! Based on Bluetooth SIG Body Composition Service specification
//! (org.bluetooth.service.body_composition).
//! Service UUID: 0x181B

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Body Composition Service UUID (16-bit)
pub const BODY_COMPOSITION_SERVICE_UUID: u16 = 0x181B;

/// Body Composition Feature characteristic UUID (16-bit)
pub const BODY_COMPOSITION_FEATURE_UUID: u16 = 0x2A9B;

/// Body Composition Measurement characteristic UUID (16-bit)
pub const BODY_COMPOSITION_MEASUREMENT_UUID: u16 = 0x2A9C;

/// BLE property for Read
const PROPERTY_READ: i32 = 1;

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8;

/// Body Composition Feature flags
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum BodyCompositionFeature {
    /// Time stamp supported
    TimeStampSupported = 0x00000001,
    /// Multiple users supported
    MultipleUsersSupported = 0x00000002,
    /// Basal Metabolism supported
    BasalMetabolismSupported = 0x00000004,
    /// Muscle Percentage supported
    MusclePercentageSupported = 0x00000008,
    /// Muscle Mass supported
    MuscleMassSupported = 0x00000010,
    /// Fat Free Mass supported
    FatFreeMassSupported = 0x00000020,
    /// Soft Lean Mass supported
    SoftLeanMassSupported = 0x00000040,
    /// Body Water Mass supported
    BodyWaterMassSupported = 0x00000080,
    /// Impedance supported
    ImpedanceSupported = 0x00000100,
    /// Weight supported
    WeightSupported = 0x00000200,
    /// Height supported
    HeightSupported = 0x00000400,
}

impl BodyCompositionFeature {
    /// Convert to u32 value
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Creates the Body Composition Profile definition.
pub fn body_composition_profile() -> ProfileDefinition {
    let default_features = BodyCompositionFeature::WeightSupported.as_u32()
        | BodyCompositionFeature::MuscleMassSupported.as_u32()
        | BodyCompositionFeature::FatFreeMassSupported.as_u32()
        | BodyCompositionFeature::BodyWaterMassSupported.as_u32()
        | BodyCompositionFeature::ImpedanceSupported.as_u32()
        | BodyCompositionFeature::MultipleUsersSupported.as_u32();

    ProfileDefinition::new(vec![ServiceDefinition::new(
        BODY_COMPOSITION_SERVICE_UUID,
        vec![
            CharacteristicDefinition::with_default_value(
                BODY_COMPOSITION_FEATURE_UUID,
                vec![PROPERTY_READ],
                default_features.to_le_bytes().to_vec(),
            ),
            CharacteristicDefinition::new(
                BODY_COMPOSITION_MEASUREMENT_UUID,
                vec![PROPERTY_INDICATE],
            ),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_composition_profile_structure() {
        let profile = body_composition_profile();
        assert_eq!(profile.services.len(), 1);
        assert_eq!(profile.services[0].uuid, BODY_COMPOSITION_SERVICE_UUID);
        assert_eq!(profile.services[0].characteristics.len(), 2);
    }

    #[test]
    fn test_feature_values() {
        assert_eq!(BodyCompositionFeature::WeightSupported.as_u32(), 0x00000200);
        assert_eq!(BodyCompositionFeature::ImpedanceSupported.as_u32(), 0x00000100);
    }
}
