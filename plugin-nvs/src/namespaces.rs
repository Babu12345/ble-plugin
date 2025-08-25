//! Contains the namespaces used for NVS storage in the plugin.
use esp_idf_svc::nvs::{EspNvs, NvsPartitionId};

use crate::keys::peripheral_config::PeripheralConfigurationKey;

/// Defines the available NVS namespaces.
pub trait NvsNamespaceTrait<T>
where
    T: NvsPartitionId,
{
    /// Returns the string representation of the namespace.
    /// DO NOT CHANGE THIS VALUE ONCE DEPLOYED!
    fn as_str() -> &'static str;
    /// Creates a new instance of the namespace with the provided NVS handle.
    fn new(nvs: EspNvs<T>) -> Self;
}

/// Defines the keys used in the Config namespace.
pub(crate) trait NvsKeyTrait<'a, T>
where
    T: NvsPartitionId,
{
    /// Returns the string representation of the key.
    /// DO NOT CHANGE THIS VALUE ONCE DEPLOYED!
    fn as_str() -> &'static str;

    /// Creates a new instance of the namespace with the provided NVS handle.
    fn new(nvs: &'a mut EspNvs<T>) -> Self;
}

/// Configuration namespace.
pub struct ConfigNamespace<T>
where
    T: NvsPartitionId,
{
    nvs: EspNvs<T>,
}

impl<T> NvsNamespaceTrait<T> for ConfigNamespace<T>
where
    T: NvsPartitionId,
{
    /// Creates a new instance of the Config namespace.
    fn new(nvs: EspNvs<T>) -> Self {
        Self { nvs }
    }

    fn as_str() -> &'static str {
        "config"
    }
}
impl<P> ConfigNamespace<P>
where
    P: NvsPartitionId,
{
    /// Returns the key struct for the config namespace
    fn key<'a, T: NvsKeyTrait<'a, P>>(&'a mut self) -> T {
        T::new(&mut self.nvs)
    }

    /// Returns the peripheral configuration key
    pub fn peripheral_config_key<'a>(&'a mut self) -> PeripheralConfigurationKey<'a, P> {
        self.key::<PeripheralConfigurationKey<'a, P>>()
    }
}
