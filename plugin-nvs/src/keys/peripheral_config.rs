// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Key for the "peripheral_config" configuration setting in NVS.

use esp_idf_svc::nvs::{EspNvs, NvsPartitionId};

use crate::{
    error::{self, Result},
    namespaces::NvsKeyTrait,
};

/// Key for the "peripheral_config" configuration setting.
pub struct PeripheralConfigurationKey<'a, T>
where
    T: NvsPartitionId,
{
    nvs: &'a mut EspNvs<T>,
}

impl<'a, T> NvsKeyTrait<'a, T> for PeripheralConfigurationKey<'a, T>
where
    T: NvsPartitionId,
{
    fn as_str() -> &'static str {
        "peripheral_config"
    }

    fn new(nvs: &'a mut EspNvs<T>) -> Self {
        Self { nvs }
    }
}

impl<'a, T> PeripheralConfigurationKey<'a, T>
where
    T: NvsPartitionId,
{
    /// Reads the "peripheral_config" value from NVS.
    pub fn read(&self, buffer: &'a mut [u8]) -> Result<Option<&[u8]>> {
        match self.nvs.get_raw(Self::as_str(), buffer) {
            Ok(value) => Ok(value),
            Err(_) => Err(error::PluginNvcError::NvsReadError),
        }
    }

    /// Writes the "peripheral_config" value to NVS.
    pub fn write(&mut self, buffer: &'a [u8]) -> Result<()> {
        match self.nvs.set_raw(Self::as_str(), buffer) {
            Ok(_) => Ok(()),
            Err(_) => Err(error::PluginNvcError::NvsWriteError),
        }
    }
}
