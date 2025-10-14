#![deny(missing_docs)]
//! Library that contains hardware agnostic methods for the plugin hardware to be used in the state machine

/// Hardware agnostic plugin configurator
pub trait PluginConfig<ERROR> {
    /// Handle peripheral configuration
    fn handle_configure_peripheral() -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle peripheral security configuration
    fn handle_configure_peripheral_security() -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle start advertisement
    fn handle_start_advertisement() -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle stop advertisement
    fn handle_stop_advertisement() -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle service configuration
    fn handle_configure_service() -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle characteristic configuration
    fn handle_configure_characteristic() -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle characteristic read configuration
    fn handle_configure_characteristic_read() -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle notify characteristic value
    fn handle_notify_characteristic_value() -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle get service info
    fn handle_get_service_info() -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle get characteristic info
    fn handle_get_characteristic_info() -> Result<(), ERROR> {
        unimplemented!()
    }

    /// Handle configure profile
    fn handle_configure_profile() -> Result<(), ERROR> {
        unimplemented!()
    }
}
