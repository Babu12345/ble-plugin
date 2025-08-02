#![deny(missing_docs)]
//! This library is used to to contain the complete processing logic and state machine to facilitate data/command transfer from BLE
//! to usb and visa versa.

pub mod errors;

use errors::Result;
use errors::StateMachineError;
use esp_idf_svc::hal::task::block_on;
use esp32_nimble::BLEAddress;
use esp32_nimble::BLEAddressType;
use protocol::io_types::BLEProperties;
use protocol::io_types::HostCommandConfigureCharacteristicRead;
use protocol::io_types::HostCommandNotifyCharacteristicValue;
use protocol::io_types::PluginData;

use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS;
use esp32_nimble::enums::{AuthReq, SecurityIOCap};
use esp32_nimble::utilities::BleUuid;
use esp32_nimble::{BLEDevice, BLEServer, BLEService, NimbleProperties};
use heapless::String;
use protocol::io_types::{
    HostCommandConfigureCharacteristic, HostCommandConfigurePeripheral,
    HostCommandConfigureService, HostCommandGetCharacteristicInfo, HostCommandGetServiceInfo,
    HostCommandStartAdvertisement, PluginCharacteristicInfoResponse, PluginConfigurationError, PluginServiceInfoResponse,
    MAX_PROPERTIES, MAX_CHARACTERISTICS_PER_SERVICE,
};
use protocol::plugin::plugin::{PluginReceiver, PluginSender};
use protocol::{DEFAULT_PACKET_SIZE, MAX_NAME_SIZE};

use std::sync::Arc;
use uuid::Uuid;
// This is used to store the metadata of the plugin state machine
#[derive(Default)]
struct PluginStateMachineMetadata {
    ble_name: Option<String<MAX_NAME_SIZE>>,
    service_to_characteristic_uuids: HashMap<Uuid, heapless::Vec<(Uuid, heapless::Vec<BLEProperties, MAX_PROPERTIES>), MAX_CHARACTERISTICS_PER_SERVICE>>, // (UUID, properties)
}

/// Contains state machine to process BLE and usb data and facilitate their data transfer
pub struct PluginStateMachine {
    usb_sender: Arc<PluginSender<DEFAULT_PACKET_SIZE>>,
    usb_receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,
    ble_device: &'static mut BLEDevice,
    server: Option<&'static mut BLEServer>,
    metadata: PluginStateMachineMetadata,
}

/// There will be 2 runners the first will be processing
/// usb data and sending it to BLE. The second will be processing
/// BLE data and sending it to USB.
impl PluginStateMachine {
    /// Create a new instance of the processing state machine
    pub fn new(
        usb_sender: PluginSender<DEFAULT_PACKET_SIZE>,
        usb_receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,
        ble_device: &'static mut BLEDevice,
    ) -> Self {
        Self {
            usb_sender: Arc::new(usb_sender),
            usb_receiver,
            ble_device,
            server: None,
            metadata: Default::default(),
        }
    }

    /// Returns a closure that can be used to run the state machine in a separate thread.
    pub fn runner_fn(mut self) -> impl FnMut() {
        move || {
            self.runner();
        }
    }

    /// USB-BLE bridge runner that processes bidirectional data transfer in a separate thread.
    ///
    /// Responsibilities:
    /// - Forwards USB commands/data to BLE device and vice versa
    /// - Configures BLE services, characteristics, and plugin settings based on USB commands
    /// - Handles BLE authentication and security requirements
    /// - Sets up BLE callback functions for BLE -> USB communication
    /// - Runs concurrently to avoid blocking the main thread
    ///
    /// TODO: Be smarter about decoding the usb data and sure that there are no collisions (meaning that the received data can be represented as > 1 commands)
    fn runner(&mut self) {
        log::info!("Starting USB-BLE bridge runner");
        loop {
            match self.usb_receiver.receive() {
                Ok(data) => {
                    log::debug!("Received USB data: {} bytes", data.size());

                    let maybe_cmd: Option<HostCommandConfigurePeripheral> = data.decode().ok();
                    if let Some(cmd) = maybe_cmd {
                        if let Err(e) = self.handle_configure_peripheral(cmd) {
                            log::error!("Failed to handle configure peripheral command: {:?}", e);
                        }
                        continue;
                    }

                    let maybe_cmd: Option<HostCommandConfigureService> = data.decode().ok();
                    if let Some(cmd) = maybe_cmd {
                        log::info!("Received USB command: {:?}", cmd);
                        if let Err(e) = self.handle_configure_service(cmd) {
                            log::error!("Failed to handle configure service command: {:?}", e);
                        }
                        continue;
                    }

                    let maybe_cmd: Option<HostCommandConfigureCharacteristic> = data.decode().ok();
                    if let Some(cmd) = maybe_cmd {
                        log::info!("Received USB command: {:?}", cmd);
                        if let Err(e) = self.handle_configure_characteristic(cmd) {
                            log::error!(
                                "Failed to handle configure characteristic command: {:?}",
                                e
                            );
                        }
                        continue;
                    }

                    let maybe_cmd: Option<HostCommandConfigureCharacteristicRead> =
                        data.decode().ok();
                    if let Some(cmd) = maybe_cmd {
                        log::info!("Received USB command: {:?}", cmd);
                        if let Err(e) = self.handle_configure_characteristic_read(cmd) {
                            log::error!(
                                "Failed to handle configure characteristic command: {:?}",
                                e
                            );
                        }
                        continue;
                    }

                    let maybe_cmd: Option<HostCommandNotifyCharacteristicValue> =
                        data.decode().ok();
                    if let Some(cmd) = maybe_cmd {
                        log::info!("Received USB command: {:?}", cmd);
                        if let Err(e) = self.handle_notify_characteristic_value(cmd) {
                            log::error!(
                                "Failed to handle notify characteristic value command: {:?}",
                                e
                            );
                        }
                        continue;
                    }

                    let maybe_cmd: Option<HostCommandGetServiceInfo> = data.decode().ok();
                    if let Some(cmd) = maybe_cmd {
                        log::info!("Received USB command: {:?}", cmd);
                        if let Err(e) = self.handle_get_service_info(cmd) {
                            log::error!("Failed to handle get service info command: {:?}", e);
                        }
                        continue;
                    }

                    let maybe_cmd: Option<HostCommandGetCharacteristicInfo> = data.decode().ok();
                    if let Some(cmd) = maybe_cmd {
                        log::info!("Received USB command: {:?}", cmd);
                        if let Err(e) = self.handle_get_characteristic_info(cmd) {
                            log::error!(
                                "Failed to handle get characteristic info command: {:?}",
                                e
                            );
                        }
                        continue;
                    }

                    let maybe_cmd: Option<HostCommandStartAdvertisement> = data.decode().ok();
                    if let Some(cmd) = maybe_cmd {
                        if let Err(e) = self.handle_start_advertisement(cmd) {
                            log::error!("Failed to handle start advertisement command: {:?}", e);
                        }
                        continue;
                    }

                    log::warn!(
                        "Received unrecognized command data from USB, raw data length: {} bytes",
                        data.size()
                    );
                }
                Err(e) => {
                    log::error!("Failed to receive data from USB: {:?}", e);
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
            }
        }
    }

    fn handle_configure_peripheral(&mut self, cmd: HostCommandConfigurePeripheral) -> Result<()> {
        log::info!(
            "Configuring peripheral with name: '{}', UUID: {}",
            cmd.name,
            cmd.uuid
        );

        log::debug!("Setting up BLE security configuration");
        self.ble_device
            .security()
            .set_auth(AuthReq::all())
            .set_passkey(123456)
            .set_io_cap(SecurityIOCap::DisplayOnly)
            .resolve_rpa();

        self.metadata.ble_name = Some(cmd.name.clone());
        self.server = Some(self.ble_device.get_server());
        log::info!("Successfully configured peripheral '{}'", cmd.name);
        Ok(())
    }

    fn handle_start_advertisement(&mut self, cmd: HostCommandStartAdvertisement) -> Result<()> {
        let advertisement = self.ble_device.get_advertising();
        log::info!(
            "Starting BLE advertisement, multi-connect: {}",
            cmd.allow_multi_connect
        );

        match self.metadata.ble_name.as_ref() {
            Some(name) => {
                advertisement
                    .lock()
                    .set_data(esp32_nimble::BLEAdvertisementData::new().name(name.as_str()))
                    .map_err(|e| {
                        log::error!("Failed to set advertisement data: {:?}", e);
                        StateMachineError::AdvertisementError("Failed to start advertisement")
                    })?;
                advertisement.lock().start().map_err(|e| {
                    log::error!("Failed to start advertisement: {:?}", e);
                    StateMachineError::AdvertisementError("Failed to start advertisement")
                })?;
                log::info!("Started BLE advertisement with name: {name}");
            }
            None => {
                log::error!(
                    "Error: Received advertisement command without peripheral configuration"
                );
                self.usb_sender
                    .send(PluginConfigurationError::AdvertisementWithoutPeripheralConfiguration)
                    .map_err(|_| StateMachineError::UsbSendError)?;
                return Err(StateMachineError::InvalidBleConfiguration);
            }
        }

        match self.server.as_mut() {
            Some(server) => {
                server.on_connect(move |server, desc| {
                    log::info!("Client connected: {:?}", desc);

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
                log::info!("Successfully configured BLE server callbacks");
            }
            None => {
                log::error!("Error: Server not initialized for BLE device");
                return Err(StateMachineError::ServerNotInitialized);
            }
        }
        Ok(())
    }

    fn handle_configure_service(&mut self, cmd: HostCommandConfigureService) -> Result<()> {
        log::info!("Configuring BLE service with UUID: {}", cmd.uuid,);

        let server = match self.server.as_mut() {
            Some(server) => server,
            None => {
                log::error!("BLE server not initialized - peripheral must be configured first");
                self.usb_sender
                    .send(PluginConfigurationError::ServiceWithoutPeripheralConfiguration)
                    .map_err(|_| StateMachineError::UsbSendError)?;
                return Err(StateMachineError::ServerNotInitialized);
            }
        };

        // Convert UUID to BleUuid
        let ble_uuid = BleUuid::from_uuid128_string(&cmd.uuid.to_string()).map_err(|e| {
            log::error!("Failed to convert UUID to BleUuid: {:?}", e);
            StateMachineError::InvalidBleConfiguration
        })?;

        // Create the BLE service
        server.create_service(ble_uuid).lock();

        log::info!("Successfully created BLE service with UUID: {}", cmd.uuid);

        Ok(())
    }

    /// Get a stored BLE service by UUID for characteristic creation
    pub fn get_service(
        &self,
        service_uuid: Uuid,
    ) -> Option<&Arc<esp32_nimble::utilities::mutex::Mutex<BLEService>>> {
        match self.server.as_ref() {
            Some(server) => block_on(
                server.get_service(BleUuid::from_uuid128_string(&service_uuid.to_string()).ok()?),
            ),
            None => None,
        }
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
            .get_service(cmd.service_uuid)
            .ok_or_else(|| {
                log::error!(
                    "Service with UUID {} not found - service must be configured first",
                    cmd.service_uuid
                );
                self.usb_sender
                    .send(PluginConfigurationError::CharacteristicWithoutServiceConfiguration)
                    .ok();
                StateMachineError::InvalidBleConfiguration
            })?
            .lock();

        // Get the characteristic
        let characteristic = block_on(service.get_characteristic(
            BleUuid::from_uuid128_string(&cmd.characteristic_uuid.to_string()).map_err(|e| {
                log::error!("Failed to convert characteristic UUID to BleUuid: {:?}", e);
                self.usb_sender
                    .send(PluginConfigurationError::InvalidCharacteristicUuid)
                    .ok();
                StateMachineError::InvalidBleConfiguration
            })?,
        ))
        .ok_or_else(|| {
            log::error!(
                "Characteristic with UUID {} not found in service {}",
                cmd.characteristic_uuid,
                cmd.service_uuid
            );
            StateMachineError::InvalidBleConfiguration
        })?;

        // Get the characteristic
        let characteristic_lock = characteristic.lock();

        match self.server.as_ref() {
            Some(server) => {
                let conn = server
                    .connections()
                    .find(|desc| {
                        desc.address()
                            == BLEAddress::from_be_bytes(
                                cmd.address,
                                Self::bluetooth_address_type_to_ble_address_type(cmd.address_type),
                            )
                    })
                    .ok_or_else(|| {
                        log::error!(
                            "Connection with address {:?} and type {:?} not found",
                            cmd.address,
                            cmd.address_type
                        );
                        StateMachineError::InvalidBleConfiguration
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
                        StateMachineError::CharacteristicNotificationError
                    })?;
            }
            None => {
                log::error!("BLE server not initialized - peripheral must be configured first");
                self.usb_sender
                    .send(PluginConfigurationError::ServiceWithoutPeripheralConfiguration)
                    .map_err(|_| StateMachineError::UsbSendError)?;
                return Err(StateMachineError::ServerNotInitialized);
            }
        }

        log::info!(
            "Successfully notified characteristic {} with value: {:?}",
            cmd.characteristic_uuid,
            cmd.value.as_slice()
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
            .get_service(cmd.service_uuid)
            .ok_or_else(|| {
                log::error!(
                    "Service with UUID {} not found - service must be configured first",
                    cmd.service_uuid
                );
                self.usb_sender
                    .send(PluginConfigurationError::CharacteristicWithoutServiceConfiguration)
                    .ok();
                StateMachineError::InvalidBleConfiguration
            })?
            .lock();

        let characteristic = block_on(service.get_characteristic(
            BleUuid::from_uuid128_string(&cmd.uuid.to_string()).map_err(|e| {
                log::error!("Failed to convert characteristic UUID to BleUuid: {:?}", e);
                self.usb_sender
                    .send(PluginConfigurationError::InvalidCharacteristicUuid)
                    .ok();
                StateMachineError::InvalidBleConfiguration
            })?,
        ))
        .ok_or_else(|| StateMachineError::InvalidBleConfiguration)?;

        characteristic.lock().set_value(cmd.value.as_slice());
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
        let service = self.get_service(cmd.service_uuid).ok_or_else(|| {
            log::error!(
                "Service with UUID {} not found - service must be configured first",
                cmd.service_uuid
            );
            self.usb_sender
                .send(PluginConfigurationError::CharacteristicWithoutServiceConfiguration)
                .ok();
            StateMachineError::InvalidBleConfiguration
        })?;

        // Convert UUID to BleUuid
        let ble_uuid = BleUuid::from_uuid128_string(&cmd.uuid.to_string()).map_err(|e| {
            log::error!("Failed to convert characteristic UUID to BleUuid: {:?}", e);
            StateMachineError::InvalidBleConfiguration
        })?;

        // Convert properties from u8 to NimbleProperties
        let mut nimble_properties = NimbleProperties::empty();
        if cmd.properties.contains(&BLEProperties::READ) {
            nimble_properties |= NimbleProperties::READ;
        }
        if cmd.properties.contains(&BLEProperties::WRITE) {
            nimble_properties |= NimbleProperties::WRITE;
        }
        if cmd.properties.contains(&BLEProperties::WriteNoRsp) {
            nimble_properties |= NimbleProperties::WRITE_NO_RSP;
        }
        if cmd.properties.contains(&BLEProperties::NOTIFY) {
            nimble_properties |= NimbleProperties::NOTIFY;
        }
        if cmd.properties.contains(&BLEProperties::INDICATE) {
            nimble_properties |= NimbleProperties::INDICATE;
        }

        // Create the characteristic
        let characteristic = service
            .lock()
            .create_characteristic(ble_uuid, nimble_properties);

        self.metadata
            .service_to_characteristic_uuids
            .entry(cmd.service_uuid)
            .or_default()
            .push((cmd.uuid, cmd.properties))
            .map_err(|_| {
                log::error!("Failed to store characteristic UUID: {}", cmd.uuid);
                StateMachineError::CharacteristicUuidStorageError
            })?;

        match nimble_properties.contains(NimbleProperties::WRITE) {
            true => {
                let char_uuid_write = cmd.uuid;
                let service_uuid_write = cmd.service_uuid;
                let usb_sender = Arc::clone(&self.usb_sender);
                characteristic.lock().on_write(move |args| {
                    log::info!(
                        "BLE write received for characteristic {} in service {}: {:?} bytes",
                        char_uuid_write,
                        service_uuid_write,
                        args.current_data()
                    );
                    usb_sender
                        .send(PluginData {
                            src_id: char_uuid_write, // This should be the peripheral ID
                            send_type: protocol::io_types::PluginDataSendType::Write,
                            data: args.current_data(),
                        })
                        .map_err(|_| StateMachineError::UsbSendError)
                        .ok();
                });
            }
            false => {
                log::warn!(
                    "Characteristic {} does not support WRITE property",
                    cmd.uuid
                );
            }
        }

        match nimble_properties.contains(NimbleProperties::READ) {
            true => {
                let usb_sender = Arc::clone(&self.usb_sender);
                characteristic.lock().on_read(move |characteristics, _| {
                    log::info!(
                        "BLE read requested for characteristic {} in service {}",
                        cmd.uuid,
                        cmd.service_uuid
                    );

                    usb_sender
                        .send(PluginData {
                            src_id: Uuid::from_str(characteristics.uuid().to_string().as_str())
                                .unwrap_or(Uuid::nil()),
                            send_type: protocol::io_types::PluginDataSendType::Read,
                            data: &[],
                        })
                        .map_err(|_| StateMachineError::UsbSendError)
                        .ok();
                });
            }
            false => {
                log::warn!("Characteristic {} does not support READ property", cmd.uuid);
            }
        };

        log::info!(
            "Successfully configured BLE characteristic with UUID: {} for service: {}",
            cmd.uuid,
            cmd.service_uuid
        );

        Ok(())
    }

    fn handle_get_service_info(&mut self, cmd: HostCommandGetServiceInfo) -> Result<()> {
        log::info!("Processing get service info command for UUID: {}", cmd.uuid);

        let characteristic_uuids = self
            .metadata
            .service_to_characteristic_uuids
            .get(&cmd.uuid)
            .map(|chars| {
                let mut uuids = heapless::Vec::new();
                for (uuid, _properties) in chars {
                    uuids.push(*uuid).ok();
                }
                uuids
            })
            .unwrap_or_else(|| {
                log::warn!("No characteristics found for service {}", cmd.uuid);
                heapless::Vec::new()
            });

        let response = PluginServiceInfoResponse {
            service_uuid: cmd.uuid,
            characteristic_uuids,
            exists: self.get_service(cmd.uuid).is_some(),
        };

        // Send the response to USB
        self.usb_sender.send(response).map_err(|_| {
            log::error!("Failed to send service info response to USB");
            StateMachineError::UsbSendError
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
            .get(&cmd.service_uuid)
            .and_then(|chars| {
                chars.iter().find_map(|(uuid, properties)| {
                    if *uuid == cmd.characteristic_uuid {
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

                (false, heapless::Vec::new())
            });

        let response = PluginCharacteristicInfoResponse {
            characteristic_uuid: cmd.characteristic_uuid,
            service_uuid: cmd.service_uuid,
            properties,
            exists,
        };

        // Send the response to USB
        self.usb_sender.send(response).map_err(|_| {
            log::error!("Failed to send characteristic info response to USB");
            StateMachineError::UsbSendError
        })?;

        log::info!(
            "Successfully sent characteristic info response for characteristic {} in service {}",
            cmd.characteristic_uuid,
            cmd.service_uuid
        );
        Ok(())
    }

    fn bluetooth_address_type_to_ble_address_type(
        address_type: protocol::io_types::BluetoothAddressType,
    ) -> BLEAddressType {
        match address_type {
            protocol::io_types::BluetoothAddressType::Public => BLEAddressType::Public,
            protocol::io_types::BluetoothAddressType::Random => BLEAddressType::Random,
            protocol::io_types::BluetoothAddressType::PublicID => BLEAddressType::PublicID,
            protocol::io_types::BluetoothAddressType::RandomID => BLEAddressType::RandomID,
        }
    }
}
