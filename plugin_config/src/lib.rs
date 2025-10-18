#![deny(missing_docs)]
//! Library that contains hardware agnostic methods for the plugin hardware to be used in the state machine

use std::fmt::Debug;

pub use protocol::plugin::*;
pub use protocol::protocol::*;
pub use protocol::utils::*;
pub use protocol::DEFAULT_PACKET_SIZE;
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
    fn handle_configure_profile(&mut self, _cmd: HostCommandConfigureProfile) -> Result<(), ERROR> {
        unimplemented!("Please implement handle_configure_profile to configure BLE profiles")
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
