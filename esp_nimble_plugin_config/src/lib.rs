#![deny(missing_docs)]
//! Implements the plugin_config to be used in the plugin state machines for esp_nimble. Which is a bluetooth crate for esp32

pub mod errors;
use std::{collections::HashMap, sync::Arc, time::Duration};

use esp32_nimble::{
    enums::{AuthReq, OwnAddrType, SecurityIOCap},
    utilities::BleUuid,
    BLEAddress, BLEDevice, BLEServer, BLEService, NimbleProperties,
};
use esp_idf_svc::hal::gpio::AnyOutputPin;
use esp_idf_svc::hal::gpio::Output;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::{
    hal::task::block_on, nvs::EspNvsPartition, sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS,
};
use plugin_config::{
    plugin::PluginSender, slice_to_array, BleProfile, BleProperties, BluetoothAddressType,
    HardwareAccessories, HostCommandConfigureCharacteristic,
    HostCommandConfigureCharacteristicRead, HostCommandConfigurePeripheral,
    HostCommandConfigurePeripheralSecurity, HostCommandConfigureProfile,
    HostCommandConfigureService, HostCommandGetCharacteristicInfo, HostCommandGetServiceInfo,
    HostCommandNotifyCharacteristicValue, HostCommandStartAdvertisement,
    HostCommandStopAdvertisement, PluginAuthenticationCompletedResponse,
    PluginCharacteristicInfoResponse, PluginConfig, PluginConfigurationError,
    PluginConfigurationErrorType, PluginData, PluginDataSendType, PluginOnConnectResponse,
    PluginServiceInfoResponse, DEFAULT_PACKET_SIZE,
};
use plugin_nvs::{namespace, namespaces::ConfigNamespace};

use std::sync::Mutex;
use threadpool::ThreadPool;
use throttle::Throttle;
mod utils;
use crate::{
    errors::{Error, Result},
    utils::{
        ble_address_type_to_bluetooth_address_type, bluetooth_address_type_to_ble_address_type,
        send_plugin_data_chunked, set_device_name,
    },
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

    /// Get the BLE device name, initializing from NVS if not already set
    fn get_or_init_name<T>(
        &mut self,
        ns: &mut ConfigNamespace<T>,
    ) -> Option<heapless::String<MAX_STRING_SIZE>>
    where
        T: NvsPartitionId,
    {
        if let Some(name) = &self.ble_name {
            return Some(name.clone());
        }

        let mut buffer = [0u8; MAX_STRING_SIZE];
        match ns.name_config_key().read(&mut buffer) {
            Ok(data) => {
                if data?.len() > MAX_STRING_SIZE {
                    log::error!(
                        "Stored name in NVS exceeds maximum length of {} bytes",
                        MAX_STRING_SIZE
                    );
                    return None;
                }
                let mut name: heapless::String<MAX_STRING_SIZE> = heapless::String::new();
                name.push_str(core::str::from_utf8(&data?).ok()?).ok()?;
                self.set_name_local(name.clone());
                return Some(name);
            }
            Err(e) => {
                log::error!("Failed to read name from NVS: {:?}", e);
                None
            }
        }
    }
}

/// Nimble struct
pub struct EspNimblePluginConfig<'a, T>
where
    T: NvsPartitionId,
{
    is_initialized: bool,
    device: &'a mut BLEDevice,
    /// Thread-safe USB sender for responses and BLE data forwarding
    sender: Arc<PluginSender<DEFAULT_PACKET_SIZE>>,

    /// Optional BLE server instance (created after peripheral configuration)
    server: Option<&'static mut BLEServer>,
    metadata: Metadata,
    /// NVS namespace for persistent configuration storage
    ///
    /// This provides access to the Non-Volatile Storage partition where device
    /// configurations are persisted across power cycles.
    ns: ConfigNamespace<T>,
}

impl<'a, T> EspNimblePluginConfig<'a, T>
where
    T: NvsPartitionId,
{
    /// Create a new instance
    pub fn new(
        sender: PluginSender<DEFAULT_PACKET_SIZE>,
        nvs_partition: EspNvsPartition<T>,
    ) -> Result<Self> {
        Ok(Self {
            sender: Arc::new(sender),
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

    /// Get all configured service UUIDs
    pub fn get_service_uuids(&self) -> heapless::Vec<u16, 16> {
        self.metadata
            .service_to_characteristic_uuids
            .keys()
            .cloned()
            .collect()
    }
}

impl<'a, T> PluginConfig<Error> for EspNimblePluginConfig<'a, T>
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

    fn handle_configure_characteristic_read(
        &mut self,
        cmd: HostCommandConfigureCharacteristicRead,
    ) -> Result<()> {
        log::info!(
            "Configuring BLE characteristic with UUID: {} for service: {} with read value: {:?}",
            cmd.uuid,
            cmd.service_uuid,
            cmd.value
        );

        // Get the service that this characteristic belongs to
        let service = self
            .get_service(cmd.service_uuid as u16)
            .ok_or_else(|| {
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
            })?
            .lock();

        let characteristic = block_on(service.get_characteristic(BleUuid::Uuid16(cmd.uuid as u16)))
            .ok_or_else(|| Error::InvalidBleConfiguration)?;

        characteristic.lock().set_value(cmd.value.as_slice());
        Ok(())
    }

    fn handle_notify_characteristic_value(
        &mut self,
        cmd: HostCommandNotifyCharacteristicValue,
    ) -> Result<()> {
        log::info!(
            "Notifying characteristic {} in service {} with {} bytes",
            cmd.characteristic_uuid,
            cmd.service_uuid,
            cmd.value.len()
        );

        // Get the service that this characteristic belongs to
        let service = self
            .get_service(cmd.service_uuid as u16)
            .ok_or_else(|| {
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
            })?
            .lock();

        // Get the characteristic
        let characteristic =
            block_on(service.get_characteristic(BleUuid::Uuid16(cmd.characteristic_uuid as u16)))
                .ok_or_else(|| {
                log::error!(
                    "Characteristic with UUID {} not found in service {}",
                    cmd.characteristic_uuid,
                    cmd.service_uuid
                );
                Error::InvalidBleConfiguration
            })?;

        // Get the characteristic
        let characteristic_lock = characteristic.lock();

        match self.server.as_ref() {
            Some(server) => {
                let conn = server
                    .connections()
                    .find(|desc| {
                        if let Ok(val) = slice_to_array(cmd.address.as_slice()) {
                            if let Some(addr_type) =
                                BluetoothAddressType::try_from(cmd.address_type).ok()
                            {
                                return desc.address()
                                    == BLEAddress::from_be_bytes(
                                        val,
                                        bluetooth_address_type_to_ble_address_type(addr_type),
                                    );
                            }
                        }
                        return false;
                    })
                    .ok_or_else(|| {
                        log::error!(
                            "Connection with address {:?} and type {:?} not found",
                            cmd.address,
                            cmd.address_type
                        );
                        Error::InvalidBleConfiguration
                    })?;

                characteristic_lock
                    .notify_with(cmd.value.as_slice(), conn.conn_handle())
                    .map_err(|e| {
                        log::error!(
                            "Failed to notify characteristic {} in service {}: {:?}",
                            cmd.characteristic_uuid,
                            cmd.service_uuid,
                            e
                        );
                        Error::CharacteristicNotificationError
                    })?;
            }
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
        }

        log::info!(
            "Successfully notified characteristic {} with value: {:?}",
            cmd.characteristic_uuid,
            cmd.value.as_slice()
        );

        Ok(())
    }

    fn handle_get_service_info(&mut self, cmd: HostCommandGetServiceInfo) -> Result<()> {
        log::info!("Processing get service info command for UUID: {}", cmd.uuid);

        let characteristic_uuids = self
            .metadata
            .service_to_characteristic_uuids
            .get(&(cmd.uuid as u16))
            .map(|chars| {
                let mut uuids = Vec::new();
                for (uuid, _properties) in chars {
                    uuids.push(*uuid as u32);
                }
                uuids
            })
            .unwrap_or_else(|| {
                log::warn!("No characteristics found for service {}", cmd.uuid);
                Vec::new()
            });

        let response = PluginServiceInfoResponse {
            service_uuid: cmd.uuid,
            characteristic_uuids,
            exists: self.get_service(cmd.uuid as u16).is_some(),
        };

        // Send the response to USB
        self.sender.send(response).map_err(|_| {
            log::error!("Failed to send service info response to USB");
            Error::UsbSendError
        })?;

        log::info!(
            "Successfully sent service info response for UUID: {}",
            cmd.uuid
        );
        Ok(())
    }

    fn handle_get_characteristic_info(
        &mut self,
        cmd: HostCommandGetCharacteristicInfo,
    ) -> Result<()> {
        log::info!(
            "Processing get characteristic info command for characteristic {} in service {}",
            cmd.characteristic_uuid,
            cmd.service_uuid
        );

        // Look for the characteristic in the specified service
        let (exists, properties) = self
            .metadata
            .service_to_characteristic_uuids
            .get(&(cmd.service_uuid as u16))
            .and_then(|chars| {
                chars.iter().find_map(|(uuid, properties)| {
                    if *uuid == (cmd.characteristic_uuid as u16) {
                        Some((true, properties.clone()))
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| {
                log::warn!(
                    "Characteristic {} not found in service {}",
                    cmd.characteristic_uuid,
                    cmd.service_uuid
                );

                (false, Vec::new())
            });

        let properties = properties.into_iter().map(|x| x.into()).collect();
        let response = PluginCharacteristicInfoResponse {
            characteristic_uuid: cmd.characteristic_uuid,
            service_uuid: cmd.service_uuid,
            properties,
            exists,
        };

        // Send the response to USB
        self.sender.send(response).map_err(|_| {
            log::error!("Failed to send characteristic info response to USB");
            Error::UsbSendError
        })?;

        log::info!(
            "Successfully sent characteristic info response for characteristic {} in service {}",
            cmd.characteristic_uuid,
            cmd.service_uuid
        );
        Ok(())
    }

    fn handle_start_advertisement(&mut self, cmd: HostCommandStartAdvertisement) -> Result<()> {
        let advertisement = self.device.get_advertising();
        log::info!(
            "Starting BLE advertisement, multi-connect: {}",
            cmd.allow_multi_connect
        );

        // Note: On the first call, this will auto-configure using any predefined profile settings.
        // Subsequent calls require explicit configuration via configure_profile() or manual service setup.
        match self.metadata.get_or_init_name(&mut self.ns).as_ref() {
            Some(name) => {
                let mut adv_data_base = esp32_nimble::BLEAdvertisementData::new();
                let adv_data = adv_data_base.name(name.as_str());

                // Get all service UUIDs to include in advertisement
                for uuid in self.get_service_uuids().into_iter() {
                    adv_data.add_service_uuid(BleUuid::from_uuid16(uuid));
                }

                advertisement.lock().set_data(adv_data).map_err(|e| {
                    log::error!("Failed to set advertisement data: {:?}", e);
                    Error::AdvertisementError("Failed to start advertisement")
                })?;
                advertisement.lock().start().map_err(|e| {
                    log::error!("Failed to start advertisement: {:?}", e);
                    Error::AdvertisementError("Failed to start advertisement")
                })?;
                log::info!("Started BLE advertisement with name: {name}");
            }
            None => {
                log::error!(
                    "Error: Received advertisement command without peripheral configuration"
                );
                self.sender
                    .send(PluginConfigurationError { error_type: PluginConfigurationErrorType::AdvertisementWithoutPeripheralConfiguration as _ })
                    .map_err(|_| Error::UsbSendError)?;
                return Err(Error::InvalidBleConfiguration);
            }
        }

        match self.server.as_mut() {
            Some(server) => {
                let sender = self.sender.clone();
                server.on_connect(move |server, desc| {
                    log::info!("Client connected: {:?}", desc);

                    let addr = desc.address();
                    let response = PluginOnConnectResponse {
                        address: addr.as_be_bytes().to_vec(),
                        address_type: ble_address_type_to_bluetooth_address_type(addr.addr_type())
                            as _,
                    };
                    sender.send(response).ok();
                    if cmd.allow_multi_connect
                        && server.connected_count() < (CONFIG_BT_NIMBLE_MAX_CONNECTIONS as usize)
                    {
                        log::info!("Multi-connect support: start advertising");
                        if let Err(e) = advertisement.lock().start() {
                            log::error!(
                                "Failed to restart advertisement for multi-connect: {:?}",
                                e
                            );
                        }
                    }
                });

                server.on_disconnect(move |_desc, reason| {
                    log::info!("Client disconnected ({:?})", reason);
                });

                let sender = self.sender.clone();
                server.on_authentication_complete(move |_, desc, status| {
                    log::info!("Authentication completed for client: {:?}", desc);
                    let addr = desc.address().as_be_bytes();
                    let response = PluginAuthenticationCompletedResponse {
                        address: addr.to_vec(),
                        address_type: ble_address_type_to_bluetooth_address_type(
                            desc.address().addr_type(),
                        ) as _,
                        success: status.is_ok(),
                    };
                    sender
                        .send(response)
                        .map_err(|e| {
                            log::error!("Failed to send authentication response: {:?}", e);
                            Error::UsbSendError
                        })
                        .ok();
                });
                log::info!("Successfully configured BLE server callbacks");
            }
            None => {
                log::error!("Error: Server not initialized for BLE device");
                return Err(Error::ServerNotInitialized);
            }
        }
        Ok(())
    }

    fn handle_configure_peripheral_security(
        &mut self,
        cmd: HostCommandConfigurePeripheralSecurity,
    ) -> Result<()> {
        log::debug!("Setting up BLE security configuration");

        if cmd.passkey > 999999 {
            log::error!("Invalid passkey: must be a 6-digit number");
            return Err(Error::InvalidPasskeyLength);
        }

        self.device
            .security()
            .set_auth(AuthReq::all())
            .set_passkey(cmd.passkey)
            .set_io_cap(SecurityIOCap::DisplayOnly)
            .resolve_rpa();

        Ok(())
    }

    fn handle_configure_profile(&mut self, cmd: HostCommandConfigureProfile) -> Result<()> {
        // Update the device name during the profile configuration.
        if let Some(name) = self.metadata.get_or_init_name(&mut self.ns) {
            set_device_name(name.as_str());
            log::info!("Configured device name")
        }

        log::info!("Configuring BLE profile: {:?}", cmd.profile);

        match BleProfile::try_from(cmd.profile) {
            Ok(BleProfile::Custom) => {
                log::info!("Using custom profile with predefined services and characteristics");
                // Get the server
                let server = match self.server.as_mut() {
                    Some(server) => server,
                    None => {
                        log::error!("No BLE server available. Configure peripheral first.");
                        return Err(Error::InvalidBleConfiguration);
                    }
                };

                // Restart the server with all predefined services and characteristics
                server.restart(true).map_err(|source| {
                    log::error!("Failed to restart BLE server: {:?}", source);
                    Error::ServerRestartError(source)
                })?;
            }
            Ok(other_profile) => {
                log::warn!(
                    "Predefined BLE profile {:?} is not implemented yet",
                    other_profile
                );
            }
            Err(_) => {
                log::error!("Unknown BLE profile ID: {:?}", cmd.profile);
                return Err(Error::InvalidBleConfiguration);
            }
        }

        log::info!("Successfully configured profile {:?} by restarting server with predefined configuration", cmd.profile);
        Ok(())
    }

    fn handle_stop_advertisement(&mut self, _cmd: HostCommandStopAdvertisement) -> Result<()> {
        log::info!("Stopping BLE advertisement");

        self.device.get_advertising().lock().stop().map_err(|e| {
            log::error!("Failed to stop advertisement: {:?}", e);
            Error::AdvertisementError("Failed to stop advertisement")
        })?;

        log::info!("Successfully stopped BLE advertisement");
        Ok(())
    }
}

/// Esp32's hardware accessories
pub struct EspHardwareAccessories {
    /// Pin indicator
    indicator: Arc<Mutex<PinDriver<'static, AnyOutputPin, Output>>>,
    /// Throttle for blink indication to prevent excessive blinking
    /// and errors
    blink_throttle: Throttle,
    /// Thread pool for managing blink operations
    blink_thread_pool: ThreadPool,
}

impl EspHardwareAccessories {
    /// New instance
    pub fn new(indicator: Arc<Mutex<PinDriver<'static, AnyOutputPin, Output>>>) -> Self {
        Self {
            indicator,
            blink_throttle: Throttle::new(Duration::from_millis(500), 1),
            blink_thread_pool: ThreadPool::new(1),
        }
    }
}

impl HardwareAccessories for EspHardwareAccessories {
    fn blink(&mut self, state: plugin_config::BlinkState) {
        // Apply throttling
        match self.blink_throttle.accept() {
            Ok(_) => {}
            Err(_) => {
                log::debug!("Blink indication throttled");
                return;
            }
        }

        let indicator = self.indicator.clone();

        // Submit blink task to thread pool
        self.blink_thread_pool.execute(move || {
            for i in 0..4 {
                // Try to acquire lock non-blocking
                match indicator.try_lock() {
                    Ok(mut indicator) => {
                        if let Err(e) = {
                            match i % 2 {
                                0 => indicator.set_low(),
                                _ => indicator.set_high(),
                            }
                        } {
                            log::error!("Failed to toggle GPIO: {:?}", e);
                            return;
                        }
                    }
                    Err(_) => {
                        log::debug!("GPIO lock busy, skipping blink");
                        return;
                    }
                }

                // Sleep after releasing the lock
                match state {
                    plugin_config::BlinkState::Success => {
                        std::thread::sleep(Duration::from_millis(if i == 0 { 50 } else { 5 }));
                    }
                    plugin_config::BlinkState::Failure => {
                        std::thread::sleep(Duration::from_millis(40));
                    }
                }
            }
        });
    }
}
