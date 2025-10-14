#![deny(missing_docs)]
//! Library that contains hardware agnostic methods for the plugin hardware to be used in the state machine

use protocol::protocol::{
    HostCommandConfigureCharacteristic, HostCommandConfigureCharacteristicRead,
    HostCommandConfigurePeripheral, HostCommandConfigurePeripheralSecurity,
    HostCommandConfigureProfile, HostCommandConfigureService,
    HostCommandGetCharacteristicInfo, HostCommandGetServiceInfo,
    HostCommandNotifyCharacteristicValue, HostCommandStartAdvertisement,
    HostCommandStopAdvertisement,
};

/// Hardware agnostic plugin configurator
pub trait PluginConfig<ERROR> {
    /// Handle peripheral configuration
    fn handle_configure_peripheral(
        &mut self,
        _cmd: HostCommandConfigurePeripheral,
    ) -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle peripheral security configuration
    fn handle_configure_peripheral_security(
        &mut self,
        _cmd: HostCommandConfigurePeripheralSecurity,
    ) -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle start advertisement
    fn handle_start_advertisement(
        &mut self,
        _cmd: HostCommandStartAdvertisement,
    ) -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle stop advertisement
    fn handle_stop_advertisement(
        &mut self,
        _cmd: HostCommandStopAdvertisement,
    ) -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle service configuration
    fn handle_configure_service(&mut self, _cmd: HostCommandConfigureService) -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle characteristic configuration
    fn handle_configure_characteristic(
        &mut self,
        _cmd: HostCommandConfigureCharacteristic,
    ) -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle characteristic read configuration
    fn handle_configure_characteristic_read(
        &mut self,
        _cmd: HostCommandConfigureCharacteristicRead,
    ) -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle notify characteristic value
    fn handle_notify_characteristic_value(
        &mut self,
        _cmd: HostCommandNotifyCharacteristicValue,
    ) -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle get service info
    fn handle_get_service_info(&mut self, _cmd: HostCommandGetServiceInfo) -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle get characteristic info
    fn handle_get_characteristic_info(
        &mut self,
        _cmd: HostCommandGetCharacteristicInfo,
    ) -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle configure profile
    fn handle_configure_profile(&mut self, _cmd: HostCommandConfigureProfile) -> Result<(), ERROR> {
        unimplemented!()
    }
}
