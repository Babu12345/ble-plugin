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
//! - **Magic Number**: 0xDE for message integrity validation
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

use plugin_config::plugin::PluginReceiver;
use plugin_config::BlinkState;
use plugin_config::HardwareAccessories;
use plugin_config::PluginConfig;

use protocol::protocol::{
    HostCommandConfigureCharacteristic, HostCommandConfigureCharacteristicRead,
    HostCommandConfigurePeripheral, HostCommandConfigurePeripheralSecurity,
    HostCommandConfigureProfile, HostCommandConfigureService, HostCommandGetCharacteristicInfo,
    HostCommandGetServiceInfo, HostCommandNotifyCharacteristicValue, HostCommandStartAdvertisement,
    HostCommandStopAdvertisement,
};

use std::fmt::Debug;
use std::marker::PhantomData;

use std::time::Duration;

use protocol::protocol::MessageTypeId;
use protocol::DEFAULT_PACKET_SIZE;

/// Internal metadata storage for the plugin state machine
///
/// This structure maintains the current state and configuration of the BLE plugin,
/// including device name, service-characteristic relationships, and connection information.
#[derive(Clone)]
struct PluginStateMachineMetadata {
    /// Processing delay for the state machine
    processing_delay: Duration,
}

impl Default for PluginStateMachineMetadata {
    fn default() -> Self {
        Self {
            processing_delay: Duration::from_millis(1),
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
pub struct PluginStateMachine<CONFIG, ERROR, H>
where
    CONFIG: PluginConfig<ERROR>,
    ERROR: Debug,
    H: HardwareAccessories,
{
    config: CONFIG,
    /// USB receiver for incoming host commands (exclusive access)
    receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,

    /// Internal state and configuration metadata
    metadata: PluginStateMachineMetadata,

    /// Hardware accessories
    accessories: H,

    /// Phantom data needed for the typechecker
    _error: PhantomData<ERROR>,
}

impl<Config, ConfigError, H> PluginStateMachine<Config, ConfigError, H>
where
    Config: PluginConfig<ConfigError>,
    ConfigError: Debug,
    H: HardwareAccessories,
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
        config: Config,
        receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,
        accessories: H,
    ) -> Result<Self, ConfigError> {
        Ok(Self {
            config,
            receiver,
            metadata: Default::default(),
            accessories: accessories,
            _error: PhantomData::<ConfigError>,
        })
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
            std::thread::sleep(self.metadata.processing_delay);
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
                                            self.config.handle_configure_peripheral(cmd).map_err(
                                                |e| StateMachineError::InternalConfigError(e),
                                            )
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
                                            self.config.handle_configure_service(cmd).map_err(|e| {
                                                StateMachineError::InternalConfigError(e)
                                            })
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
                                            self.config
                                                .handle_configure_characteristic(cmd)
                                                .map_err(|e| {
                                                    StateMachineError::InternalConfigError(e)
                                                })
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
                                            self.config
                                                .handle_configure_characteristic_read(cmd)
                                                .map_err(|e| {
                                                    StateMachineError::InternalConfigError(e)
                                                })
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
                                            self.config
                                                .handle_notify_characteristic_value(cmd)
                                                .map_err(|e| {
                                                    StateMachineError::InternalConfigError(e)
                                                })
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
                                            self.config.handle_get_service_info(cmd).map_err(|e| {
                                                StateMachineError::InternalConfigError(e)
                                            })
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
                                            self.config.handle_get_characteristic_info(cmd).map_err(
                                                |e| StateMachineError::InternalConfigError(e),
                                            )
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
                                            self.config.handle_start_advertisement(cmd).map_err(
                                                |e| StateMachineError::InternalConfigError(e),
                                            )
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
                                            self.config
                                                .handle_configure_peripheral_security(cmd)
                                                .map_err(|e| {
                                                    StateMachineError::InternalConfigError(e)
                                                })
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
                                            self.config.handle_configure_profile(cmd).map_err(|e| {
                                                StateMachineError::InternalConfigError(e)
                                            })
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
                                            self.config.handle_stop_advertisement(cmd).map_err(
                                                |e| StateMachineError::InternalConfigError(e),
                                            )
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
                                self.accessories.blink(BlinkState::Failure);
                            } else {
                                self.accessories.blink(BlinkState::Success);
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to extract message type ID: {:?}", e);
                            log::warn!(
                                "Received unrecognized command data from USB, raw data length: {} bytes",
                                data.size()
                            );
                            self.accessories.blink(BlinkState::Failure);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to receive data from USB: {:?}", e);
                    std::thread::sleep(Duration::from_millis(100));
                    self.accessories.blink(BlinkState::Failure);
                }
            }
        }
    }
}
