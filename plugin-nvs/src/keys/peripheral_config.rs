//! Key for the "peripheral_config" configuration setting in NVS.

use crate::{
    EspNvsDefault,
    error::{self, Result},
    namespaces::NvsKeyTrait,
};

/// Key for the "peripheral_config" configuration setting.
pub struct PeripheralConfigurationKey<'a> {
    nvs: &'a mut EspNvsDefault,
}

impl<'a> NvsKeyTrait<'a> for PeripheralConfigurationKey<'a> {
    fn as_str() -> &'static str {
        return "peripheral_config";
    }

    fn new(nvs: &'a mut EspNvsDefault) -> Self {
        Self { nvs }
    }
}

impl<'a> PeripheralConfigurationKey<'a> {
    /// Reads the "peripheral_config" value from NVS.
    pub fn read(&self, buffer: &'a mut [u8]) -> Result<Option<&[u8]>> {
        match self.nvs.get_blob(Self::as_str(), buffer) {
            Ok(value) => Ok(value),
            Err(_) => Err(error::PluginNvcError::NvsReadError),
        }
    }

    /// Writes the "peripheral_config" value to NVS.
    pub fn write(&mut self, buffer: &'a [u8]) -> Result<()> {
        match self.nvs.set_blob(Self::as_str(), buffer) {
            Ok(_) => Ok(()),
            Err(_) => Err(error::PluginNvcError::NvsWriteError),
        }
    }
}
