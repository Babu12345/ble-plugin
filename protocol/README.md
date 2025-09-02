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
- **Protocol Buffers**: Uses protobuf for efficient, cross-platform serialization
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
- **Payload**: Binary serialized message data

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

// Create a peripheral configuration command
let mut addr = Vec::from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);

let command = HostCommandConfigurePeripheral {
    name: String::try_from("MyDevice").unwrap(),
    addr: addr,  // 6-byte BLE address
};
```

### Message Serialization

```rust
use protocol::{IO, DEFAULT_PACKET_SIZE};

// Serialize using protobuf with header
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
- `HostCommandConfigurePeripheral`: Set up device name and BLE address
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

- `prost`: Protocol Buffers implementation for Rust
- `uuid`: UUID handling for BLE identifiers
- `lib_utils`: Utility functions for array operations

## Feature Flags

- `std`: Standard library support (enabled by default)
- `protocol_buffer`: Protocol Buffers serialization support (using prost)
- `quick_protocol_buffer`: Fast Protocol Buffers serialization support (using quick-protobuf)
- `bincode_serialization`: Optional bincode serialization support
- `defmt`: Defmt logging support for embedded systems

## Serialization

The protocol requires **exactly one** primary serialization format to be enabled at compile time:

### Protocol Buffers (using prost)

Enable with the `protocol_buffer` feature flag for robust cross-platform compatibility:

```toml
# In Cargo.toml
[dependencies]
protocol = { path = "../protocol", features = ["protocol_buffer"] }
```

**Protocol Buffers (prost) advantages:**
- **Cross-Platform**: Same message format across Rust, Python, and other languages
- **Schema Evolution**: Forward and backward compatible message evolution
- **Language Agnostic**: Works with any language that supports protobuf
- **Type Safety**: Strong typing with automatic validation
- **Mature Ecosystem**: Well-established protobuf implementation

### Fast Protocol Buffers (using quick-protobuf)

Enable with the `quick_protocol_buffer` feature flag for high-performance protobuf:

```toml
# In Cargo.toml
[dependencies]
protocol = { path = "../protocol", features = ["quick_protocol_buffer"] }
```

**Quick Protocol Buffers advantages:**
- **Higher Performance**: Faster serialization/deserialization than prost
- **Lower Memory Usage**: Reduced allocation overhead
- **Cross-Platform**: Same protobuf compatibility as prost
- **Zero-Copy**: Can deserialize without allocation in some cases
- **Smaller Binary Size**: More compact generated code
- **Embedded Friendly**: Works on targets without atomic CAS operations (unlike prost)

### Optional: Bincode Serialization

Bincode can be added as an **additional** serialization option alongside either protobuf implementation:

```toml
# In Cargo.toml - with prost + bincode
[dependencies]
protocol = { path = "../protocol", default-features = false, features = [
    "protocol_buffer",
    "bincode_serialization"
] }

# OR with quick-protobuf + bincode
[dependencies]  
protocol = { path = "../protocol", default-features = false, features = [
    "quick_protocol_buffer", 
    "bincode_serialization"
] }
```

**Bincode advantages:**
- **Highest Performance**: Fastest serialization for Rust-to-Rust communication
- **Smallest Binary Size**: Most compact encoding for simple data structures
- **Zero-Copy**: Direct deserialization into Rust structs without allocation
- **Native Rust Types**: Direct support for Rust enums, Options, and collections

### Important: Primary Serialization Requirement

⚠️ **Exactly one primary protobuf implementation must be enabled.** The crate will fail to compile if:
- Both `protocol_buffer` and `quick_protocol_buffer` are enabled simultaneously
- Neither `protocol_buffer` nor `quick_protocol_buffer` is enabled

💡 **Tip**: Use `default-features = false` in your Cargo.toml to explicitly control which features are enabled, avoiding conflicts with the default `protocol_buffer` feature.

**When to use each:**
- **Protocol Buffers (prost)**: Maximum cross-language compatibility, mature ecosystem, standard library environments
- **Quick Protocol Buffers**: High-performance protobuf, embedded systems (especially those without atomic CAS support), performance-critical applications
- **Bincode (optional)**: Additional Rust-to-Rust performance optimization

## Compatibility

- **Rust Version**: 1.70+
- **Embedded**: Full no_std support
- **Platforms**: Cross-platform (desktop, mobile, embedded)
- **Atomic Operations**: `quick_protocol_buffer` works on targets without atomic CAS operations; `protocol_buffer` requires atomic support
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
# Test with prost implementation
cargo test --features protocol_buffer

# Test with quick-protobuf implementation  
cargo test --features quick_protocol_buffer

# Test with bincode support
cargo test --features "protocol_buffer,bincode_serialization"
cargo test --features "quick_protocol_buffer,bincode_serialization"

# Test no_std compatibility
cargo test --no-default-features --features protocol_buffer
cargo test --no-default-features --features quick_protocol_buffer
```

## Contributing

When adding new message types:

1. Add the new type ID to `MessageTypeId` enum
2. Create the message struct in `io_types`
3. Implement `MessageType` trait
4. Add comprehensive tests
5. Update documentation


## Protocol Buffer Setup

To work with Protocol Buffers:

```bash
# Install protobuf compiler
brew install protobuf  # macOS
# or
sudo apt-get install protobuf-compiler  # Ubuntu/Debian
```

The `.proto` files are automatically compiled into Rust code during the build process.