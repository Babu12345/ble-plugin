// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Fitness Machine Profile implementation.
//!
//! Based on Bluetooth SIG Fitness Machine Service specification
//! (org.bluetooth.service.fitness_machine).
//! Service UUID: 0x1826

use super::{CharacteristicDefinition, ProfileDefinition, ServiceDefinition};

/// Fitness Machine Service UUID (16-bit)
pub const FITNESS_MACHINE_SERVICE_UUID: u16 = 0x1826;

/// Fitness Machine Feature characteristic UUID (16-bit)
pub const FITNESS_MACHINE_FEATURE_UUID: u16 = 0x2ACC;

/// Treadmill Data characteristic UUID (16-bit)
pub const TREADMILL_DATA_UUID: u16 = 0x2ACD;

/// Cross Trainer Data characteristic UUID (16-bit)
pub const CROSS_TRAINER_DATA_UUID: u16 = 0x2ACE;

/// Step Climber Data characteristic UUID (16-bit)
pub const STEP_CLIMBER_DATA_UUID: u16 = 0x2ACF;

/// Stair Climber Data characteristic UUID (16-bit)
pub const STAIR_CLIMBER_DATA_UUID: u16 = 0x2AD0;

/// Rower Data characteristic UUID (16-bit)
pub const ROWER_DATA_UUID: u16 = 0x2AD1;

/// Indoor Bike Data characteristic UUID (16-bit)
pub const INDOOR_BIKE_DATA_UUID: u16 = 0x2AD2;

/// Training Status characteristic UUID (16-bit)
pub const TRAINING_STATUS_UUID: u16 = 0x2AD3;

/// Supported Speed Range characteristic UUID (16-bit)
pub const SUPPORTED_SPEED_RANGE_UUID: u16 = 0x2AD4;

/// Supported Inclination Range characteristic UUID (16-bit)
pub const SUPPORTED_INCLINATION_RANGE_UUID: u16 = 0x2AD5;

/// Supported Resistance Level Range characteristic UUID (16-bit)
pub const SUPPORTED_RESISTANCE_LEVEL_RANGE_UUID: u16 = 0x2AD6;

/// Supported Power Range characteristic UUID (16-bit)
pub const SUPPORTED_POWER_RANGE_UUID: u16 = 0x2AD8;

/// Fitness Machine Control Point characteristic UUID (16-bit)
pub const FITNESS_MACHINE_CONTROL_POINT_UUID: u16 = 0x2AD9;

/// Fitness Machine Status characteristic UUID (16-bit)
pub const FITNESS_MACHINE_STATUS_UUID: u16 = 0x2ADA;

/// BLE property for Read
const PROPERTY_READ: i32 = 1; // BleProperties::Read

/// BLE property for Notify
const PROPERTY_NOTIFY: i32 = 4; // BleProperties::Notify

/// BLE property for Write
const PROPERTY_WRITE: i32 = 2; // BleProperties::Write

/// BLE property for Indicate
const PROPERTY_INDICATE: i32 = 8; // BleProperties::Indicate

/// Fitness Machine Type
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum FitnessMachineType {
    /// Treadmill
    Treadmill = 0,
    /// Cross Trainer (elliptical)
    CrossTrainer = 1,
    /// Step Climber
    StepClimber = 2,
    /// Stair Climber
    StairClimber = 3,
    /// Rower
    Rower = 4,
    /// Indoor Bike
    IndoorBike = 5,
}

impl FitnessMachineType {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Training Status values
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TrainingStatus {
    /// Other
    Other = 0,
    /// Idle
    Idle = 1,
    /// Warming Up
    WarmingUp = 2,
    /// Low Intensity Interval
    LowIntensityInterval = 3,
    /// High Intensity Interval
    HighIntensityInterval = 4,
    /// Recovery Interval
    RecoveryInterval = 5,
    /// Isometric
    Isometric = 6,
    /// Heart Rate Control
    HeartRateControl = 7,
    /// Fitness Test
    FitnessTest = 8,
    /// Speed Outside of Control Region - Low
    SpeedOutsideControlLow = 9,
    /// Speed Outside of Control Region - High
    SpeedOutsideControlHigh = 10,
    /// Cool Down
    CoolDown = 11,
    /// Watt Control
    WattControl = 12,
    /// Manual Mode
    ManualMode = 13,
    /// Pre-Workout
    PreWorkout = 14,
    /// Post-Workout
    PostWorkout = 15,
}

impl TrainingStatus {
    /// Convert to byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Creates the Fitness Machine Profile definition.
///
/// This profile includes:
/// - Fitness Machine Service (0x1826)
///   - Fitness Machine Feature (0x2ACC): Read (machine type and features)
///   - Machine-specific Data characteristics: Notify (workout data)
///   - Training Status (0x2AD3): Read, Notify (training state)
///   - Supported Range characteristics: Read (machine capabilities)
///   - Fitness Machine Control Point (0x2AD9): Write, Indicate (control commands)
///   - Fitness Machine Status (0x2ADA): Notify (status changes and errors)
///
/// # Returns
/// A complete `ProfileDefinition` for the Fitness Machine Profile.
pub fn fitness_machine_profile() -> ProfileDefinition {
    ProfileDefinition::new(vec![ServiceDefinition::new(
        FITNESS_MACHINE_SERVICE_UUID,
        vec![
            // Fitness Machine Feature - Read (machine type and supported features)
            CharacteristicDefinition::new(FITNESS_MACHINE_FEATURE_UUID, vec![PROPERTY_READ]),
            // Treadmill Data - Notify (speed, inclination, distance, calories)
            CharacteristicDefinition::new(TREADMILL_DATA_UUID, vec![PROPERTY_NOTIFY]),
            // Cross Trainer Data - Notify (speed, stride rate, distance, resistance)
            CharacteristicDefinition::new(CROSS_TRAINER_DATA_UUID, vec![PROPERTY_NOTIFY]),
            // Step Climber Data - Notify (floors, step rate, elevation gain)
            CharacteristicDefinition::new(STEP_CLIMBER_DATA_UUID, vec![PROPERTY_NOTIFY]),
            // Stair Climber Data - Notify (floors, step rate, elevation gain)
            CharacteristicDefinition::new(STAIR_CLIMBER_DATA_UUID, vec![PROPERTY_NOTIFY]),
            // Rower Data - Notify (stroke rate, stroke count, distance, pace)
            CharacteristicDefinition::new(ROWER_DATA_UUID, vec![PROPERTY_NOTIFY]),
            // Indoor Bike Data - Notify (speed, cadence, resistance, power)
            CharacteristicDefinition::new(INDOOR_BIKE_DATA_UUID, vec![PROPERTY_NOTIFY]),
            // Training Status - Read, Notify (current training phase)
            CharacteristicDefinition::with_default_value(
                TRAINING_STATUS_UUID,
                vec![PROPERTY_READ, PROPERTY_NOTIFY],
                vec![TrainingStatus::Idle.as_u8()],
            ),
            // Supported Speed Range - Read (min/max speed capabilities)
            CharacteristicDefinition::new(SUPPORTED_SPEED_RANGE_UUID, vec![PROPERTY_READ]),
            // Supported Inclination Range - Read (min/max incline)
            CharacteristicDefinition::new(SUPPORTED_INCLINATION_RANGE_UUID, vec![PROPERTY_READ]),
            // Supported Resistance Level Range - Read (min/max resistance)
            CharacteristicDefinition::new(
                SUPPORTED_RESISTANCE_LEVEL_RANGE_UUID,
                vec![PROPERTY_READ],
            ),
            // Supported Power Range - Read (min/max power output)
            CharacteristicDefinition::new(SUPPORTED_POWER_RANGE_UUID, vec![PROPERTY_READ]),
            // Fitness Machine Control Point - Write, Indicate (start, stop, set parameters)
            CharacteristicDefinition::new(
                FITNESS_MACHINE_CONTROL_POINT_UUID,
                vec![PROPERTY_WRITE, PROPERTY_INDICATE],
            ),
            // Fitness Machine Status - Notify (machine status, errors, warnings)
            CharacteristicDefinition::new(FITNESS_MACHINE_STATUS_UUID, vec![PROPERTY_NOTIFY]),
        ],
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fitness_machine_profile_structure() {
        let profile = fitness_machine_profile();

        // Should have exactly one service
        assert_eq!(profile.services.len(), 1);

        let service = &profile.services[0];
        assert_eq!(service.uuid, FITNESS_MACHINE_SERVICE_UUID);

        // Should have fourteen characteristics
        assert_eq!(service.characteristics.len(), 14);

        // Check Fitness Machine Feature characteristic
        let feature = &service.characteristics[0];
        assert_eq!(feature.uuid, FITNESS_MACHINE_FEATURE_UUID);
        assert_eq!(feature.properties, vec![PROPERTY_READ]);

        // Check Training Status characteristic
        let training_status = &service.characteristics[7];
        assert_eq!(training_status.uuid, TRAINING_STATUS_UUID);
        assert_eq!(
            training_status.properties,
            vec![PROPERTY_READ, PROPERTY_NOTIFY]
        );
        assert_eq!(
            training_status.default_value,
            Some(vec![TrainingStatus::Idle.as_u8()])
        );
    }

    #[test]
    fn test_fitness_machine_type_values() {
        assert_eq!(FitnessMachineType::Treadmill.as_u8(), 0);
        assert_eq!(FitnessMachineType::CrossTrainer.as_u8(), 1);
        assert_eq!(FitnessMachineType::Rower.as_u8(), 4);
        assert_eq!(FitnessMachineType::IndoorBike.as_u8(), 5);
    }

    #[test]
    fn test_training_status_values() {
        assert_eq!(TrainingStatus::Idle.as_u8(), 1);
        assert_eq!(TrainingStatus::WarmingUp.as_u8(), 2);
        assert_eq!(TrainingStatus::HighIntensityInterval.as_u8(), 4);
        assert_eq!(TrainingStatus::CoolDown.as_u8(), 11);
        assert_eq!(TrainingStatus::PostWorkout.as_u8(), 15);
    }

    #[test]
    fn test_all_machine_data_characteristics_present() {
        let profile = fitness_machine_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&TREADMILL_DATA_UUID));
        assert!(uuids.contains(&CROSS_TRAINER_DATA_UUID));
        assert!(uuids.contains(&STEP_CLIMBER_DATA_UUID));
        assert!(uuids.contains(&STAIR_CLIMBER_DATA_UUID));
        assert!(uuids.contains(&ROWER_DATA_UUID));
        assert!(uuids.contains(&INDOOR_BIKE_DATA_UUID));
    }

    #[test]
    fn test_control_characteristics_present() {
        let profile = fitness_machine_profile();
        let service = &profile.services[0];

        let uuids: Vec<u16> = service.characteristics.iter().map(|c| c.uuid).collect();
        assert!(uuids.contains(&FITNESS_MACHINE_CONTROL_POINT_UUID));
        assert!(uuids.contains(&FITNESS_MACHINE_STATUS_UUID));
        assert!(uuids.contains(&TRAINING_STATUS_UUID));
    }
}
