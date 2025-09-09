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
//! - **Memory Efficient**: Uses `heapless` collections for predictable memory usage
//! - **Non-Volatile Storage**: Persistent configuration storage using ESP32 NVS partitions
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
//! let (sender, receiver): (PluginSender<DEFAULT_PACKET_SIZE>, _) =
//!     /* your USB channel setup */;
//! # panic!("This is a documentation example");
//! let ble_device = BLEDevice::take();
//!
//! // Create and run the state machine
//! let state_machine = PluginStateMachine::new(sender, receiver, ble_device);
//! let runner = state_machine.runner_fn();
//!
//! // Typically run in a separate thread
//! std::thread::spawn(runner);
//! ```
//!
//! ## Non-Volatile Storage (NVS)
//!
//! The state machine leverages ESP32's Non-Volatile Storage (NVS) subsystem for persistent
//! configuration management. This enables the plugin device to retain critical settings
//! across power cycles and resets.
//!
//! ### NVS Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         NVS Partition (Flash)           │
//! ├─────────────────────────────────────────┤
//! │  ConfigNamespace                        │
//! │  ├── BLE Device Name                    │
//! │  ├── [Future: Service Configurations]   │
//! │  └── [Future: Security Settings]        │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ### Current Storage Capabilities
//!
//! - **BLE Device Name**: Automatically persisted when configured via [`HostCommandConfigurePeripheral`]
//!   - Stored in the `ConfigNamespace` under the `name_config_key`
//!   - Survives device resets and power cycles
//!   - Maximum name length: `MAX_NAME_SIZE` bytes
//!
//! ### Storage Operations
//!
//! The NVS integration provides:
//! - **Automatic Persistence**: Configuration changes are immediately written to flash
//! - **Namespace Isolation**: Uses dedicated `ConfigNamespace` to prevent conflicts
//! - **Error Recovery**: Graceful handling of write failures with error logging
//! - **Thread-Safe Access**: NVS operations are protected by internal synchronization
//!
//! ### Future NVS Enhancements
//!
//! The NVS infrastructure is designed for extensibility:
//! - Service and characteristic configurations
//! - Security settings and pairing information
//! - Custom application-specific data
//! - Connection history and trusted devices
//!
//! ### Usage Example
//!
//! ```rust,no_run
//! use plugin_state_machine_std::PluginStateMachine;
//! use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};
//!
//! // Initialize NVS partition
//! let nvs_partition = EspNvsPartition::<NvsDefault>::take()?;
//!
//! // Create state machine with NVS support
//! let state_machine = PluginStateMachine::new(
//!     sender,
//!     receiver,
//!     indicator,
//!     nvs_partition
//! )?;
//!
//! // BLE name will be automatically persisted to NVS when configured
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Supported Commands
//!
//! ### Peripheral Management
//! - [`HostCommandConfigurePeripheral`]: Configure BLE peripheral with name and address (persisted to NVS)
//! - [`HostCommandStartAdvertisement`]: Start BLE advertising
//! - [`HostCommandConfigurePeripheralSecurity`]: Configure security settings (pairing, passkey)
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
//! ### Profile Management
//! - [`HostCommandConfigureProfile`]: Configure predefined BLE profiles using existing definitions
//!
//! ## Error Handling
//!
//! The state machine provides comprehensive error handling through the [`errors`] module:
//!
//! - [`StateMachineError::InvalidMessageFormat`]: Malformed USB messages
//! - [`StateMachineError::UnknownMessageType`]: Unsupported command types  
//! - [`StateMachineError::InvalidBleConfiguration`]: BLE setup errors
//! - [`StateMachineError::UsbSendError`]: USB communication failures
//! - [`StateMachineError::NvsWriteError`]: Failed to persist data to NVS
//! - [`StateMachineError::FailedToResolveNvsNamespace`]: NVS namespace initialization error
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
//! [`HostCommandConfigurePeripheralSecurity`]: protocol::io_types::HostCommandConfigurePeripheralSecurity
//! [`HostCommandConfigureService`]: protocol::io_types::HostCommandConfigureService
//! [`HostCommandGetServiceInfo`]: protocol::io_types::HostCommandGetServiceInfo
//! [`HostCommandConfigureCharacteristic`]: protocol::io_types::HostCommandConfigureCharacteristic
//! [`HostCommandConfigureCharacteristicRead`]: protocol::io_types::HostCommandConfigureCharacteristicRead
//! [`HostCommandGetCharacteristicInfo`]: protocol::io_types::HostCommandGetCharacteristicInfo
//! [`HostCommandNotifyCharacteristicValue`]: protocol::io_types::HostCommandNotifyCharacteristicValue
//! [`HostCommandConfigureProfile`]: protocol::io_types::HostCommandConfigureProfile
//! [`StateMachineError::InvalidMessageFormat`]: errors::StateMachineError::InvalidMessageFormat
//! [`StateMachineError::UnknownMessageType`]: errors::StateMachineError::UnknownMessageType
//! [`StateMachineError::InvalidBleConfiguration`]: errors::StateMachineError::InvalidBleConfiguration
//! [`StateMachineError::UsbSendError`]: errors::StateMachineError::UsbSendError
//! [`StateMachineError::NvsWriteError`]: errors::StateMachineError::NvsWriteError
//! [`StateMachineError::FailedToResolveNvsNamespace`]: errors::StateMachineError::FailedToResolveNvsNamespace

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
use esp_idf_svc::nvs::EspNvsPartition;
use esp_idf_svc::nvs::NvsPartitionId;
use plugin_nvs::namespace;
use plugin_nvs::namespaces::ConfigNamespace;
use protocol::protocol::{
    BleProfile, BleProperties, BluetoothAddressType, HostCommandConfigureCharacteristic,
    HostCommandConfigureCharacteristicRead, HostCommandConfigurePeripheral,
    HostCommandConfigurePeripheralSecurity, HostCommandConfigureProfile,
    HostCommandConfigureService, HostCommandGetCharacteristicInfo, HostCommandGetServiceInfo,
    HostCommandNotifyCharacteristicValue, HostCommandStartAdvertisement,
    HostCommandStopAdvertisement, PluginAuthenticationCompletedResponse,
    PluginCharacteristicInfoResponse, PluginConfigurationError, PluginConfigurationErrorType,
    PluginData, PluginServiceInfoResponse,
};
use protocol::utils::slice_to_array;
use threadpool::ThreadPool;
use throttle::Throttle;

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::Duration;

use esp32_nimble::enums::{AuthReq, SecurityIOCap};
use esp32_nimble::utilities::BleUuid;
use esp32_nimble::{BLEDevice, BLEServer, BLEService, NimbleProperties};
use esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS;
use heapless::String;
use protocol::plugin::plugin::{PluginReceiver, PluginSender};
use protocol::protocol::MessageTypeId;
use protocol::{
    DEFAULT_PACKET_SIZE, MAX_NAME_SIZE, MESSAGE_HEADER_SIZE, MESSAGE_MAGIC, MESSAGE_MAGIC_BYTES,
};

/// Maximum number of characteristics per service
const MAX_CHARACTERISTICS_PER_SERVICE: usize = 16;

/// Check whether the statemachine has been initialized
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);

use std::sync::Arc;
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
    service_to_characteristic_uuids:
        HashMap<u16, heapless::Vec<(u16, Vec<i32>), MAX_CHARACTERISTICS_PER_SERVICE>>, // (UUID, properties)
}

impl PluginStateMachineMetadata {
    /// Set the local BLE device name
    /// Does not persist to NVS
    fn set_name_local(&mut self, name: String<MAX_NAME_SIZE>) {
        self.ble_name = Some(name);
    }

    /// Set the BLE device name for advertising
    /// Also persists the name to NVS storage
    fn set_name<T>(&mut self, ns: &mut ConfigNamespace<T>, name: String<MAX_NAME_SIZE>)
    where
        T: NvsPartitionId,
    {
        self.set_name_local(name.clone());
        ns.name_config_key()
            .write(name.as_bytes())
            .map_err(|e| {
                log::error!("Failed to write name to NVS: {:?}", e);
                StateMachineError::NvsWriteError
            })
            .ok();
    }

    /// Get the BLE device name, initializing from NVS if not already set
    fn get_or_init_name<T>(&mut self, ns: &mut ConfigNamespace<T>) -> Option<String<MAX_NAME_SIZE>>
    where
        T: NvsPartitionId,
    {
        if let Some(name) = &self.ble_name {
            return Some(name.clone());
        }

        let mut buffer = [0u8; MAX_NAME_SIZE];
        match ns.name_config_key().read(&mut buffer) {
            Ok(data) => {
                if data?.len() > MAX_NAME_SIZE {
                    log::error!(
                        "Stored name in NVS exceeds maximum length of {} bytes",
                        MAX_NAME_SIZE
                    );
                    return None;
                }
                let mut name: String<MAX_NAME_SIZE> = String::new();
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
pub struct PluginStateMachine<T>
where
    T: NvsPartitionId,
{
    /// Thread-safe USB sender for responses and BLE data forwarding
    sender: Arc<PluginSender<DEFAULT_PACKET_SIZE>>,

    /// USB receiver for incoming host commands (exclusive access)
    receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,

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

    /// NVS namespace for persistent configuration storage
    ///
    /// This provides access to the Non-Volatile Storage partition where device
    /// configurations are persisted across power cycles.
    ns: ConfigNamespace<T>,
}

/// Enum representing the possible states of the blink indication
enum BlinkState {
    /// Indicates a successful operation
    Success,
    /// Indicates a failure or error condition
    Failure,
}

impl<T> PluginStateMachine<T>
where
    T: NvsPartitionId,
{
    /// Create a new instance of the plugin state machine
    ///
    /// Initializes the state machine with the necessary communication channels and BLE device.
    /// The state machine starts in an unconfigured state and requires peripheral configuration
    /// before it can handle BLE operations.
    ///
    /// # Arguments
    ///
    /// * `sender` - Channel for sending responses and data to the USB host
    /// * `receiver` - Channel for receiving commands from the USB host  
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
    /// let (sender, receiver): (PluginSender<DEFAULT_PACKET_SIZE>, _) =
    ///     /* your USB channel setup */;
    /// # panic!("Documentation example");
    /// let ble_device = BLEDevice::take();
    ///
    /// let state_machine = PluginStateMachine::new(sender, receiver, ble_device);
    /// ```
    pub fn new(
        sender: PluginSender<DEFAULT_PACKET_SIZE>,
        receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,
        indicator: Arc<Mutex<PinDriver<'static, AnyOutputPin, Output>>>,
        nvs_partition: EspNvsPartition<T>,
    ) -> Result<Self>
    where
        T: NvsPartitionId,
    {
        Ok(Self {
            indicator,
            sender: Arc::new(sender),
            receiver,
            ble_device: BLEDevice::take(),
            server: None,
            metadata: Default::default(),
            blink_throttle: Throttle::new(Self::THROTTLE_INFO.0, Self::THROTTLE_INFO.1),
            blink_thread_pool: ThreadPool::new(1),
            ns: namespace::<T, ConfigNamespace<T>>(nvs_partition)
                .map_err(|_| StateMachineError::FailedToResolveNvsNamespace())?,
        })
    }

    /// Throttle information for blink indication - allow 5 blinks per second
    const THROTTLE_INFO: (Duration, usize) = (Duration::from_millis(500), 1);

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
            match self.receiver.receive() {
                Ok(data) => {
                    log::debug!("Received USB data of length : {} bytes", data.size());

                    // Extract message type ID for efficient dispatch
                    match data.extract_message_type_id() {
                        Ok(message_type) => {
                            let result = match message_type {
                                MessageTypeId::TypeHostCommandConfigurePeripheral => {
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
                                MessageTypeId::TypeHostCommandConfigureService => {
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
                                MessageTypeId::TypeHostCommandConfigureCharacteristic => {
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
                                MessageTypeId::TypeHostCommandConfigureCharacteristicRead => {
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
                                MessageTypeId::TypeHostCommandNotifyCharacteristicValue => {
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
                                MessageTypeId::TypeHostCommandGetServiceInfo => {
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
                                MessageTypeId::TypeHostCommandGetCharacteristicInfo => {
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
                                MessageTypeId::TypeHostCommandStartAdvertisement => {
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

                                MessageTypeId::TypeHostCommandConfigurePeripheralSecurity => {
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
                                MessageTypeId::TypeHostCommandConfigureProfile => {
                                    match data.decode::<HostCommandConfigureProfile>() {
                                        Ok(cmd) => {
                                            log::info!("Received USB command: {:?}", cmd);
                                            self.handle_configure_profile(cmd)
                                        }
                                        Err(_) => Err(StateMachineError::FailedToDecodeMessage(
                                            "HostCommandConfigureProfile",
                                        )),
                                    }
                                }
                                MessageTypeId::TypeHostCommandStopAdvertisement => {
                                    match data.decode::<HostCommandStopAdvertisement>() {
                                        Ok(cmd) => {
                                            log::info!("Received USB command: {:?}", cmd);
                                            self.handle_stop_advertisement(cmd)
                                        }
                                        Err(_) => Err(StateMachineError::FailedToDecodeMessage(
                                            "HostCommandStopAdvertisement",
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
                            log::error!("Failed to extract message type ID: {:?}", e);
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
            "Configuring peripheral with name: '{}', address: {:?}",
            cmd.name,
            cmd.addr
        );

        // If we haven't already initialized then we can set the BLE device address
        // otherwise we cannot without resetting the BLE device
        if !IS_INITIALIZED.load(std::sync::atomic::Ordering::SeqCst) {
            IS_INITIALIZED.store(true, std::sync::atomic::Ordering::SeqCst);
            self.ble_device.set_own_addr_type(OwnAddrType::Random);
            let addr = slice_to_array(cmd.addr.as_slice()).map_err(|_| {
                log::error!("Invalid address length: must be 6 bytes");
                StateMachineError::InvalidBleConfiguration
            })?;
            self.ble_device.set_rnd_addr(addr).map_err(|_| {
                log::error!("Failed to set random address for BLE device");
                StateMachineError::UnableToSetRNDAddress
            })?;
        }

        let name: heapless::String<30> = heapless::String::try_from(cmd.name.as_str())
            .map_err(|_| StateMachineError::InvalidBleConfiguration)?;
        self.metadata.set_name(&mut self.ns, name);
        self.server = Some(
            self.ble_device
                .get_server()
                .advertise_on_disconnect(false)
                .clear_services(),
        );

        self.clear_all_services_and_metadata();
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
                self.sender
                    .send(PluginConfigurationError { error_type: PluginConfigurationErrorType::AdvertisementWithoutPeripheralConfiguration as _ })
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

                let sender = self.sender.clone();
                server.on_authentication_complete(move |_, desc, status| {
                    log::info!("Authentication completed for client: {:?}", desc);
                    let addr = desc.address().as_be_bytes();
                    let response = PluginAuthenticationCompletedResponse {
                        address: addr.to_vec(),
                        address_type: Self::ble_address_type_to_bluetooth_address_type(
                            desc.address().addr_type(),
                        ) as _,
                        success: status.is_ok(),
                    };
                    sender
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
                self.sender
                    .send(PluginConfigurationError {
                        error_type:
                            PluginConfigurationErrorType::ServiceWithoutPeripheralConfiguration
                                as _,
                    })
                    .map_err(|_| StateMachineError::UsbSendError)?;
                return Err(StateMachineError::ServerNotInitialized);
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
                StateMachineError::InvalidBleConfiguration
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
                StateMachineError::InvalidBleConfiguration
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
                                        Self::bluetooth_address_type_to_ble_address_type(addr_type),
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
                self.sender
                    .send(PluginConfigurationError {
                        error_type:
                            PluginConfigurationErrorType::ServiceWithoutPeripheralConfiguration
                                as _,
                    })
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
                StateMachineError::InvalidBleConfiguration
            })?
            .lock();

        let characteristic = block_on(service.get_characteristic(BleUuid::Uuid16(cmd.uuid as u16)))
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
            StateMachineError::InvalidBleConfiguration
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
                        StateMachineError::CharacteristicUuidStorageError
                    })?;
            }
        }

        match nimble_properties.contains(NimbleProperties::WRITE) {
            true => {
                let char_uuid_write = cmd.uuid;
                let service_uuid_write = cmd.service_uuid;
                let sender = self.sender.clone();
                characteristic.lock().on_write(move |args| {
                    log::info!(
                        "BLE write received for characteristic {} in service {}: {:?} bytes",
                        char_uuid_write,
                        service_uuid_write,
                        args.recv_data()
                    );
                    sender
                        .send(PluginData {
                            src_addr: args.desc().address().as_be_bytes().as_ref().to_vec(),
                            src_addr_type: Self::ble_address_type_to_bluetooth_address_type(
                                args.desc().address().addr_type(),
                            ) as _,
                            send_type: protocol::protocol::PluginDataSendType::WriteType as _,
                            characteristic_uuid: char_uuid_write,
                            service_uuid: service_uuid_write,
                            data: args.recv_data().to_vec(),
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
                let sender = self.sender.clone();
                characteristic.lock().on_read(move |_, desc| {
                    log::info!(
                        "BLE read requested for characteristic {} in service {}",
                        cmd.uuid,
                        cmd.service_uuid
                    );

                    sender
                        .send(PluginData {
                            src_addr: desc.address().as_be_bytes().as_ref().to_vec(),
                            src_addr_type: Self::ble_address_type_to_bluetooth_address_type(
                                desc.address().addr_type(),
                            ) as _,
                            send_type: protocol::protocol::PluginDataSendType::ReadType as _,
                            characteristic_uuid: cmd.uuid,
                            service_uuid: cmd.service_uuid,
                            data: Vec::new(),
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
        address_type: protocol::protocol::BluetoothAddressType,
    ) -> BLEAddressType {
        match address_type {
            protocol::protocol::BluetoothAddressType::Public => BLEAddressType::Public,
            protocol::protocol::BluetoothAddressType::Random => BLEAddressType::Random,
            protocol::protocol::BluetoothAddressType::PublicId => BLEAddressType::PublicID,
            protocol::protocol::BluetoothAddressType::RandomId => BLEAddressType::RandomID,
            _ => unreachable!(),
        }
    }

    fn ble_address_type_to_bluetooth_address_type(
        address_type: BLEAddressType,
    ) -> protocol::protocol::BluetoothAddressType {
        match address_type {
            BLEAddressType::Public => protocol::protocol::BluetoothAddressType::Public,
            BLEAddressType::Random => protocol::protocol::BluetoothAddressType::Random,
            BLEAddressType::PublicID => protocol::protocol::BluetoothAddressType::PublicId,
            BLEAddressType::RandomID => protocol::protocol::BluetoothAddressType::RandomId,
        }
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

    fn handle_configure_profile(&mut self, cmd: HostCommandConfigureProfile) -> Result<()> {
        log::info!("Configuring BLE profile: {:?}", cmd.profile);

        match BleProfile::try_from(cmd.profile) {
            Ok(BleProfile::Custom) => {
                log::info!("Using custom profile with predefined services and characteristics");
                // Get the server
                let server = match self.server.as_mut() {
                    Some(server) => server,
                    None => {
                        log::error!("No BLE server available. Configure peripheral first.");
                        return Err(StateMachineError::InvalidBleConfiguration);
                    }
                };

                // Restart the server with all predefined services and characteristics
                server.restart(true).map_err(|source| {
                    log::error!("Failed to restart BLE server: {:?}", source);
                    StateMachineError::ServerRestartError(source)
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
                return Err(StateMachineError::InvalidBleConfiguration);
            }
        }

        log::info!("Successfully configured profile {:?} by restarting server with predefined configuration", cmd.profile);
        Ok(())
    }

    fn handle_stop_advertisement(&mut self, _cmd: HostCommandStopAdvertisement) -> Result<()> {
        log::info!("Stopping BLE advertisement");

        self.ble_device
            .get_advertising()
            .lock()
            .stop()
            .map_err(|e| {
                log::error!("Failed to stop advertisement: {:?}", e);
                StateMachineError::AdvertisementError("Failed to stop advertisement")
            })?;

        log::info!("Successfully stopped BLE advertisement");
        Ok(())
    }
}
