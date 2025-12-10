#![deny(missing_docs)]

// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.


//! # Plugin State Machine Standard
//!
//! A hardware-agnostic BLE-USB bridge state machine implementation for BLE plugin devices.
//!
//! This library provides the core processing logic and state management required to facilitate
//! bidirectional data and command transfer between BLE peripherals and USB hosts. It serves as
//! the central processing unit for BLE plugin devices, handling USB command processing, BLE
//! device management, and efficient message routing.
//!
//! ## Hardware Agnostic Design
//!
//! The state machine is designed to work with any BLE stack through a trait-based architecture:
//!
//! - **[`PluginConfig<ERROR>`]**: Trait defining BLE operations (peripheral config, services, characteristics, etc.)
//! - **[`HardwareAccessories`]**: Trait for hardware-specific functionality (LED indicators, etc.)
//!
//! This design allows the same state machine core to support multiple BLE stacks (ESP32-Nimble,
//! BlueZ, Apache Mynewt, Zephyr, etc.) and hardware platforms by simply implementing these traits.
//!
//! ## Key Features
//!
//! - **Hardware Agnostic**: Trait-based design works with any BLE stack implementation
//! - **Efficient Message Dispatch**: Uses message type IDs for O(1) command routing
//! - **Protocol Validation**: Magic number validation and header integrity checking
//! - **Flexible BLE Integration**: Support for any BLE stack through [`PluginConfig`] trait
//! - **Thread-Safe Communication**: USB communication channels for safe concurrent access
//! - **Comprehensive Error Handling**: Detailed error types for robust operation
//! - **Memory Efficient**: Optimized for embedded systems
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
//! use plugin_config::{PluginConfig, HardwareAccessories, BlinkState};
//! use protocol::plugin::plugin::PluginReceiver;
//! use protocol::DEFAULT_PACKET_SIZE;
//!
//! // Step 1: Implement PluginConfig for your BLE stack
//! struct MyBleConfig {
//!     // Your BLE stack specific fields
//! }
//!
//! impl PluginConfig<MyError> for MyBleConfig {
//!     fn handle_configure_peripheral(
//!         &mut self,
//!         cmd: HostCommandConfigurePeripheral
//!     ) -> Result<(), MyError> {
//!         // Your BLE-specific implementation
//!         Ok(())
//!     }
//!     // Implement other trait methods...
//! }
//!
//! // Step 2: Implement HardwareAccessories
//! struct MyHardware;
//!
//! impl HardwareAccessories for MyHardware {
//!     fn blink(&mut self, state: BlinkState) {
//!         // Your hardware-specific LED control
//!     }
//! }
//!
//! # struct MyError;
//! # impl std::fmt::Debug for MyError { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }}
//! # let receiver: PluginReceiver<DEFAULT_PACKET_SIZE> = panic!();
//! // Step 3: Create and run the state machine
//! let config = MyBleConfig { /* ... */ };
//! let accessories = MyHardware;
//!
//! let state_machine = PluginStateMachine::new(config, receiver, accessories)?;
//! let runner = state_machine.runner_fn();
//!
//! // Typically run in a separate thread
//! std::thread::spawn(runner);
//! # Ok::<(), MyError>(())
//! ```
//!
//! ## Trait-Based Architecture
//!
//! The state machine achieves hardware independence through two core traits from [`plugin_config`]:
//!
//! ### PluginConfig Trait
//!
//! The [`PluginConfig<ERROR>`] trait defines all BLE operations:
//!
//! - `handle_configure_peripheral`: Configure BLE peripheral (name, address)
//! - `handle_configure_peripheral_security`: Set up security/authentication
//! - `handle_start_advertisement` / `handle_stop_advertisement`: Control advertising
//! - `handle_configure_service`: Create BLE services
//! - `handle_configure_characteristic`: Create characteristics with properties
//! - `handle_configure_characteristic_read`: Configure read operations
//! - `handle_notify_characteristic_value`: Send notifications to clients
//! - `handle_get_service_info` / `handle_get_characteristic_info`: Query information
//! - `handle_configure_profile`: Load predefined BLE profiles
//!
//! **Note**: All trait methods have default implementations that call `unimplemented!()`,
//! allowing you to implement only the methods you need. Additional methods may be added
//! to this trait in future versions to support new BLE features and operations.
//!
//! ### HardwareAccessories Trait
//!
//! The [`HardwareAccessories`] trait provides hardware-specific functionality:
//!
//! - `blink`: Visual feedback through LED indicators (success/failure states)
//!
//! **Note**: All trait methods have default implementations that call `unimplemented!()`,
//! allowing you to implement only the methods you need. Additional methods may be added
//! to this trait in future versions to support new hardware accessories and functionality.
//!
//! ### Example Implementations
//!
//! - **ESP32-Nimble**: See [`esp_nimble_plugin_config`] crate for ESP32-Nimble implementation
//! - **Custom BLE Stack**: Implement these traits for BlueZ, Apache Mynewt, Zephyr, etc.
//!
//! ### Benefits of Trait-Based Design
//!
//! - **Portability**: Same state machine works across different hardware platforms
//! - **Testability**: Easy to create mock implementations for testing
//! - **Flexibility**: Swap BLE stacks without changing state machine code
//! - **Extensibility**: Add support for new BLE stacks by implementing traits
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
//! - [`StateMachineError::UnhandledMessageType`]: Unsupported command types
//! - [`StateMachineError::FailedToDecodeMessage`]: Message deserialization errors
//! - [`StateMachineError::InternalConfigError`]: Errors from the underlying BLE configuration implementation
//!
//! BLE-specific errors are handled by the [`PluginConfig`] trait implementation and wrapped
//! in [`StateMachineError::InternalConfigError`].
//!
//! ## Performance Characteristics
//!
//! - **Command Routing**: O(1) lookup using message type IDs
//! - **Memory Usage**: Optimized for embedded systems
//! - **Latency**: Minimal processing overhead with direct dispatch
//! - **Throughput**: Efficient binary serialization with bincode
//! - **Hardware Independence**: Zero-cost abstractions through trait monomorphization
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
//! [`StateMachineError::UnhandledMessageType`]: errors::StateMachineError::UnhandledMessageType
//! [`StateMachineError::FailedToDecodeMessage`]: errors::StateMachineError::FailedToDecodeMessage
//! [`StateMachineError::InternalConfigError`]: errors::StateMachineError::InternalConfigError
//! [`PluginConfig<ERROR>`]: plugin_config::PluginConfig
//! [`HardwareAccessories`]: plugin_config::HardwareAccessories
//! [`plugin_config`]: plugin_config

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

/// Internal metadata for the plugin state machine
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
/// ## Hardware Agnostic Design
///
/// This state machine is hardware-agnostic and works with any BLE stack through traits:
/// - **CONFIG**: Generic type implementing [`PluginConfig<ERROR>`] for BLE operations
/// - **H**: Generic type implementing [`HardwareAccessories`] for hardware-specific functions
///
/// ## Architecture
///
/// The state machine operates as a bridge:
/// - **USB Side**: Receives commands from host through receiver
/// - **BLE Side**: Delegates to CONFIG implementation (ESP32-Nimble, BlueZ, etc.)
/// - **Processing**: Efficiently routes messages using type IDs and maintains state
/// - **Hardware**: Uses H for hardware-specific operations (LED indicators, etc.)
///
/// ## Thread Safety
///
/// - USB receiver has exclusive access for command processing
/// - BLE operations are delegated to the CONFIG implementation
/// - Hardware accessories are managed through the H implementation
///
/// ## Usage Pattern
///
/// 1. Create with a CONFIG implementation, USB receiver, and hardware accessories
/// 2. Start the runner (typically in a separate thread)
/// 3. State machine processes commands automatically
/// 4. BLE operations are handled by your CONFIG implementation
pub struct PluginStateMachine<CONFIG, ERROR, H>
where
    CONFIG: PluginConfig<ERROR>,
    ERROR: Debug,
    H: HardwareAccessories,
{
    config: CONFIG,
    /// USB receiver for incoming host commands (exclusive access)
    receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,

    /// Internal state machine metadata
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
    /// Initializes the state machine with the necessary communication channels, BLE configuration,
    /// and hardware accessories. The state machine is hardware-agnostic and works with any BLE
    /// stack that implements the [`PluginConfig`] trait.
    ///
    /// # Type Parameters
    ///
    /// * `Config` - Type implementing [`PluginConfig<ConfigError>`] for BLE operations
    /// * `ConfigError` - Error type used by the BLE configuration implementation
    /// * `H` - Type implementing [`HardwareAccessories`] for hardware-specific operations
    ///
    /// # Arguments
    ///
    /// * `config` - Your BLE stack implementation (ESP32-Nimble, BlueZ, etc.)
    /// * `receiver` - Channel for receiving commands from the USB host
    /// * `accessories` - Hardware accessories implementation for LED indicators, etc.
    ///
    /// # Returns
    ///
    /// A new `PluginStateMachine` instance ready to process commands
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use plugin_state_machine_std::PluginStateMachine;
    /// use plugin_config::{PluginConfig, HardwareAccessories};
    /// use protocol::plugin::plugin::PluginReceiver;
    /// use protocol::DEFAULT_PACKET_SIZE;
    ///
    /// # struct MyBleConfig;
    /// # struct MyError;
    /// # impl std::fmt::Debug for MyError { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }}
    /// # impl PluginConfig<MyError> for MyBleConfig {}
    /// # struct MyHardware;
    /// # impl HardwareAccessories for MyHardware {}
    /// # let receiver: PluginReceiver<DEFAULT_PACKET_SIZE> = panic!();
    /// let config = MyBleConfig { /* ... */ };
    /// let accessories = MyHardware;
    ///
    /// let state_machine = PluginStateMachine::new(config, receiver, accessories)?;
    /// # Ok::<(), MyError>(())
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
                                            log::info!("Received USB command: {cmd:?}");
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
                                            log::info!("Received USB command: {cmd:?}");
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
                                            log::info!("Received USB command: {cmd:?}");
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
                                            log::info!("Received USB command: {cmd:?}");
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
                                            log::info!("Received USB command: {cmd:?}");
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
                                            log::info!("Received USB command: {cmd:?}");
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
                                            log::info!("Received USB command: {cmd:?}");
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
                                            log::info!("Received USB command: {cmd:?}");
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
                                            log::info!("Received USB command: {cmd:?}");
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
                                            log::info!("Received USB command: {cmd:?}");
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
                                            log::info!("Received USB command: {cmd:?}");
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

                            self.accessories.blink({
                                if let Err(e) = result {
                                    log::error!(
                                        "Failed to handle command {message_type:?}: {e:?}",
                                    );
                                    BlinkState::Failure
                                } else {
                                    BlinkState::Success
                                }
                            });
                        }
                        Err(e) => {
                            log::error!("Failed to extract message type ID: {e:?}");
                            log::warn!(
                                "Received unrecognized command data from USB, raw data length: {} bytes",
                                data.size()
                            );
                            self.accessories.blink(BlinkState::Failure);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to receive data from USB: {e:?}");
                    std::thread::sleep(Duration::from_millis(100));
                    self.accessories.blink(BlinkState::Failure);
                }
            }
        }
    }
}
