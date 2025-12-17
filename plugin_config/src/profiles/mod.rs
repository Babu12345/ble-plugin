// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! BLE profile definitions based on Bluetooth SIG specifications.
//!
//! This module provides hardware-agnostic profile definitions for standard
//! BLE profiles that can be used by any BLE stack implementation.

pub mod battery_service;
pub mod blood_pressure;
pub mod body_composition;
pub mod bond_management;
pub mod continuous_glucose_monitoring;
pub mod current_time;
pub mod cycling_speed_cadence;
pub mod device_info;
pub mod environmental_sensing;
pub mod fitness_machine;
pub mod glucose_monitoring;
pub mod health_thermometer;
pub mod heart_rate;
pub mod hid_over_gatt;
pub mod insulin_delivery;
pub mod location_navigation;
pub mod phone_alert_status;
pub mod proximity;
pub mod pulse_oximeter;
pub mod running_speed_cadence;
pub mod scan_parameters;
pub mod user_data;
pub mod weight_scale;

/// A complete BLE profile definition containing one or more services.
#[derive(Debug, Clone)]
pub struct ProfileDefinition {
    /// The services that make up this profile
    pub services: Vec<ServiceDefinition>,
}

/// A BLE service definition with its UUID and characteristics.
#[derive(Debug, Clone)]
pub struct ServiceDefinition {
    /// The 16-bit service UUID
    pub uuid: u16,
    /// The characteristics belonging to this service
    pub characteristics: Vec<CharacteristicDefinition>,
}

/// A BLE characteristic definition with its UUID, properties, and optional default value.
#[derive(Debug, Clone)]
pub struct CharacteristicDefinition {
    /// The 16-bit characteristic UUID
    pub uuid: u16,
    /// The properties of this characteristic (as i32 for protocol compatibility)
    pub properties: Vec<i32>,
    /// Optional default value for the characteristic
    pub default_value: Option<Vec<u8>>,
}

impl ProfileDefinition {
    /// Creates a new profile definition with the given services.
    pub fn new(services: Vec<ServiceDefinition>) -> Self {
        Self { services }
    }
}

impl ServiceDefinition {
    /// Creates a new service definition with the given UUID and characteristics.
    pub fn new(uuid: u16, characteristics: Vec<CharacteristicDefinition>) -> Self {
        Self {
            uuid,
            characteristics,
        }
    }
}

impl CharacteristicDefinition {
    /// Creates a new characteristic definition with the given UUID and properties.
    pub fn new(uuid: u16, properties: Vec<i32>) -> Self {
        Self {
            uuid,
            properties,
            default_value: None,
        }
    }

    /// Creates a new characteristic definition with a default value.
    pub fn with_default_value(uuid: u16, properties: Vec<i32>, default_value: Vec<u8>) -> Self {
        Self {
            uuid,
            properties,
            default_value: Some(default_value),
        }
    }
}
