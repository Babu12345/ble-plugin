#![deny(missing_docs)]
//! This library is used to to contain the complete processing logic and state machine to facilitate data/command transfer from BLE
//! to usb and visa versa.

pub mod errors;

use errors::Result;
use errors::StateMachineError;

use std::time::Duration;

use esp32_nimble::enums::{AuthReq, SecurityIOCap};
use esp32_nimble::{BLEDevice, BLEServer};
use heapless::String;
use protocol::io_types::{
    HostCommandConfigureCharacteristic, HostCommandConfigurePeripheral,
    HostCommandConfigureService, HostCommandGetCharacteristicInfo, HostCommandGetServiceInfo,
    HostCommandStartAdvertisement, PluginConfigurationError,
};
use protocol::plugin::plugin::{PluginReceiver, PluginSender};
use protocol::{DEFAULT_PACKET_SIZE, MAX_NAME_SIZE};

// This is used to store the metadata of the plugin state machine
#[derive(Default)]
struct PluginStateMachineMetadata {
    ble_name: Option<String<MAX_NAME_SIZE>>,
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

        match self.metadata.ble_name.take() {
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

        match self.server.take() {
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
        log::info!("Processing configure service command: {:?}", cmd);
        log::warn!("Service configuration not yet implemented");
        Ok(())
    }

    fn handle_configure_characteristic(
        &mut self,
        cmd: HostCommandConfigureCharacteristic,
    ) -> Result<()> {
        log::info!("Processing configure characteristic command: {:?}", cmd);
        log::warn!("Characteristic configuration not yet implemented");
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
