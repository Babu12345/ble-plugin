#![deny(missing_docs)]
//! # Plugin State Machine Standard
//!
//! A comprehensive BLE-USB bridge state machine implementation for ESP32-based plugin devices.
//!
//! This library provides the core processing logic and state management required to facilitate
//! bidirectional data and command transfer between BLE peripherals and USB hosts. It serves as
//! the central processing unit for BLE plugin devices, handling USB command processing, BLE
//! device management, and efficient message routing.
//!
//! ## Key Features
//!
//! - **Efficient Message Dispatch**: Uses message type IDs for O(1) command routing
//! - **Protocol Validation**: Magic number validation and header integrity checking
//! - **BLE Integration**: Deep integration with ESP32-Nimble BLE stack
//! - **Thread-Safe Communication**: Arc-wrapped senders for callback integration
//! - **Comprehensive Error Handling**: Detailed error types for robust operation
//! - **Memory Efficient**: Designed for embedded systems with limited resources
//!
//! ## Architecture Overview
//!
//! ```text
//! ┌─────────────┐    USB Commands    ┌─────────────────────┐    BLE Operations    ┌─────────────┐
//! │   USB Host  │ ──────────────────►│Plugin State Machine │ ───────────────────► │ BLE Clients │
//! │             │ ◄──────────────────│                     │ ◄─────────────────── │             │
//! └─────────────┘    USB Responses   └─────────────────────┘    BLE Callbacks     └─────────────┘
//! ```
//!
//! ## Message Protocol
//!
//! The state machine uses a standardized 5-byte message header:
//!
//! ```text
//! ┌─────────────┬─────────────┬─────────────┬─────────────────┐
//! │   Magic     │   Type ID   │   Length    │     Payload     │
//! │  (2 bytes)  │  (1 byte)   │  (2 bytes)  │  (limited size) │
//! └─────────────┴─────────────┴─────────────┴─────────────────┘
//! ```
//!
//! - **Magic Number**: 0xDEAD for message integrity validation
//! - **Type ID**: Enables efficient O(1) command dispatch
//! - **Length**: Payload size for proper deserialization
//! - **Payload**: Bincode-serialized command/response data
//!
//! **Size Constraints**: The total message size (header + payload) cannot exceed
//! `DEFAULT_PACKET_SIZE`. With a 5-byte header, the maximum payload
//! size is `DEFAULT_PACKET_SIZE` - 5 bytes.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use plugin_state_machine_std::PluginStateMachine;
//! use protocol::plugin::plugin::{PluginSender, PluginReceiver};
//! use esp32_nimble::BLEDevice;
//! use protocol::DEFAULT_PACKET_SIZE;
//!
//! // Initialize communication channels
//! let (usb_sender, usb_receiver): (PluginSender<DEFAULT_PACKET_SIZE>, _) =
//!     /* your USB channel setup */;
//! # panic!("This is a documentation example");
//! let ble_device = BLEDevice::take();
//!
//! // Create and run the state machine
//! let state_machine = PluginStateMachine::new(usb_sender, usb_receiver, ble_device);
//! let runner = state_machine.runner_fn();
//!
//! // Typically run in a separate thread
//! std::thread::spawn(runner);
//! ```
//!
//! ## Supported Commands
//!
//! ### Peripheral Management
//! - [`HostCommandConfigurePeripheral`]: Configure BLE peripheral with name and UUID
//! - [`HostCommandStartAdvertisement`]: Start BLE advertising
//!
//! ### Service Operations  
//! - [`HostCommandConfigureService`]: Create BLE services
//! - [`HostCommandGetServiceInfo`]: Query service information
//!
//! ### Characteristic Management
//! - [`HostCommandConfigureCharacteristic`]: Create characteristics with properties
//! - [`HostCommandConfigureCharacteristicRead`]: Set up read operations
//! - [`HostCommandGetCharacteristicInfo`]: Query characteristic details
//! - [`HostCommandNotifyCharacteristicValue`]: Send notifications to clients
//!
//! ## Error Handling
//!
//! The state machine provides comprehensive error handling through the [`errors`] module:
//!
//! - [`StateMachineError::InvalidMessageFormat`]: Malformed USB messages
//! - [`StateMachineError::UnknownMessageType`]: Unsupported command types  
//! - [`StateMachineError::InvalidBleConfiguration`]: BLE setup errors
//! - [`StateMachineError::UsbSendError`]: USB communication failures
//!
//! ## Performance Characteristics
//!
//! - **Command Routing**: O(1) lookup using message type IDs
//! - **Memory Usage**: Optimized for embedded systems using heapless collections
//! - **Latency**: Minimal processing overhead with direct dispatch
//! - **Throughput**: Efficient binary serialization with bincode
//!
//! [`HostCommandConfigurePeripheral`]: protocol::io_types::HostCommandConfigurePeripheral
//! [`HostCommandStartAdvertisement`]: protocol::io_types::HostCommandStartAdvertisement
//! [`HostCommandConfigureService`]: protocol::io_types::HostCommandConfigureService
//! [`HostCommandGetServiceInfo`]: protocol::io_types::HostCommandGetServiceInfo
//! [`HostCommandConfigureCharacteristic`]: protocol::io_types::HostCommandConfigureCharacteristic
//! [`HostCommandConfigureCharacteristicRead`]: protocol::io_types::HostCommandConfigureCharacteristicRead
//! [`HostCommandGetCharacteristicInfo`]: protocol::io_types::HostCommandGetCharacteristicInfo
//! [`HostCommandNotifyCharacteristicValue`]: protocol::io_types::HostCommandNotifyCharacteristicValue
//! [`StateMachineError::InvalidMessageFormat`]: errors::StateMachineError::InvalidMessageFormat
//! [`StateMachineError::UnknownMessageType`]: errors::StateMachineError::UnknownMessageType
//! [`StateMachineError::InvalidBleConfiguration`]: errors::StateMachineError::InvalidBleConfiguration
//! [`StateMachineError::UsbSendError`]: errors::StateMachineError::UsbSendError

pub mod errors;

use errors::Result;
use errors::StateMachineError;
use esp32_nimble::enums::OwnAddrType;
use esp32_nimble::BLEAddress;
use esp32_nimble::BLEAddressType;
use esp_idf_svc::hal::gpio::AnyOutputPin;
use esp_idf_svc::hal::gpio::Output;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::task::block_on;
use protocol::io_types::BLEProperties;
use protocol::io_types::HostCommandConfigureCharacteristicRead;
use protocol::io_types::HostCommandConfigurePeripheralSecurity;
use protocol::io_types::HostCommandNotifyCharacteristicValue;
use protocol::io_types::PluginAuthenticationCompletedResponse;
use protocol::io_types::PluginData;
use protocol::MESSAGE_HEADER_SIZE;
use threadpool::ThreadPool;
use throttle::Throttle;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Duration;

use esp32_nimble::enums::{AuthReq, SecurityIOCap};
use esp32_nimble::utilities::BleUuid;
use esp32_nimble::{BLEDevice, BLEServer, BLEService, NimbleProperties};
use esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS;
use heapless::String;
use protocol::io_types::{
    HostCommandConfigureCharacteristic, HostCommandConfigurePeripheral,
    HostCommandConfigureService, HostCommandGetCharacteristicInfo, HostCommandGetServiceInfo,
    HostCommandStartAdvertisement, PluginCharacteristicInfoResponse, PluginConfigurationError,
    PluginServiceInfoResponse, MAX_CHARACTERISTICS_PER_SERVICE, MAX_PROPERTIES,
};
use protocol::plugin::plugin::{PluginReceiver, PluginSender};
use protocol::{MessageTypeId, MESSAGE_MAGIC, MESSAGE_MAGIC_BYTES};
use protocol::{DEFAULT_PACKET_SIZE, MAX_NAME_SIZE};

use std::sync::Arc;
use uuid::Uuid;
/// Internal metadata storage for the plugin state machine
///
/// This structure maintains the current state and configuration of the BLE plugin,
/// including device name, service-characteristic relationships, and connection information.
#[derive(Default)]
struct PluginStateMachineMetadata {
    /// Optional BLE device name for advertising
    ble_name: Option<String<MAX_NAME_SIZE>>,

    /// Mapping from service UUIDs to their characteristic UUIDs and properties
    ///
    /// This enables efficient lookup of characteristics within services and
    /// provides quick access to characteristic properties for validation.
    service_to_characteristic_uuids: HashMap<
        Uuid,
        heapless::Vec<
            (Uuid, heapless::Vec<BLEProperties, MAX_PROPERTIES>),
            MAX_CHARACTERISTICS_PER_SERVICE,
        >,
    >, // (UUID, properties)
}

impl PluginStateMachineMetadata {
    /// Set the BLE device name for advertising
    fn set_name(mut self, name: String<MAX_NAME_SIZE>) -> Self {
        self.ble_name = Some(name);
        self
    }
}

/// Main state machine for processing BLE and USB data and facilitating bidirectional transfer
///
/// The `PluginStateMachine` serves as the central processing unit for BLE plugin devices,
/// handling USB command reception, BLE device configuration, and data forwarding between
/// the two communication domains.
///
/// ## Architecture
///
/// The state machine operates as a bridge:
/// - **USB Side**: Receives commands from host and sends responses/data
/// - **BLE Side**: Manages peripheral configuration and handles client interactions
/// - **Processing**: Efficiently routes messages using type IDs and maintains state
///
/// ## Thread Safety
///
/// - USB sender is Arc-wrapped for sharing across BLE callbacks
/// - USB receiver has exclusive access for command processing
/// - BLE device uses static mutable reference for ESP32 integration
///
/// ## Usage Pattern
///
/// 1. Create with communication channels and BLE device
/// 2. Start the runner (typically in a separate thread)
/// 3. State machine processes commands automatically
/// 4. BLE callbacks forward data back to USB host
pub struct PluginStateMachine {
    /// Thread-safe USB sender for responses and BLE data forwarding
    usb_sender: Arc<PluginSender<DEFAULT_PACKET_SIZE>>,

    /// USB receiver for incoming host commands (exclusive access)
    usb_receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,

    /// ESP32 BLE device instance (static mutable for hardware integration)
    ble_device: &'static mut BLEDevice,

    /// Optional BLE server instance (created after peripheral configuration)
    server: Option<&'static mut BLEServer>,

    /// Internal state and configuration metadata
    metadata: PluginStateMachineMetadata,

    /// Output pin for LED control (e.g., status indication)
    indicator: Arc<Mutex<PinDriver<'static, AnyOutputPin, Output>>>,

    /// Throttle for blink indication to prevent excessive blinking
    /// and errors
    blink_throttle: Throttle,
    /// Thread pool for managing blink operations
    blink_thread_pool: ThreadPool,
}

/// Enum representing the possible states of the blink indication
enum BlinkState {
    /// Indicates a successful operation
    Success,
    /// Indicates a failure or error condition
    Failure,
}

impl PluginStateMachine {
    /// Create a new instance of the plugin state machine
    ///
    /// Initializes the state machine with the necessary communication channels and BLE device.
    /// The state machine starts in an unconfigured state and requires peripheral configuration
    /// before it can handle BLE operations.
    ///
    /// # Arguments
    ///
    /// * `usb_sender` - Channel for sending responses and data to the USB host
    /// * `usb_receiver` - Channel for receiving commands from the USB host  
    /// * `ble_device` - ESP32 BLE device instance (must be static for hardware integration)
    ///
    /// # Returns
    ///
    /// A new `PluginStateMachine` instance ready to process commands
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use plugin_state_machine_std::PluginStateMachine;
    /// use protocol::plugin::plugin::{PluginSender, PluginReceiver};
    /// use esp32_nimble::BLEDevice;
    /// use protocol::DEFAULT_PACKET_SIZE;
    ///
    /// let (usb_sender, usb_receiver): (PluginSender<DEFAULT_PACKET_SIZE>, _) =
    ///     /* your USB channel setup */;
    /// # panic!("Documentation example");
    /// let ble_device = BLEDevice::take();
    ///
    /// let state_machine = PluginStateMachine::new(usb_sender, usb_receiver, ble_device);
    /// ```
    pub fn new(
        usb_sender: PluginSender<DEFAULT_PACKET_SIZE>,
        usb_receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,
        ble_device: &'static mut BLEDevice,
        indicator: Arc<Mutex<PinDriver<'static, AnyOutputPin, Output>>>,
    ) -> Self {
        Self {
            indicator,
            usb_sender: Arc::new(usb_sender),
            usb_receiver,
            ble_device,
            server: None,
            metadata: Default::default(),
            blink_throttle: Throttle::new(Self::THROTTLE_INFO.0, Self::THROTTLE_INFO.1),
            blink_thread_pool: ThreadPool::new(1),
        }
    }

    /// Throttle information for blink indication - allow 5 blinks per second
    const THROTTLE_INFO: (Duration, usize) = (Duration::from_millis(500), 1);

    /// Extract message type ID from received USB data with validation
    ///
    /// This method validates the message header format and extracts the message type ID
    /// for efficient command dispatch. It performs integrity checks including magic
    /// number validation and header size verification.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw USB data buffer containing message header and payload
    ///
    /// # Returns
    ///
    /// * `Ok(MessageTypeId)` - Successfully extracted message type ID
    /// * `Err(StateMachineError)` - Invalid message format or unknown type ID
    ///
    /// # Errors
    ///
    /// * `InvalidMessageFormat` - Data too short, invalid magic number
    /// * `UnknownMessageType` - Unrecognized message type ID
    ///
    /// # Message Header Format
    ///
    /// ```text
    /// [0-1]: Magic number (0xDEAD, little-endian)
    /// [2]:   Message type ID
    /// [3-4]: Payload length (little-endian)
    /// [5+]:  Payload data
    /// ```
    fn extract_message_type_id(data: &[u8]) -> Result<MessageTypeId> {
        // Check if we have enough bytes for a valid header
        if data.len() < MESSAGE_HEADER_SIZE {
            log::error!("Received data too short for valid message header");
            return Err(StateMachineError::InvalidMessageFormat);
        }

        // Verify magic number
        let magic = u16::from_le_bytes([data[0], data[1]]);
        if magic != MESSAGE_MAGIC {
            log::error!(
                "Invalid magic number: expected 0x{:X}, got 0x{:X}",
                MESSAGE_MAGIC,
                magic
            );
            return Err(StateMachineError::InvalidMessageFormat);
        }

        // Extract message type ID
        let type_id = data[MESSAGE_MAGIC_BYTES];
        match type_id {
            0x01 => Ok(MessageTypeId::HostCommandConfigurePeripheral),
            0x02 => Ok(MessageTypeId::HostCommandConfigureService),
            0x03 => Ok(MessageTypeId::HostCommandConfigureCharacteristic),
            0x04 => Ok(MessageTypeId::HostCommandConfigureCharacteristicRead),
            0x05 => Ok(MessageTypeId::HostCommandGetServiceInfo),
            0x06 => Ok(MessageTypeId::HostCommandGetCharacteristicInfo),
            0x07 => Ok(MessageTypeId::HostCommandStartAdvertisement),
            0x08 => Ok(MessageTypeId::HostCommandNotifyCharacteristicValue),
            _ => {
                log::error!("Unknown message type ID: 0x{:02X}", type_id);
                Err(StateMachineError::UnknownMessageType)
            }
        }
    }

    /// Returns a closure that can be used to run the state machine in a separate thread
    ///
    /// This method consumes the state machine and returns a closure suitable for
    /// spawning in a separate thread. The closure contains the main processing loop
    /// that handles USB commands and manages BLE operations.
    ///
    /// # Returns
    ///
    /// A closure that runs the state machine's main processing loop
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use plugin_state_machine_std::PluginStateMachine;
    /// # let state_machine = panic!("Documentation example");
    ///
    /// let runner = state_machine.runner_fn();
    ///
    /// // Run in separate thread
    /// std::thread::spawn(runner);
    ///
    /// // Or run in async context
    /// // tokio::task::spawn_blocking(runner);
    /// ```
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
    /// Uses message type ID for fast and accurate message dispatch.
    fn runner(&mut self) {
        log::info!("Starting USB-BLE bridge runner");
        loop {
            match self.usb_receiver.receive() {
                Ok(data) => {
                    log::debug!("Received USB data of length : {} bytes", data.size());

                    // Extract message type ID for efficient dispatch
                    match Self::extract_message_type_id(data.raw_bytes()) {
                        Ok(message_type) => {
                            let result = match message_type {
                                MessageTypeId::HostCommandConfigurePeripheral => {
                                    match data.decode::<HostCommandConfigurePeripheral>() {
                                        Ok(cmd) => {
                                            log::info!("Received USB command: {:?}", cmd);
                                            self.handle_configure_peripheral(cmd)
                                        }
                                        Err(_) => Err(StateMachineError::FailedToDecodeMessage(
                                            "HostCommandConfigurePeripheral",
                                        )),
                                    }
                                }
                                MessageTypeId::HostCommandConfigureService => {
                                    match data.decode::<HostCommandConfigureService>() {
                                        Ok(cmd) => {
                                            log::info!("Received USB command: {:?}", cmd);
                                            self.handle_configure_service(cmd)
                                        }
                                        Err(_) => Err(StateMachineError::FailedToDecodeMessage(
                                            "HostCommandConfigureService",
                                        )),
                                    }
                                }
                                MessageTypeId::HostCommandConfigureCharacteristic => {
                                    match data.decode::<HostCommandConfigureCharacteristic>() {
                                        Ok(cmd) => {
                                            log::info!("Received USB command: {:?}", cmd);
                                            self.handle_configure_characteristic(cmd)
                                        }
                                        Err(_) => Err(StateMachineError::FailedToDecodeMessage(
                                            "HostCommandConfigureCharacteristic",
                                        )),
                                    }
                                }
                                MessageTypeId::HostCommandConfigureCharacteristicRead => {
                                    match data.decode::<HostCommandConfigureCharacteristicRead>() {
                                        Ok(cmd) => {
                                            log::info!("Received USB command: {:?}", cmd);
                                            self.handle_configure_characteristic_read(cmd)
                                        }
                                        Err(_) => Err(StateMachineError::FailedToDecodeMessage(
                                            "HostCommandConfigureCharacteristicRead",
                                        )),
                                    }
                                }
                                MessageTypeId::HostCommandNotifyCharacteristicValue => {
                                    match data.decode::<HostCommandNotifyCharacteristicValue>() {
                                        Ok(cmd) => {
                                            log::info!("Received USB command: {:?}", cmd);
                                            self.handle_notify_characteristic_value(cmd)
                                        }
                                        Err(_) => Err(StateMachineError::FailedToDecodeMessage(
                                            "HostCommandNotifyCharacteristicValue",
                                        )),
                                    }
                                }
                                MessageTypeId::HostCommandGetServiceInfo => {
                                    match data.decode::<HostCommandGetServiceInfo>() {
                                        Ok(cmd) => {
                                            log::info!("Received USB command: {:?}", cmd);
                                            self.handle_get_service_info(cmd)
                                        }
                                        Err(_) => Err(StateMachineError::FailedToDecodeMessage(
                                            "HostCommandGetServiceInfo",
                                        )),
                                    }
                                }
                                MessageTypeId::HostCommandGetCharacteristicInfo => {
                                    match data.decode::<HostCommandGetCharacteristicInfo>() {
                                        Ok(cmd) => {
                                            log::info!("Received USB command: {:?}", cmd);
                                            self.handle_get_characteristic_info(cmd)
                                        }
                                        Err(_) => Err(StateMachineError::FailedToDecodeMessage(
                                            "HostCommandGetCharacteristicInfo",
                                        )),
                                    }
                                }
                                MessageTypeId::HostCommandStartAdvertisement => {
                                    match data.decode::<HostCommandStartAdvertisement>() {
                                        Ok(cmd) => {
                                            log::info!("Received USB command: {:?}", cmd);
                                            self.handle_start_advertisement(cmd)
                                        }
                                        Err(_) => Err(StateMachineError::FailedToDecodeMessage(
                                            "HostCommandStartAdvertisement",
                                        )),
                                    }
                                }

                                MessageTypeId::HostCommandConfigurePeripheralSecurity => {
                                    match data.decode::<HostCommandConfigurePeripheralSecurity>() {
                                        Ok(cmd) => {
                                            log::info!("Received USB command: {:?}", cmd);
                                            self.handle_configure_peripheral_security(cmd)
                                        }
                                        Err(_) => Err(StateMachineError::FailedToDecodeMessage(
                                            "HostCommandConfigurePeripheralSecurity",
                                        )),
                                    }
                                }
                                _ => Err(StateMachineError::UnhandledMessageType(message_type)),
                            };

                            if let Err(e) = result {
                                log::error!("Failed to handle command {:?}: {:?}", message_type, e);
                                self.blink_indication(BlinkState::Failure);
                            } else {
                                self.blink_indication(BlinkState::Success);
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to extract message type ID: {e}");
                            log::warn!(
                                "Received unrecognized command data from USB, raw data length: {} bytes",
                                data.size()
                            );
                            self.blink_indication(BlinkState::Failure);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to receive data from USB: {:?}", e);
                    std::thread::sleep(Duration::from_millis(100));
                    self.blink_indication(BlinkState::Failure);
                }
            }
        }
    }

    fn blink_indication(&mut self, state: BlinkState) {
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
                    BlinkState::Success => {
                        std::thread::sleep(Duration::from_millis(if i == 0 { 50 } else { 5 }));
                    }
                    BlinkState::Failure => {
                        std::thread::sleep(Duration::from_millis(40));
                    }
                }
            }
        });
    }

    fn handle_configure_peripheral(&mut self, cmd: HostCommandConfigurePeripheral) -> Result<()> {
        log::info!(
            "Configuring peripheral with name: '{}', UUID: {:?}",
            cmd.name,
            cmd.addr
        );

        // If we haven't already initialized then we can set the BLE device address
        // otherwise we cannot without resetting the BLE device
        if self.metadata.ble_name.is_none() {
            self.ble_device.set_own_addr_type(OwnAddrType::Random);
            self.ble_device.set_rnd_addr(cmd.addr).map_err(|_| {
                log::error!("Failed to set random address for BLE device");
                StateMachineError::UnableToSetRNDAddress
            })?;
        }

        self.metadata = PluginStateMachineMetadata::default().set_name(cmd.name.clone());
        self.server = Some(self.ble_device.get_server());
        log::info!("Successfully configured peripheral '{}'", cmd.name);
        Ok(())
    }

    fn handle_configure_peripheral_security(
        &mut self,
        cmd: HostCommandConfigurePeripheralSecurity,
    ) -> Result<()> {
        log::debug!("Setting up BLE security configuration");

        if cmd.passkey > 999999 {
            log::error!("Invalid passkey: must be a 6-digit number");
            return Err(StateMachineError::InvalidPasskeyLength);
        }

        self.ble_device
            .security()
            .set_auth(AuthReq::all())
            .set_passkey(cmd.passkey)
            .set_io_cap(SecurityIOCap::DisplayOnly)
            .resolve_rpa();

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
                let mut adv_data_base = esp32_nimble::BLEAdvertisementData::new();
                let adv_data = adv_data_base.name(name.as_str());

                // Get all service UUIDs to include in advertisement
                for uuid in self.get_service_uuids().iter() {
                    let ble_uuid =
                        BleUuid::from_uuid128_string(&uuid.to_string()).map_err(|e| {
                            log::error!("Failed to convert service UUID to BleUuid: {:?}", e);
                            StateMachineError::InvalidBleConfiguration
                        })?;
                    adv_data.add_service_uuid(ble_uuid);
                }

                advertisement.lock().set_data(adv_data).map_err(|e| {
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

                let usb_sender = self.usb_sender.clone();
                server.on_authentication_complete(move |_, desc, status| {
                    log::info!("Authentication completed for client: {:?}", desc);
                    let response = PluginAuthenticationCompletedResponse {
                        address: desc.address().as_be_bytes(),
                        address_type: Self::ble_address_type_to_bluetooth_address_type(
                            desc.address().addr_type(),
                        ),
                        success: status.is_ok(),
                    };
                    usb_sender
                        .send(response)
                        .map_err(|e| {
                            log::error!("Failed to send authentication response: {:?}", e);
                            StateMachineError::UsbSendError
                        })
                        .ok();
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
        server.create_service(ble_uuid);

        log::info!("Successfully created BLE service with UUID: {}", cmd.uuid);

        // Create a serivce entry and clear any existing characteristics for this service
        self.metadata
            .service_to_characteristic_uuids
            .entry(cmd.uuid)
            .or_default()
            .clear();

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

    /// Get all configured service UUIDs
    pub fn get_service_uuids(&self) -> heapless::Vec<Uuid, 16> {
        self.metadata
            .service_to_characteristic_uuids
            .keys()
            .cloned()
            .collect()
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
                let usb_sender = self.usb_sender.clone();
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
                let usb_sender = self.usb_sender.clone();
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

    fn ble_address_type_to_bluetooth_address_type(
        address_type: BLEAddressType,
    ) -> protocol::io_types::BluetoothAddressType {
        match address_type {
            BLEAddressType::Public => protocol::io_types::BluetoothAddressType::Public,
            BLEAddressType::Random => protocol::io_types::BluetoothAddressType::Random,
            BLEAddressType::PublicID => protocol::io_types::BluetoothAddressType::PublicID,
            BLEAddressType::RandomID => protocol::io_types::BluetoothAddressType::RandomID,
        }
    }
}
