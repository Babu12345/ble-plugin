# Protocol - BLE Plugin Communication Protocol

A comprehensive communication protocol library for BLE-USB bridge systems, defining standardized message formats, serialization, and type-safe command/response structures for plugin devices.

## Overview

This library provides the complete communication protocol between host devices (PCs, mobile devices, embedded devices) and BLE plugin devices (ESP32-based bridge devices). It ensures reliable, type-safe communication across the USB-BLE bridge with efficient binary serialization and protocol validation.

## Architecture

```text
┌─────────────────┐     USB Commands     ┌─────────────────┐     BLE Operations      ┌─────────────┐
│   Host Device   │ ──────────────────►  │  Plugin Device  │ ──────────────────────► │ BLE Clients │
│ (PC/Mobile/     │                      │  (ESP32 + BLE)  │                         │             │
│  Embedded)      │ ◄──────────────────  │                 │ ◄────────────────────── │             │
└─────────────────┘     USB Responses    └─────────────────┘     BLE Callbacks       └─────────────┘
```

## Key Features

- **Type-Safe Messages**: Rust type system ensures protocol correctness
- **Efficient Serialization**: Binary serialization using bincode
- **Message Validation**: Magic number and header integrity checking
- **Version Compatibility**: Structured message IDs for protocol evolution
- **Cross-Platform**: Supports both embedded (no_std) and standard environments
- **Extensible Design**: Easy addition of new command and response types

## Message Protocol Format

All messages use a standardized 5-byte header followed by serialized payload:

```text
┌─────────────┬─────────────┬─────────────┬─────────────────┐
│   Magic     │   Type ID   │   Length    │     Payload     │
│  (2 bytes)  │  (1 byte)   │  (2 bytes)  │  (limited size) │
└─────────────┴─────────────┴─────────────┴─────────────────┘
```

- **Magic Number**: 0xDEAD (little-endian) for message integrity validation
- **Type ID**: Unique identifier for each message type (enables O(1) dispatch)
- **Length**: Payload size in bytes (little-endian)
- **Payload**: Bincode-serialized message data

**Size Constraints**: The total message size (header + payload) cannot exceed `DEFAULT_PACKET_SIZE`. With a `MESSAGE_HEADER_SIZE` header, the maximum payload size is `DEFAULT_PACKET_SIZE` - `MESSAGE_HEADER_SIZE` bytes.

## Message Categories

### Host Commands (0x01-0x7F)
Commands sent from host devices to configure and control the BLE plugin:

- **Peripheral Management**: Configure device name, UUID, advertising
- **Service Operations**: Create and manage BLE services
- **Characteristic Control**: Create characteristics with properties
- **Data Operations**: Read/write/notify characteristic values
- **Query Commands**: Get service and characteristic information

### Plugin Responses (0x80-0xFF)
Responses and data sent from plugin devices back to hosts:

- **Configuration Responses**: Success/error status for commands
- **Data Forwarding**: BLE client data forwarded to host
- **Information Responses**: Service and characteristic details
- **Error Notifications**: Detailed error information

## Usage Examples

### Basic Message Creation

```rust
use protocol::io_types::HostCommandConfigurePeripheral;
use heapless::{String, Vec};

// Create a peripheral configuration command
let mut addr = Vec::new();
addr.extend_from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]).unwrap();

let command = HostCommandConfigurePeripheral {
    name: String::try_from("MyDevice").unwrap(),
    addr: addr,  // 6-byte BLE address using heapless::Vec<u8, 6>
};
```

### Message Serialization

```rust
use protocol::{IO, DEFAULT_PACKET_SIZE};

// Serialize to fixed-size buffer with header
let serialized: [u8; DEFAULT_PACKET_SIZE] = command.to_bytes()?;

// Or serialize to provided buffer (no allocation)
let mut buffer = [0u8; DEFAULT_PACKET_SIZE];
command.to_bytes_in_slice(&mut buffer)?;
```

### Message Deserialization

```rust
use protocol::{IO, io_types::HostCommandConfigurePeripheral};

// Deserialize from received bytes (includes header validation)
let received_data: &[u8] = &[/* USB data */];
let command = HostCommandConfigurePeripheral::from_bytes(received_data)?;
```

## Core Modules

- **[`io`]**: Core serialization traits and message header handling
- **[`io_types`]**: All message type definitions and structures  
- **[`host`]**: Host-specific communication utilities
- **[`plugin`]**: Plugin-specific communication channels
- **[`errors`]**: Comprehensive error handling

## Supported Commands

### Peripheral Management
- `HostCommandConfigurePeripheral`: Set up device name and BLE address (heapless::Vec<u8, 6>)
- `HostCommandStartAdvertisement`: Begin BLE advertising
- `HostCommandConfigurePeripheralSecurity`: Configure security settings (pairing, passkey)

### Service Operations
- `HostCommandConfigureService`: Create BLE services
- `HostCommandGetServiceInfo`: Query service information

### Characteristic Management
- `HostCommandConfigureCharacteristic`: Create characteristics with properties
- `HostCommandConfigureCharacteristicRead`: Set up read operations
- `HostCommandGetCharacteristicInfo`: Query characteristic details
- `HostCommandNotifyCharacteristicValue`: Send notifications to clients

### Profile Management
- `HostCommandConfigureProfile`: Configure using predefined BLE profiles
  - Custom (0): User-defined services and characteristics
  - HeartRateMonitor (1): Heart Rate Monitor profile
  - BatteryService (2): Battery Service profile
  - DeviceInformation (3): Device Information Service profile

### Plugin Responses
- `PluginData`: BLE client data forwarded to host
- `PluginConfigurationError`: Error responses from plugin
- `PluginServiceInfoResponse`: Service information with characteristic list
- `PluginCharacteristicInfoResponse`: Characteristic details with properties

## Protocol Constants

- `MAX_NAME_SIZE`: Maximum length for device names (30 characters)
- `DEFAULT_PACKET_SIZE`: Standard USB packet size
- `MESSAGE_HEADER_SIZE`: Protocol header size
- `MESSAGE_MAGIC`: Magic number for validation (0xDEAD)
- **Maximum Payload Size**: `DEFAULT_PACKET_SIZE` - `MESSAGE_HEADER_SIZE` bytes

## Dependencies

- `serde`: Serialization framework
- `bincode`: Efficient binary serialization
- `heapless`: No-allocation collections for embedded systems
- `uuid`: UUID handling for BLE identifiers
- `lib_utils`: Utility functions for array operations

## Feature Flags

- `std`: Standard library support (enabled by default)
- `serde`: Serde serialization support
- `defmt`: Defmt logging support for embedded systems

## Serialization Configuration

The protocol supports configurable serialization methods through cfg settings:

- `bincode_serialization`: Use bincode for binary serialization (default)
  - Efficient binary format with minimal overhead
  - Suitable for embedded systems and high-performance applications
  - Configure with: `cfg(bincode_serialization)`

Future serialization methods can be added by implementing the appropriate cfg flags. This allows switching between different serialization formats without changing the protocol API.

## Compatibility

- **Rust Version**: 1.70+
- **Embedded**: Full no_std support with heapless collections
- **Platforms**: Cross-platform (desktop, mobile, embedded)
- **Endianness**: Little-endian byte order for consistency

## Testing

The protocol includes comprehensive tests covering:

- Message header validation and integrity
- Serialization/deserialization round-trips
- Type ID uniqueness and ranges
- Error handling and edge cases
- Cross-platform compatibility

Run tests with:

```bash
cargo test
cargo test --no-default-features  # Test no_std compatibility
```

## Contributing

When adding new message types:

1. Add the new type ID to `MessageTypeId` enum
2. Create the message struct in `io_types`
3. Implement `MessageType` trait
4. Add comprehensive tests
5. Update documentation


## IDE Configuration

### VS Code / rust-analyzer
To ensure rust-analyzer properly handles the serialization cfg flags:
- Add `"rust-analyzer.cargo.features": [ "bincode_serialization" ],` to your `.vscode/settings.json`
- This prevents rust-analyzer from showing compiler errors for cfg-gated code branches when the appropriate cfg flag is set