//! Contains the namespaces used for NVS storage in the plugin.
use crate::{
    EspNvsDefault,
    error::{self, Result},
};

/// Defines the available NVS namespaces.
pub trait NvsNamespaceTrait {
    /// Returns the string representation of the namespace.
    /// DO NOT CHANGE THIS VALUE ONCE DEPLOYED!
    fn as_str() -> &'static str;
    /// Creates a new instance of the namespace with the provided NVS handle.
    fn new(nvs: EspNvsDefault) -> Self;
}

/// Configuration namespace.
pub struct ConfigNamespace {
    nvs: EspNvsDefault,
}

impl ConfigNamespace {
    /// Gets the "is_on" configuration value.
    pub fn get_is_on(&self) -> bool {
        match self.nvs.get_i32("is_on") {
            Ok(value) => matches!(value, Some(1)),
            Err(_) => false,
        }
    }

    /// Sets the "is_on" configuration value.
    pub fn set_is_on(&mut self, is_on: bool) -> Result<()> {
        let value = if is_on { 1 } else { 0 };
        self.nvs
            .set_i32("is_on", value)
            .map_err(|_| error::PluginNvcError::NvsWriteError)?;
        Ok(())
    }
}

impl NvsNamespaceTrait for ConfigNamespace {
    /// Creates a new instance of the Config namespace.
    fn new(nvs: EspNvsDefault) -> Self {
        Self { nvs }
    }

    fn as_str() -> &'static str {
        return "config";
    }
}
