#![deny(missing_docs)]
//! Implements the plugin_config to be used in the plugin state machines for esp_nimble. Which is a bluetooth crate for esp32

pub mod errors;
use std::{collections::HashMap, sync::Arc};

use esp32_nimble::{
    enums::OwnAddrType, utilities::BleUuid, BLEDevice, BLEServer, BLEService, NimbleProperties,
};
use esp_idf_svc::{hal::task::block_on, nvs::EspNvsPartition};
use plugin_config::{
    plugin::{PluginReceiver, PluginSender},
    slice_to_array, BleProperties, HostCommandConfigureCharacteristic,
    HostCommandConfigurePeripheral, HostCommandConfigureService, PluginConfig,
    PluginConfigurationError, PluginConfigurationErrorType, PluginData, PluginDataSendType,
    DEFAULT_PACKET_SIZE,
};
use plugin_nvs::{namespace, namespaces::ConfigNamespace};
mod utils;
use crate::{
    errors::{Error, Result},
    utils::{ble_address_type_to_bluetooth_address_type, send_plugin_data_chunked},
};
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

    /// Get the maximum plugin data send size
    fn get_max_plugin_data_send_size(&self) -> u16 {
        if self.max_plugin_data_send_size == 0 {
            32
        } else {
            self.max_plugin_data_send_size
        }
    }
}

/// Nimble struct
pub struct EspNimble<'a, T>
where
    T: NvsPartitionId,
{
    is_initialized: bool,
    device: &'a mut BLEDevice,
    /// Thread-safe USB sender for responses and BLE data forwarding
    sender: Arc<PluginSender<DEFAULT_PACKET_SIZE>>,

    /// USB receiver for incoming host commands (exclusive access)
    receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,
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
    pub fn new(
        sender: PluginSender<DEFAULT_PACKET_SIZE>,
        receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,
        nvs_partition: EspNvsPartition<T>,
    ) -> Result<Self> {
        Ok(Self {
            sender: Arc::new(sender),
            receiver,
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

    /// Get a stored BLE service by UUID for characteristic creation
    pub fn get_service(
        &self,
        service_uuid: u16,
    ) -> Option<&Arc<esp32_nimble::utilities::mutex::Mutex<BLEService>>> {
        match self.server.as_ref() {
            Some(server) => block_on(server.get_service(BleUuid::from_uuid16(service_uuid))),
            None => None,
        }
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

    fn handle_configure_service(&mut self, cmd: HostCommandConfigureService) -> Result<()> {
        log::info!("Configuring BLE service with UUID: {}", cmd.uuid,);

        let server = match self.server.as_mut() {
            Some(server) => server,
            None => {
                log::error!("BLE server not initialized - peripheral must be configured first");
                self.sender
                    .send(PluginConfigurationError {
                        error_type:
                            PluginConfigurationErrorType::ServiceWithoutPeripheralConfiguration
                                as _,
                    })
                    .map_err(|_| Error::UsbSendError)?;
                return Err(Error::ServerNotInitialized);
            }
        };

        // Create the BLE service converting from u16 to BleUuid
        server.create_service(BleUuid::from_uuid16(cmd.uuid as u16));

        log::info!("Successfully created BLE service with UUID: {}", cmd.uuid);

        // Create a serivce entry and clear any existing characteristics for this service
        self.metadata
            .service_to_characteristic_uuids
            .entry(cmd.uuid as u16)
            .or_default()
            .clear();

        Ok(())
    }

    fn handle_configure_characteristic(
        &mut self,
        cmd: HostCommandConfigureCharacteristic,
    ) -> Result<()> {
        log::info!(
            "Configuring BLE characteristic with UUID: {} for service: {} with properties: {:?}",
            cmd.uuid,
            cmd.service_uuid,
            cmd.properties
        );

        // Get the service that this characteristic belongs to
        let service = self.get_service(cmd.service_uuid as u16).ok_or_else(|| {
            log::error!(
                "Service with UUID {} not found - service must be configured first",
                cmd.service_uuid
            );
            self.sender
                .send(PluginConfigurationError {
                    error_type:
                        PluginConfigurationErrorType::CharacteristicWithoutServiceConfiguration
                            as _,
                })
                .ok();
            Error::InvalidBleConfiguration
        })?;

        // Convert UUID to BleUuid
        let ble_uuid = BleUuid::from_uuid16(cmd.uuid as u16);

        // Convert properties from u8 to NimbleProperties
        let mut nimble_properties = NimbleProperties::empty();
        if cmd.properties.contains(&(BleProperties::Read as _)) {
            nimble_properties |= NimbleProperties::READ;
        }
        if cmd.properties.contains(&(BleProperties::WriteRsp as _)) {
            nimble_properties |= NimbleProperties::WRITE;
        }
        if cmd.properties.contains(&(BleProperties::WriteNoRsp as _)) {
            nimble_properties |= NimbleProperties::WRITE_NO_RSP;
        }
        if cmd.properties.contains(&(BleProperties::Notify as _)) {
            nimble_properties |= NimbleProperties::NOTIFY;
        }
        if cmd.properties.contains(&(BleProperties::Indicate as _)) {
            nimble_properties |= NimbleProperties::INDICATE;
        }

        // Create the characteristic
        let characteristic = service
            .lock()
            .create_characteristic(ble_uuid, nimble_properties);

        // Only append the characteristic if it doesn't already exist for this service
        let characteristics = self
            .metadata
            .service_to_characteristic_uuids
            .entry(cmd.service_uuid as u16)
            .or_default();

        // Check if characteristic with this UUID already exists
        match characteristics
            .iter()
            .any(|(uuid, _)| *uuid == (cmd.uuid as u16))
        {
            true => log::info!(
                "Characteristic {} already exists for service {}, skipping",
                cmd.uuid,
                cmd.service_uuid
            ),
            false => {
                characteristics
                    .push((
                        cmd.uuid as u16,
                        cmd.properties.into_iter().map(|x| x as _).collect(),
                    ))
                    .map_err(|_| {
                        log::error!("Failed to store characteristic UUID: {}", cmd.uuid);
                        Error::CharacteristicUuidStorageError
                    })?;
            }
        }

        match nimble_properties.contains(NimbleProperties::WRITE)
            | nimble_properties.contains(NimbleProperties::WRITE_NO_RSP)
        {
            true => {
                let char_uuid_write = cmd.uuid;
                let service_uuid_write = cmd.service_uuid;
                let sender = self.sender.clone();
                let max_plugin_data_send_size = self.metadata.get_max_plugin_data_send_size();
                characteristic.lock().on_write(move |args| {
                    // Avoid logging on the write hot path
                    let plugin_data = PluginData {
                        src_addr: args.desc().address().as_be_bytes().as_ref().to_vec(),
                        src_addr_type: ble_address_type_to_bluetooth_address_type(
                            args.desc().address().addr_type(),
                        ) as _,
                        send_type: PluginDataSendType::WriteType as _,
                        characteristic_uuid: char_uuid_write,
                        service_uuid: service_uuid_write,
                        data: args.recv_data().to_vec(),
                    };
                    send_plugin_data_chunked(
                        sender.clone(),
                        plugin_data,
                        max_plugin_data_send_size as _,
                    )
                    .ok();
                });
            }
            false => {
                log::trace!(
                    "Characteristic {} does not support WRITE or WRITE_NO_RSP property",
                    cmd.uuid
                );
            }
        }

        match nimble_properties.contains(NimbleProperties::READ) {
            true => {
                let sender = self.sender.clone();
                let max_plugin_data_send_size = self.metadata.get_max_plugin_data_send_size();
                characteristic.lock().on_read(move |_, desc| {
                    log::info!(
                        "BLE read requested for characteristic {} in service {}",
                        cmd.uuid,
                        cmd.service_uuid
                    );

                    let plugin_data = PluginData {
                        src_addr: desc.address().as_be_bytes().as_ref().to_vec(),
                        src_addr_type: ble_address_type_to_bluetooth_address_type(
                            desc.address().addr_type(),
                        ) as _,
                        send_type: PluginDataSendType::ReadType as _,
                        characteristic_uuid: cmd.uuid,
                        service_uuid: cmd.service_uuid,
                        data: Vec::new(),
                    };
                    send_plugin_data_chunked(
                        sender.clone(),
                        plugin_data,
                        max_plugin_data_send_size as _,
                    )
                    .ok();
                });
            }
            false => {
                log::trace!("Characteristic {} does not support READ property", cmd.uuid);
            }
        };

        log::info!(
            "Successfully configured BLE characteristic with UUID: {} for service: {}",
            cmd.uuid,
            cmd.service_uuid
        );

        Ok(())
    }
}
