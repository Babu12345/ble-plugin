//! Provides non-volatile storage functionality for the BLE plugin system.
#![deny(missing_docs)]

use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault, NvsPartitionId};

use crate::{error::Result, namespaces::NvsNamespaceTrait};
pub mod error;
pub mod keys;
pub mod namespaces;

/// Type alias for the default NVS partition.
pub type EspNvsDefaultPartition = EspNvsPartition<NvsDefault>;
/// Type alias for the default NVS handle.
pub type EspNvsDefault = EspNvs<NvsDefault>;

/// Configures and returns an NVS handle for the specified namespace.
pub fn namespace<P: NvsPartitionId, T: NvsNamespaceTrait<P>>(nvs: EspNvsPartition<P>) -> Result<T> {
    let nvs = match EspNvs::new(nvs, T::as_str(), true) {
        Ok(nvs) => Ok(nvs),
        Err(_) => return Err(error::PluginNvcError::NamespaceAcquisitionError),
    }?;

    Ok(T::new(nvs))
}
