#![deny(missing_docs)]
//! This library is used to to contain the complete processing logic and state machine to facilitate data/command transfer from BLE
//! to usb and visa versa.

pub mod errors;

use errors::Result;
use errors::StateMachineError;

use std::time::Duration;

use esp32_nimble::enums::{AuthReq, SecurityIOCap};
use esp32_nimble::utilities::BleUuid;
use esp32_nimble::{BLEDevice, BLEServer, BLEService};
use heapless::String;
use protocol::io_types::{
    HostCommandConfigureCharacteristic, HostCommandConfigurePeripheral,
    HostCommandConfigureService, HostCommandGetCharacteristicInfo, HostCommandGetServiceInfo,
    HostCommandStartAdvertisement, PluginConfigurationError,
};
use protocol::plugin::plugin::{PluginReceiver, PluginSender};
use protocol::{DEFAULT_PACKET_SIZE, MAX_NAME_SIZE};
use std::collections::HashMap;
use uuid::Uuid;

// This is used to store the metadata of the plugin state machine
#[derive(Default)]
struct PluginStateMachineMetadata {
    ble_name: Option<String<MAX_NAME_SIZE>>,
    services: HashMap<Uuid, std::sync::Arc<esp32_nimble::utilities::mutex::Mutex<BLEService>>>,
}

/// Contains state machine to process BLE and usb data and facilitate their data transfer
pub struct PluginStateMachine {
    usb_sender: PluginSender<DEFAULT_PACKET_SIZE>,
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
            usb_sender,
            usb_receiver,
            ble_device,
            server: None,
            metadata: Default::default(),
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
    pub fn runner(&mut self) {
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
        let server = self.ble_device.get_server();
        self.server = Some(server);
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
                        && server.connected_count()
                            < (esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _)
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
                server.on_disconnect(|_desc, reason| {
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
        log::info!(
            "Configuring BLE service with UUID: {} and name: '{}'",
            cmd.uuid,
            cmd.name
        );

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
        let service = server.create_service(ble_uuid);

        // Store the service for later characteristic creation
        self.metadata.services.insert(cmd.uuid, service);

        log::info!(
            "Successfully created BLE service '{}' with UUID: {}",
            cmd.name,
            cmd.uuid
        );

        Ok(())
    }

    /// Get a stored BLE service by UUID for characteristic creation
    pub fn get_service(
        &self,
        service_uuid: &Uuid,
    ) -> Option<&std::sync::Arc<esp32_nimble::utilities::mutex::Mutex<BLEService>>> {
        self.metadata.services.get(service_uuid)
    }

    fn handle_configure_characteristic(
        &mut self,
        cmd: HostCommandConfigureCharacteristic,
    ) -> Result<()> {
        log::info!("Processing configure characteristic command: {:?}", cmd);
        
        // Note: The current HostCommandConfigureCharacteristic struct is empty.
        // This implementation assumes it will be extended with required fields:
        // - characteristic_uuid: Uuid
        // - service_uuid: Uuid  
        // - properties: characteristic properties (read, write, notify, etc.)
        // - name: optional name for identification
        
        // For now, we'll create a placeholder implementation that demonstrates
        // the callback pattern for sending data to USB
        
        log::warn!("HostCommandConfigureCharacteristic struct is currently empty - this is a placeholder implementation");
        log::info!("Characteristic callbacks will send data to USB when characteristic operations occur");
        
        // TODO: Once HostCommandConfigureCharacteristic is properly defined with fields:
        // 1. Get the service using cmd.service_uuid and self.get_service()
        // 2. Create characteristic with cmd.characteristic_uuid and properties
        // 3. Set up read/write/notify callbacks that use self.usb_sender
        // 4. Store characteristic reference for later use
        
        // Example of how callbacks should work (pseudo-code for when struct has fields):
        /*
        let service = self.get_service(&cmd.service_uuid)
            .ok_or(StateMachineError::InvalidBleConfiguration)?;
            
        let ble_uuid = BleUuid::from_uuid128_string(&cmd.characteristic_uuid.to_string())
            .map_err(|_| StateMachineError::InvalidBleConfiguration)?;
            
        let characteristic = service.lock().create_characteristic(ble_uuid, properties);
        
        // Set up callbacks that send data to USB
        let usb_sender = self.usb_sender.clone();
        characteristic.on_read(move |_| {
            // Send read event to USB
            if let Err(e) = usb_sender.send(read_event_data) {
                log::error!("Failed to send read event to USB: {:?}", e);
            }
        });
        
        characteristic.on_write(move |data| {
            // Send write event to USB  
            if let Err(e) = usb_sender.send(write_event_data) {
                log::error!("Failed to send write event to USB: {:?}", e);
            }
        });
        */
        
        Ok(())
    }

    fn handle_get_service_info(&mut self, cmd: HostCommandGetServiceInfo) -> Result<()> {
        log::info!("Processing get service info command: {:?}", cmd);
        log::warn!("Get service info not yet implemented");
        Ok(())
    }

    fn handle_get_characteristic_info(
        &mut self,
        cmd: HostCommandGetCharacteristicInfo,
    ) -> Result<()> {
        log::info!("Processing get characteristic info command: {:?}", cmd);
        log::warn!("Get characteristic info not yet implemented");
        Ok(())
    }
}
