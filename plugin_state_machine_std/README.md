# Plugin State Machine Standard

A hardware-agnostic Rust library implementing a complete BLE-USB bridge state machine for BLE plugin devices. This crate provides the core processing logic and state management to facilitate bidirectional data and command transfer between BLE peripherals and USB hosts.

## Hardware Agnostic Design

The state machine is designed to be hardware-agnostic through the use of traits, allowing it to work with any BLE stack implementation. The library uses two key traits:

- **`PluginConfig<ERROR>`**: Defines the interface for BLE stack operations (peripheral configuration, services, characteristics, advertising, etc.)
- **`HardwareAccessories`**: Provides hardware-specific functionality like LED indicators

This design allows the same state machine core to support multiple BLE stacks (ESP32-Nimble, BlueZ, etc.) and hardware platforms by simply implementing these traits.

## Overview

The `plugin_state_machine_std` crate serves as the central processing unit for BLE plugin devices, handling:

- **USB Command Processing**: Receives and processes host commands over USB
- **BLE Device Management**: Configures and manages BLE peripherals, services, and characteristics
- **Bidirectional Communication**: Facilitates data transfer between USB hosts and BLE clients
- **Message Type Dispatch**: Uses efficient message type ID-based command routing
- **State Management**: Maintains device configuration and connection state

## Architecture

The state machine operates as a bridge between two communication domains:

```
USB Host ←→ Plugin State Machine ←→ BLE Peripheral/Central
```

### State Machine Diagram

```
                    ┌─────────────────────────────────────────────────────────┐
                    │                   INITIALIZATION                        │
                    │                                                         │
                    │  - BLEDevice::take()                                    │
                    │  - Initialize NVS partition                             │
                    │  - Create USB channels (sender/receiver)                │
                    │  - Start runner thread                                  │
                    └────────────────────┬────────────────────────────────────┘
                                         │
                                         ▼
                    ┌─────────────────────────────────────────────────────────┐
                    │               UNCONFIGURED STATE                        │
                    │                                                         │
                    │  - No BLE server initialized                            │
                    │  - No peripheral name set                               │
                    │  - Waiting for ConfigurePeripheral command              │
                    └────────────────────┬────────────────────────────────────┘
                                         │
                                         │ HostCommandConfigurePeripheral
                                         │ (name, address)
                                         │
                                         ▼
                    ┌─────────────────────────────────────────────────────────┐
                    │              PERIPHERAL CONFIGURED                      │
                    │                                                         │
                    │  - BLE server created                                   │
                    │  - Device name persisted to NVS                         │
                    │  - Random address set                                   │
                    │  - Ready for service/profile configuration              │
                    └───┬─────────────────────────────────┬───────────────────┘
                        │                                 │
                        │ ConfigureService                │ ConfigureProfile
                        │                                 │
                        ▼                                 ▼
         ┌──────────────────────────────┐    ┌──────────────────────────────┐
         │    MANUAL CONFIGURATION      │    │    PROFILE CONFIGURATION     │
         │                              │    │                              │
         │  - ConfigureService          │    │  - ConfigureProfile          │
         │  - ConfigureCharacteristic   │    │  - Auto-loads predefined     │
         │  - ConfigureCharRead         │    │    services/characteristics  │
         │  - GetServiceInfo            │    │  - Server restart            │
         │  - GetCharacteristicInfo     │    │                              │
         └──────────────┬───────────────┘    └──────────┬───────────────────┘
                        │                               │
                        │                               │
                        └───────────┬───────────────────┘
                                    │
                                    │ HostCommandStartAdvertisement
                                    │ (allow_multi_connect)
                                    │
                                    ▼
                    ┌─────────────────────────────────────────────────────────┐
                    │                  ADVERTISING                            │
                    │                                                         │
                    │  - Broadcasting device name                             │
                    │  - Advertising service UUIDs                            │
                    │  - Waiting for client connections                       │
                    │  - on_connect callback registered                       │
                    │  - on_disconnect callback registered                    │
                    │  - on_authentication_complete callback registered       │
                    └────────────────────┬────────────────────────────────────┘
                                         │
                                         │ BLE Client Connects
                                         │
                                         ▼
                    ┌─────────────────────────────────────────────────────────┐
                    │                CONNECTED STATE                          │
                    │                                                         │
                    │  - Send PluginOnConnectResponse to USB                  │
                    │  - Handle characteristic operations                     │
                    │  - Process security/authentication                      │
                    │  - Re-advertise if multi-connect enabled                │
                    └─────┬──────────────────────────┬────────────────────────┘
                          │                          │
                          │ BLE Operations           │ Client Disconnects
                          │                          │
                          ▼                          ▼
         ┌────────────────────────────────────┐   ┌──────────────────────┐
         │    BIDIRECTIONAL DATA FLOW         │   │  DISCONNECT HANDLER  │
         │                                    │   │                      │
         │  USB → BLE:                        │   │  - Log disconnect    │
         │   - NotifyCharacteristicValue      │   │  - Cleanup state     │
         │   - ConfigureCharacteristicRead    │   │  - Return to         │
         │                                    │   │    ADVERTISING       │
         │  BLE → USB:                        │   │    (if enabled)      │
         │   - PluginData (Write events)      │   │                      │
         │   - PluginData (Read requests)     │   └──────────────────────┘
         │   - Authentication events          │
         │                                    │
         │  Security:                         │
         │   - ConfigurePeripheralSecurity    │
         │   - Passkey validation             │
         │   - Authentication callbacks       │
         └────────────────────────────────────┘

         ┌─────────────────────────────────────────────────────────────────┐
         │                    COMMAND PROCESSING LOOP                      │
         │                                                                 │
         │  1. Receive USB data (with throttling)                          │
         │  2. Extract message type ID (O(1) dispatch)                     │
         │  3. Deserialize command using protobuf                          │
         │  4. Execute handler for message type                            │
         │  5. Send response/error to USB                                  │
         │  6. Blink LED indicator (success/failure)                       │
         │  7. Sleep for processing_delay                                  │
         │  8. Repeat                                                      │
         └─────────────────────────────────────────────────────────────────┘

         ┌─────────────────────────────────────────────────────────────────┐
         │                       ERROR HANDLING                            │
         │                                                                 │
         │  - InvalidMessageFormat → Log + Blink(Failure)                  │
         │  - UnknownMessageType → Log + Blink(Failure)                    │
         │  - ServerNotInitialized → Send PluginConfigurationError         │
         │  - InvalidBleConfiguration → Send PluginConfigurationError      │
         │  - UsbSendError → Log error                                     │
         │  - NvsWriteError → Log error, continue operation                │
         └─────────────────────────────────────────────────────────────────┘

         ┌─────────────────────────────────────────────────────────────────┐
         │                  SUPPORTED STATE TRANSITIONS                    │
         │                                                                 │
         │  UNCONFIGURED → PERIPHERAL_CONFIGURED                           │
         │  PERIPHERAL_CONFIGURED → MANUAL_CONFIG                          │
         │  PERIPHERAL_CONFIGURED → PROFILE_CONFIG                         │
         │  MANUAL_CONFIG → ADVERTISING                                    │
         │  PROFILE_CONFIG → ADVERTISING                                   │
         │  ADVERTISING → CONNECTED                                        │
         │  CONNECTED → ADVERTISING (on disconnect)                        │
         │  ADVERTISING → UNCONFIGURED (StopAdvertisement)                 │
         │  PERIPHERAL_CONFIGURED → UNCONFIGURED (reconfigure)             │
         └─────────────────────────────────────────────────────────────────┘
```

### Core Components

1. **PluginStateMachine**: Main state machine handling USB-BLE bridging
2. **MessageDecoder**: Efficient message type extraction and validation
3. **BLE Management**: Device, service, and characteristic configuration
4. **Command Handlers**: Specific handlers for each USB command type

## Usage

### Basic Setup

```rust
use plugin_state_machine_std::PluginStateMachine;
use plugin_config::{PluginConfig, HardwareAccessories};
use protocol::plugin::plugin::PluginReceiver;

// Implement the PluginConfig trait for your specific BLE stack
struct MyBleConfig {
    // Your BLE stack specific fields
}

impl PluginConfig<MyError> for MyBleConfig {
    fn handle_configure_peripheral(&mut self, cmd: HostCommandConfigurePeripheral) -> Result<(), MyError> {
        // Your BLE-specific implementation
    }
    // ... implement other trait methods
}

// Implement HardwareAccessories for your hardware
struct MyHardwareAccessories;

impl HardwareAccessories for MyHardwareAccessories {
    fn blink(&mut self, state: BlinkState) {
        // Your hardware-specific LED control
    }
}

// Create the state machine with your implementations
let config = MyBleConfig::new(/* ... */);
let receiver = /* USB channel setup */;
let accessories = MyHardwareAccessories;

let state_machine = PluginStateMachine::new(
    config,
    receiver,
    accessories
)?;

// Run the state machine (typically in a separate thread)
let runner = state_machine.runner_fn();
std::thread::spawn(runner);
```

### Message Processing Flow

The state machine processes incoming USB commands using message type IDs for efficient dispatch:

1. **Message Reception**: Receives raw USB data with throttle protection
2. **Throttle Check**: Applies rate limiting to prevent buffer overflow
3. **Header Validation**: Validates magic number and extracts message type ID
4. **Type-Based Dispatch**: Routes to appropriate handler based on message type
5. **Command Processing**: Executes BLE operations
6. **Response Generation**: Sends responses back over USB

#### Data Throttling

The state machine implements input throttling to ensure stable data processing:

- **Rate Limiting**: Configurable throttle with interval and max requests per interval
- **Overflow Protection**: Prevents buffer overflow by dropping excess messages
- **Logging**: Warns when throttle limit is reached for debugging
- **Configuration**: Set via USB processor initialization (e.g., 10 requests per 10ms)

## Supported Commands

### Peripheral Configuration

- **ConfigurePeripheral**: Set up BLE peripheral with name and 6-byte address
- **StartAdvertisement**: Begin BLE advertising with optional multi-connect support

### Service Management

- **ConfigureService**: Create BLE services with specified u16 UUIDs
- **GetServiceInfo**: Retrieve service information and characteristic lists

### Characteristic Operations

- **ConfigureCharacteristic**: Create characteristics with specified properties
- **ConfigureCharacteristicRead**: Set up read operations with default values
- **GetCharacteristicInfo**: Retrieve characteristic properties and status
- **NotifyCharacteristicValue**: Send notifications to connected clients

## Message Protocol

The state machine uses a 5-byte message header format:

```
[Magic (1 byte)][Type ID (2 bytes)][Length (2 bytes)][Payload (limited size)]
```

- **Magic Number**: 0xDE for message integrity validation
- **Type ID**: Efficient command routing using message type ids
- **Length**: Payload size for proper deserialization
- **Payload**: Serialized command/response data using Protocol Buffers

**Size Constraints**: The total message size (header + payload) cannot exceed `DEFAULT_PACKET_SIZE`. With a 5-byte header, the maximum payload size is `DEFAULT_PACKET_SIZE` - 5 bytes.

## Error Handling

The state machine provides comprehensive error handling:

- **InvalidMessageFormat**: Malformed USB messages
- **UnknownMessageType**: Unsupported command types
- **InvalidBleConfiguration**: BLE setup errors
- **UsbSendError**: USB communication failures
- **ServerNotInitialized**: BLE server not ready

## Thread Safety

The state machine is designed for single-threaded operation but uses thread-safe communication channels:

- **USB Sender**: Arc-wrapped for sharing across BLE callbacks
- **USB Receiver**: Exclusive access for command processing
- **BLE Device**: Static mutable reference for ESP32 integration

## Configuration Management

The state machine maintains internal metadata:

- **Peripheral Name**: BLE device advertising name (persisted to NVS)
- **Service-Characteristic Mapping**: UUID relationships for efficient lookups
- **Connection State**: Active client connections and capabilities

### Non-Volatile Storage (NVS)

The state machine leverages ESP32's NVS subsystem for persistent configuration:

- **Automatic Persistence**: Device configurations survive power cycles and resets
- **Current Storage**: BLE device name is automatically saved when configured
- **Namespace Isolation**: Uses dedicated `ConfigNamespace` to prevent conflicts
- **Future Ready**: Infrastructure supports expansion for service configs, security settings, and custom data

## Integration with ESP32-Nimble

The crate integrates deeply with the ESP32-Nimble BLE stack:

- **Security Configuration**: Authentication and encryption setup
- **Service Creation**: Dynamic BLE service and characteristic creation
- **Callback Management**: BLE event handling with USB forwarding
- **Connection Handling**: Multi-connect support and connection lifecycle

## Performance Considerations

- **Message Type Dispatch**: O(1) command routing using type IDs
- **Memory Efficient**: Uses heapless collections where possible for static allocation
- **Minimal Allocations**: Stack-based processing where possible
- **Async-Ready**: Compatible with ESP-IDF async runtime
- **Data Throttling**: Input rate limiting to prevent buffer overflow and ensure stable processing

## Trait-Based Architecture

The state machine's hardware-agnostic design is built on two core traits from the `plugin_config` crate:

### PluginConfig Trait

The `PluginConfig<ERROR>` trait defines all BLE operations that must be implemented for your specific hardware:

```rust
pub trait PluginConfig<ERROR: Debug> {
    fn handle_configure_peripheral(&mut self, cmd: HostCommandConfigurePeripheral) -> Result<(), ERROR>;
    fn handle_configure_peripheral_security(&mut self, cmd: HostCommandConfigurePeripheralSecurity) -> Result<(), ERROR>;
    fn handle_start_advertisement(&mut self, cmd: HostCommandStartAdvertisement) -> Result<(), ERROR>;
    fn handle_stop_advertisement(&mut self, cmd: HostCommandStopAdvertisement) -> Result<(), ERROR>;
    fn handle_configure_service(&mut self, cmd: HostCommandConfigureService) -> Result<(), ERROR>;
    fn handle_configure_characteristic(&mut self, cmd: HostCommandConfigureCharacteristic) -> Result<(), ERROR>;
    fn handle_configure_characteristic_read(&mut self, cmd: HostCommandConfigureCharacteristicRead) -> Result<(), ERROR>;
    fn handle_notify_characteristic_value(&mut self, cmd: HostCommandNotifyCharacteristicValue) -> Result<(), ERROR>;
    fn handle_get_service_info(&mut self, cmd: HostCommandGetServiceInfo) -> Result<(), ERROR>;
    fn handle_get_characteristic_info(&mut self, cmd: HostCommandGetCharacteristicInfo) -> Result<(), ERROR>;
    fn handle_configure_profile(&mut self, cmd: HostCommandConfigureProfile) -> Result<(), ERROR>;
}
```

### HardwareAccessories Trait

The `HardwareAccessories` trait provides hardware-specific functionality:

```rust
pub trait HardwareAccessories {
    fn blink(&mut self, state: BlinkState);
}
```

### Example Implementations

- **ESP32-Nimble**: See `esp_nimble_plugin_config` crate for ESP32-Nimble BLE stack implementation
- **Custom Implementation**: Implement these traits for your BLE stack (BlueZ, Apache Mynewt, Zephyr, etc.)

## Dependencies

- `plugin_config`: Core traits for hardware abstraction
- `protocol`: Shared protocol definitions (Protocol Buffers)
- `log`: Logging support
