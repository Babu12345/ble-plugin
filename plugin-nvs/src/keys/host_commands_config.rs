//! Key for "host_command_config" configuration settings in NVS which contains previous commands needed to reconfigure the peripheral

use esp_idf_svc::nvs::{EspNvs, NvsPartitionId};

use crate::{
    error::{self, Result},
    namespaces::NvsKeyTrait,
};

/// Key for the "host_commands_config" configuration setting.
pub struct HostCommandsConfigurationKey<'a, T>
where
    T: NvsPartitionId,
{
    nvs: &'a mut EspNvs<T>,
}

impl<'a, T> NvsKeyTrait<'a, T> for HostCommandsConfigurationKey<'a, T>
where
    T: NvsPartitionId,
{
    fn as_str() -> &'static str {
        "host_commands_config"
    }

    fn new(nvs: &'a mut EspNvs<T>) -> Self {
        Self { nvs }
    }
}

impl<'a, T> HostCommandsConfigurationKey<'a, T>
where
    T: NvsPartitionId,
{
    /// Reads the "host_commands_config" value from NVS.
    pub fn read(&self, buffer: &'a mut [u8]) -> Result<Option<&[u8]>> {
        match self.nvs.get_raw(Self::as_str(), buffer) {
            Ok(value) => Ok(value),
            Err(_) => Err(error::PluginNvcError::NvsReadError),
        }
    }

    /// Writes the "host_commands_config" value to NVS.
    pub fn write(&mut self, buffer: &'a [u8]) -> Result<()> {
        match self.nvs.set_raw(Self::as_str(), buffer) {
            Ok(_) => Ok(()),
            Err(_) => Err(error::PluginNvcError::NvsWriteError),
        }
    }
}
