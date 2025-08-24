//! Contains the namespaces used for NVS storage in the plugin.
use crate::{EspNvsDefault, keys::peripheral_config::PeripheralConfigurationKey};

/// Defines the available NVS namespaces.
pub trait NvsNamespaceTrait {
    /// Returns the string representation of the namespace.
    /// DO NOT CHANGE THIS VALUE ONCE DEPLOYED!
    fn as_str() -> &'static str;
    /// Creates a new instance of the namespace with the provided NVS handle.
    fn new(nvs: EspNvsDefault) -> Self;
}

/// Defines the keys used in the Config namespace.
pub trait NvsKeyTrait<'a> {
    /// Returns the string representation of the key.
    /// DO NOT CHANGE THIS VALUE ONCE DEPLOYED!
    fn as_str() -> &'static str;

    /// Creates a new instance of the namespace with the provided NVS handle.
    fn new(nvs: &'a mut EspNvsDefault) -> Self;
}

/// Configuration namespace.
pub struct ConfigNamespace {
    nvs: EspNvsDefault,
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
impl ConfigNamespace {
    /// Returns the "is_on" key for the Config namespace.
    pub fn peripheral_config_key(&mut self) -> PeripheralConfigurationKey {
        PeripheralConfigurationKey::new(&mut self.nvs)
    }
}
