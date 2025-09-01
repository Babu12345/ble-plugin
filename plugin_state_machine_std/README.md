# Plugin State Machine Standard

A Rust library implementing a complete BLE-USB bridge state machine for ESP32-based BLE plugin devices. This crate provides the core processing logic and state management to facilitate bidirectional data and command transfer between BLE peripherals and USB hosts.

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

### Core Components

1. **PluginStateMachine**: Main state machine handling USB-BLE bridging
2. **MessageDecoder**: Efficient message type extraction and validation
3. **BLE Management**: Device, service, and characteristic configuration
4. **Command Handlers**: Specific handlers for each USB command type

## Usage

### Basic Setup

```rust
use plugin_state_machine_std::PluginStateMachine;
use protocol::plugin::plugin::{PluginSender, PluginReceiver};
use esp32_nimble::BLEDevice;
use std::time::Duration;

// Initialize communication channels with throttle configuration
// Throttle parameters: (interval, max_requests_per_interval)
let throttle_config = (Duration::from_millis(10), 10);
let (usb_sender, usb_receiver) = /* USB channel setup with throttle */;
let ble_device = BLEDevice::take();

// Create state machine
let state_machine = PluginStateMachine::new(
    usb_sender,
    usb_receiver, 
    ble_device
);

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
[Magic (2 bytes)][Type ID (1 byte)][Length (2 bytes)][Payload (limited size)]
```

- **Magic Number**: 0xDEAD for message integrity validation
- **Type ID**: Efficient command routing (0x01-0x7F for host commands, 0x80-0xFF for plugin responses)
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

## Dependencies

- `esp32-nimble`: BLE stack integration
- `esp-idf-svc`: ESP32 system services
- `protocol`: Shared protocol definitions (Protocol Buffers)
- `prost`: Protocol Buffers implementation for Rust
- `heapless`: No-allocation structures where possible
- `uuid`: UUID handling
- `log`: Logging support
