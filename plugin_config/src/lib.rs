#![deny(missing_docs)]
// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Library that contains hardware agnostic methods for the plugin hardware to be used in the state machine

use std::fmt::Debug;

pub use protocol::plugin::*;
pub use protocol::protocol::*;
pub use protocol::utils::*;
pub use protocol::DEFAULT_PACKET_SIZE;

pub mod profiles;
/// Hardware agnostic plugin configurator
pub trait PluginConfig<ERROR: Debug> {
    /// Handle peripheral configuration
    fn handle_configure_peripheral(
        &mut self,
        _cmd: HostCommandConfigurePeripheral,
    ) -> Result<(), ERROR> {
        unimplemented!(
            "Please implement handle_configure_peripheral to configure the BLE peripheral"
        )
    }

    /// Handle peripheral security configuration
    fn handle_configure_peripheral_security(
        &mut self,
        _cmd: HostCommandConfigurePeripheralSecurity,
    ) -> Result<(), ERROR> {
        unimplemented!("Please implement handle_configure_peripheral_security to configure BLE security settings")
    }

    /// Handle start advertisement
    fn handle_start_advertisement(
        &mut self,
        _cmd: HostCommandStartAdvertisement,
    ) -> Result<(), ERROR> {
        unimplemented!("Please implement handle_start_advertisement to start BLE advertising")
    }

    /// Handle stop advertisement
    fn handle_stop_advertisement(
        &mut self,
        _cmd: HostCommandStopAdvertisement,
    ) -> Result<(), ERROR> {
        unimplemented!("Please implement handle_stop_advertisement to stop BLE advertising")
    }

    /// Handle service configuration
    fn handle_configure_service(&mut self, _cmd: HostCommandConfigureService) -> Result<(), ERROR> {
        unimplemented!("Please implement handle_configure_service to configure BLE services")
    }

    /// Handle characteristic configuration
    fn handle_configure_characteristic(
        &mut self,
        _cmd: HostCommandConfigureCharacteristic,
    ) -> Result<(), ERROR> {
        unimplemented!(
            "Please implement handle_configure_characteristic to configure BLE characteristics"
        )
    }

    /// Handle characteristic read configuration
    fn handle_configure_characteristic_read(
        &mut self,
        _cmd: HostCommandConfigureCharacteristicRead,
    ) -> Result<(), ERROR> {
        unimplemented!("Please implement handle_configure_characteristic_read to configure characteristic read operations")
    }

    /// Handle notify characteristic value
    fn handle_notify_characteristic_value(
        &mut self,
        _cmd: HostCommandNotifyCharacteristicValue,
    ) -> Result<(), ERROR> {
        unimplemented!("Please implement handle_notify_characteristic_value to send characteristic notifications")
    }

    /// Handle get service info
    fn handle_get_service_info(&mut self, _cmd: HostCommandGetServiceInfo) -> Result<(), ERROR> {
        unimplemented!("Please implement handle_get_service_info to retrieve service information")
    }

    /// Handle get characteristic info
    fn handle_get_characteristic_info(
        &mut self,
        _cmd: HostCommandGetCharacteristicInfo,
    ) -> Result<(), ERROR> {
        unimplemented!("Please implement handle_get_characteristic_info to retrieve characteristic information")
    }

    /// Handle configure profile
    ///
    /// Default implementation that handles standard BLE profiles:
    /// - Custom: Uses pre-configured services/characteristics, just restarts server
    /// - HeartRateMonitor: Standard Heart Rate Service (0x180D)
    /// - BatteryService: Standard Battery Service (0x180F)
    /// - DeviceInformation: Standard Device Information Service (0x180A)
    /// - EnvironmentalSensing: Environmental Sensing Service (0x181A)
    /// - ProximityProfile: Proximity Profile (0x1802/0x1803/0x1804)
    /// - HealthThermometer: Health Thermometer Service (0x1809)
    /// - CyclingSpeedAndCadence: Cycling Speed and Cadence Service (0x1816)
    /// - CurrentTimeService: Current Time Service (0x1805)
    ///
    /// Implementations must provide:
    /// - `restart_server_with_profile()` to restart the BLE server
    /// - `handle_unknown_profile()` for error handling
    fn handle_configure_profile(&mut self, cmd: HostCommandConfigureProfile) -> Result<(), ERROR> {
        // Match on the profile type
        match BleProfile::try_from(cmd.profile) {
            Ok(BleProfile::Custom) => {
                // Custom profile is already configured via prior configure_service
                // and configure_characteristic commands. Just restart the server.
                return self.restart_server_with_profile(cmd.save_on_disconnect);
            }
            Ok(BleProfile::HeartRateMonitor) => {
                let profile_def = profiles::heart_rate::heart_rate_profile();
                self.apply_profile_definition(profile_def, cmd.save_on_disconnect)?;
            }
            Ok(BleProfile::BatteryService) => {
                let profile_def = profiles::battery_service::battery_service_profile();
                self.apply_profile_definition(profile_def, cmd.save_on_disconnect)?;
            }
            Ok(BleProfile::DeviceInformation) => {
                let profile_def = profiles::device_info::device_info_profile();
                self.apply_profile_definition(profile_def, cmd.save_on_disconnect)?;
            }
            Ok(BleProfile::EnvironmentalSensing) => {
                let profile_def = profiles::environmental_sensing::environmental_sensing_profile();
                self.apply_profile_definition(profile_def, cmd.save_on_disconnect)?;
            }
            Ok(BleProfile::ProximityProfile) => {
                let profile_def = profiles::proximity::proximity_profile();
                self.apply_profile_definition(profile_def, cmd.save_on_disconnect)?;
            }
            Ok(BleProfile::HealthThermometer) => {
                let profile_def = profiles::health_thermometer::health_thermometer_profile();
                self.apply_profile_definition(profile_def, cmd.save_on_disconnect)?;
            }
            Ok(BleProfile::CyclingSpeedAndCadence) => {
                let profile_def = profiles::cycling_speed_cadence::cycling_speed_cadence_profile();
                self.apply_profile_definition(profile_def, cmd.save_on_disconnect)?;
            }
            Ok(BleProfile::CurrentTimeService) => {
                let profile_def = profiles::current_time::current_time_profile();
                self.apply_profile_definition(profile_def, cmd.save_on_disconnect)?;
            }
            Ok(BleProfile::Unspecified) | Err(_) => {
                return self.handle_unknown_profile();
            }
        }

        Ok(())
    }

    /// Apply a profile definition by configuring its services and characteristics.
    ///
    /// This helper method iterates through the profile's services and characteristics,
    /// calling the appropriate handler methods to configure the BLE stack.
    ///
    /// # Arguments
    /// * `profile` - The profile definition to apply
    /// * `save_on_disconnect` - Whether to save the profile configuration to NVS
    ///
    /// # Returns
    /// Result indicating success or failure
    fn apply_profile_definition(
        &mut self,
        profile: profiles::ProfileDefinition,
        save_on_disconnect: bool,
    ) -> Result<(), ERROR> {
        // Configure each service and its characteristics
        for service in profile.services {
            // Configure the service
            self.handle_configure_service(HostCommandConfigureService {
                uuid: service.uuid as u32,
            })?;

            // Configure each characteristic in the service
            for characteristic in service.characteristics {
                self.handle_configure_characteristic(HostCommandConfigureCharacteristic {
                    uuid: characteristic.uuid as u32,
                    service_uuid: service.uuid as u32,
                    properties: characteristic.properties,
                })?;

                // If the characteristic has a default value, set it
                if let Some(default_value) = characteristic.default_value {
                    self.handle_configure_characteristic_read(
                        HostCommandConfigureCharacteristicRead {
                            uuid: characteristic.uuid as u32,
                            service_uuid: service.uuid as u32,
                            value: default_value,
                        },
                    )?;
                }
            }
        }

        // Restart the server with the new profile configuration
        self.restart_server_with_profile(save_on_disconnect)?;

        Ok(())
    }

    /// Restart the BLE server with the configured profile.
    ///
    /// This method should restart the BLE server to apply the new profile configuration.
    /// Implementations may also handle NVS persistence if `save_on_disconnect` is true.
    ///
    /// # Arguments
    /// * `save_on_disconnect` - Whether to save the profile configuration to NVS
    ///
    /// # Returns
    /// Result indicating success or failure
    fn restart_server_with_profile(&mut self, _save_on_disconnect: bool) -> Result<(), ERROR> {
        unimplemented!("Please implement restart_server_with_profile to restart the BLE server")
    }

    /// Handle unknown or unspecified profile.
    ///
    /// This method is called when an unknown profile ID is received.
    /// Implementations should return an appropriate error.
    ///
    /// # Returns
    /// Result indicating failure with appropriate error
    fn handle_unknown_profile(&mut self) -> Result<(), ERROR> {
        unimplemented!("Please implement handle_unknown_profile to handle unknown profile errors")
    }
}

/// Enum representing the possible states of the blink indication
pub enum BlinkState {
    /// Indicates a successful operation
    Success,
    /// Indicates a failure or error condition
    Failure,
}

/// Trait for hardware accessories like blinking
pub trait HardwareAccessories {
    /// Blink
    fn blink(&mut self, _state: BlinkState) {
        unimplemented!("Implement blink to allow blinking")
    }
}
