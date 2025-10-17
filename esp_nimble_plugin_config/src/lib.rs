#![deny(missing_docs)]
//! Implements the plugin_config to be used in the plugin state machines for esp_nimble. Which is a bluetooth crate for esp32

pub mod errors;
use std::collections::HashMap;

use esp32_nimble::{enums::OwnAddrType, BLEDevice, BLEServer};
use esp_idf_svc::nvs::EspNvsPartition;
use plugin_config::{slice_to_array, HostCommandConfigurePeripheral, PluginConfig};
use plugin_nvs::{namespace, namespaces::ConfigNamespace};
mod utils;
use crate::errors::{Error, Result};
use esp_idf_svc::nvs::NvsPartitionId;

/// Maximum number of characteristics per service
const MAX_CHARACTERISTICS_PER_SERVICE: usize = 16;

/// Max string size
const MAX_STRING_SIZE: usize = 30;

/// Stores processing metadata
#[derive(Default)]
struct Metadata {
    ble_name: Option<heapless::String<MAX_STRING_SIZE>>,
    /// The maximum size of the data that we can send via the plugin data type
    /// If the size is greater than `max_plugin_data_send_size` then we perform automatic chunking
    max_plugin_data_send_size: u16,
    /// Mapping from service UUIDs to their characteristic UUIDs and properties
    ///
    /// This enables efficient lookup of characteristics within services and
    /// provides quick access to characteristic properties for validation.
    service_to_characteristic_uuids:
        HashMap<u16, heapless::Vec<(u16, Vec<i32>), MAX_CHARACTERISTICS_PER_SERVICE>>, // (UUID, properties)
}

impl Metadata {
    fn set_name_local(&mut self, name: heapless::String<MAX_STRING_SIZE>) {
        self.ble_name = Some(name);
    }
    /// Set the BLE device name for advertising
    /// Also persists the name to NVS storage
    fn set_name<T>(&mut self, ns: &mut ConfigNamespace<T>, name: heapless::String<MAX_STRING_SIZE>)
    where
        T: NvsPartitionId,
    {
        self.set_name_local(name.clone());
        ns.name_config_key()
            .write(name.as_bytes())
            .map_err(|e| {
                log::error!("Failed to write name to NVS: {:?}", e);
                Error::NvsWriteError
            })
            .ok();
    }

    /// Set the maximum plugin data send size
    fn set_max_plugin_data_send_size(&mut self, max_size: u16) {
        self.max_plugin_data_send_size = max_size;
    }
}

/// Nimble struct
pub struct EspNimble<'a, T>
where
    T: NvsPartitionId,
{
    is_initialized: bool,
    device: &'a mut BLEDevice,
    /// Optional BLE server instance (created after peripheral configuration)
    server: Option<&'static mut BLEServer>,
    metadata: Metadata,
    /// NVS namespace for persistent configuration storage
    ///
    /// This provides access to the Non-Volatile Storage partition where device
    /// configurations are persisted across power cycles.
    ns: ConfigNamespace<T>,
}

impl<'a, T> EspNimble<'a, T>
where
    T: NvsPartitionId,
{
    /// Create a new instance
    pub fn new(nvs_partition: EspNvsPartition<T>) -> Result<Self> {
        Ok(Self {
            is_initialized: false,
            device: BLEDevice::take(),
            metadata: Default::default(),
            server: None,
            ns: namespace::<T, ConfigNamespace<T>>(nvs_partition)
                .map_err(|_| Error::FailedToResolveNvsNamespace)?,
        })
    }
    /// Initialize the struct
    pub fn initialize(&mut self) {
        self.is_initialized = true
    }

    /// Helper function to clear all services and associated metadata atomically
    fn clear_all_services_and_metadata(&mut self) {
        // Clear services on the server first, then clear metadata to ensure consistency
        if let Some(server) = self.server.as_mut() {
            server.clear_services();
        }

        // Clear metadata after server operation to keep them synchronized
        self.metadata.service_to_characteristic_uuids.clear();
    }
}

impl<'a, T> PluginConfig<Error> for EspNimble<'a, T>
where
    T: NvsPartitionId,
{
    fn handle_configure_peripheral(&mut self, cmd: HostCommandConfigurePeripheral) -> Result<()> {
        log::info!(
            "Configuring peripheral with name: '{}', address: {:?}",
            cmd.name,
            cmd.addr
        );

        // If we haven't already initialized then we can set the BLE device address
        // otherwise we cannot without resetting the BLE device
        if !self.is_initialized {
            self.initialize();
            self.device.set_own_addr_type(OwnAddrType::Random);
            let addr = slice_to_array(cmd.addr.as_slice()).map_err(|_| {
                log::error!("Invalid address length: must be 6 bytes");
                Error::InvalidBleConfiguration
            })?;
            self.device.set_rnd_addr(addr).map_err(|_| {
                log::error!("Failed to set random address for BLE device");
                Error::UnableToSetRNDAddress
            })?;
        }

        let name: heapless::String<30> = heapless::String::try_from(cmd.name.as_str())
            .map_err(|_| Error::InvalidBleConfiguration)?;
        self.metadata.set_name(&mut self.ns, name);
        self.metadata.set_max_plugin_data_send_size(32u16);
        self.server = Some(
            self.device
                .get_server()
                .advertise_on_disconnect(false)
                .clear_services(),
        );

        self.clear_all_services_and_metadata();
        log::info!("Successfully configured peripheral '{}'", cmd.name);
        Ok(())
    }
}
