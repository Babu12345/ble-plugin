# Protocol IO - Attribute Macros for BLE Plugin Protocol

A procedural macro crate providing convenient attribute macros for implementing protocol I/O traits in the BLE plugin communication system. This crate automatically generates trait implementations for `HostIO`, `PluginIO`, and `MessageType`, reducing boilerplate and ensuring consistent protocol handling.

## Overview

The BLE plugin protocol distinguishes between two types of communication:
- **Host I/O**: Messages sent from host devices (PCs, mobile devices, embedded devices) to plugin devices
- **Plugin I/O**: Messages sent from plugin devices back to host devices

This crate provides attribute macros that automatically implement the appropriate I/O traits based on the message direction, handling lifetime parameters and generic constraints correctly.

## Key Features

- **Automatic Trait Implementation**: Generates `IOBase`, `IO`, `HostIO`/`PluginIO`, and `MessageType` traits
- **Consolidated API**: Single attribute combines trait implementation and message type ID
- **Lifetime Handling**: Correctly manages lifetime parameters in generic types
- **Zero Runtime Cost**: Pure compile-time code generation
- **Type Safety**: Ensures protocol trait consistency at compile time
- **Minimal Dependencies**: Lightweight procedural macro implementation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
protocol_io = { path = "../protocol_io" }
protocol = { path = "../protocol" }
serde = { version = "1.0", features = ["derive"] }
```

## Attribute Macros

### `#[HostIO(MessageTypeId)]`

Implements `IOBase<'a>`, `IO<'a>`, `HostIO<'a>`, and `MessageType` traits for message types sent from hosts to plugins. Use this for command messages that configure or control the BLE plugin device.

**Syntax:**
```rust
use protocol_io::HostIO;
use serde::{Serialize, Deserialize};
use protocol::MessageTypeId;

#[derive(Serialize, Deserialize)]
#[HostIO(MessageTypeId::HostCommandConfigurePeripheral)]
struct ConfigurePeripheralCommand {
    name: String,
    uuid: String,
}

// Automatically implements IOBase<'a>, IO<'a>, HostIO<'a>, and MessageType
```

### `#[PluginIO(MessageTypeId)]`

Implements `IOBase<'a>`, `IO<'a>`, `PluginIO<'a>`, and `MessageType` traits for message types sent from plugins to hosts. Use this for response messages, data forwarding, and error notifications.

**Syntax:**
```rust
use protocol_io::PluginIO;
use serde::{Serialize, Deserialize};
use protocol::MessageTypeId;

#[derive(Serialize, Deserialize)]
#[PluginIO(MessageTypeId::PluginServiceInfoResponse)]
struct ServiceInfoResponse {
    service_uuid: String,
    characteristics: Vec<String>,
    exists: bool,
}

// Automatically implements IOBase<'a>, IO<'a>, PluginIO<'a>, and MessageType
```

## Advanced Usage

### Generic Types with Lifetimes

The macros correctly handle generic types with lifetime parameters:

```rust
use protocol_io::HostIO;
use serde::{Serialize, Deserialize};
use protocol::MessageTypeId;

#[derive(Serialize, Deserialize)]
#[HostIO(MessageTypeId::HostCommandConfigureCharacteristic)]
struct GenericCommand<'a> {
    data: &'a [u8],
    name: &'a str,
}

// Correctly implements IOBase<'a>, IO<'a> and HostIO<'a> with proper lifetime bounds
```

### Multiple Lifetimes

For types with multiple lifetimes, the macro uses the first lifetime parameter:

```rust
use protocol_io::PluginIO;
use serde::{Serialize, Deserialize};
use protocol::MessageTypeId;

#[derive(Serialize, Deserialize)]
#[PluginIO(MessageTypeId::PluginData)]
struct MultiLifetimeResponse<'a, 'b> {
    primary: &'a str,
    secondary: &'b [u8],
}

// Generates: IOBase<'a>, IO<'a>, PluginIO<'a>, and MessageType implementations
// Note: Uses 'a (first lifetime parameter) for the IO traits
```

## Code Generation

The attribute macros generate implementations that look like this:

### For `#[HostIO(MessageTypeId::SomeCommand)]`:
```rust
impl<'a> IOBase<'a> for YourType {}
impl<'a> IO<'a> for YourType {}
impl<'a> HostIO<'a> for YourType {}
impl MessageType for YourType {
    const MESSAGE_TYPE_ID: MessageTypeId = MessageTypeId::SomeCommand;
}
```

### For `#[PluginIO(MessageTypeId::SomeResponse)]`:
```rust
impl<'a> IOBase<'a> for YourType {}
impl<'a> IO<'a> for YourType {}
impl<'a> PluginIO<'a> for YourType {}
impl MessageType for YourType {
    const MESSAGE_TYPE_ID: MessageTypeId = MessageTypeId::SomeResponse;
}
```

### With Existing Lifetimes:
```rust
// For a type like: struct MyType<'a, 'b> { ... }
impl<'a, 'b> IOBase<'a> for MyType<'a, 'b> {}
impl<'a, 'b> IO<'a> for MyType<'a, 'b> {}
impl<'a, 'b> HostIO<'a> for MyType<'a, 'b> {}
impl<'a, 'b> MessageType for MyType<'a, 'b> {
    const MESSAGE_TYPE_ID: MessageTypeId = MessageTypeId::YourVariant;
}
```

## IOBase Trait

The `IOBase` trait is the foundational trait that provides the serialization constraints required for protocol I/O operations. It is automatically implemented by the attribute macros and serves as the basis for the more specialized `IO`, `HostIO`, and `PluginIO` traits.

### What is IOBase?

`IOBase` defines the core trait bounds that message types must satisfy to participate in the protocol:

- **Serde Support**: `Serialize + Deserialize<'a>` for data serialization
- **Message Type**: `MessageType` trait for type identification  
- **Sizing**: `Sized` constraint for stack allocation
- **Protocol Buffer Support**: Either `prost::Message + Default` (when using `protocol_buffer` feature) or `MessageWrite + MessageRead<'a> + Default` (when using `quick_protocol_buffer` feature)

### Feature-Dependent Definitions

The exact definition of `IOBase` depends on your chosen serialization backend:

**With `protocol_buffer` feature (default):**
```rust
pub trait IOBase<'a>:
    Serialize + Deserialize<'a> + Sized + MessageType + prost::Message + Default
{
}
```

**With `quick_protocol_buffer` feature:**
```rust
pub trait IOBase<'a>:
    Serialize + Deserialize<'a> + Sized + MessageType + Default + MessageWrite + MessageRead<'a>
{
}
```

### Why IOBase Matters

- **Trait Hierarchy**: `IOBase` → `IO` → `HostIO`/`PluginIO` provides a clean inheritance structure
- **Compile-time Safety**: Ensures all required traits are implemented before any I/O operations
- **Flexible Serialization**: Abstracts over different protocol buffer implementations
- **Zero Runtime Cost**: All constraints are resolved at compile-time

### Implementation

You don't need to implement `IOBase` manually. The `#[HostIO(...)]` and `#[PluginIO(...)]` attribute macros automatically generate the implementation for you, along with all the other required traits.

## Usage Guidelines

### When to Use `#[HostIO(MessageTypeId)]`

Use this attribute for:
- Configuration commands (peripheral, service, characteristic setup)
- Control commands (start/stop advertising, notifications)
- Query commands (get service/characteristic information)
- Any message sent from host to plugin device

### When to Use `#[PluginIO(MessageTypeId)]`

Use this attribute for:
- Configuration responses (success/error status)
- Data forwarding from BLE clients to host
- Service and characteristic information responses
- Error notifications and status updates
- Any message sent from plugin device to host

## Requirements

To use these attribute macros, your types must:

1. **Implement Serde traits**: `#[derive(Serialize, Deserialize)]`
2. **Use appropriate attribute**: `#[HostIO(...)]` for host→plugin, `#[PluginIO(...)]` for plugin→host
3. **Provide MessageTypeId**: Pass the appropriate `MessageTypeId` variant as the attribute parameter

## Error Handling

The macros perform compile-time validation and will produce clear error messages if used incorrectly. Common issues include:

### Missing Serde Derives
```rust
// ❌ This will fail
#[HostIO(MessageTypeId::HostCommandConfigurePeripheral)]
struct BadCommand {
    data: String,
}

// ✅ This will work
#[derive(Serialize, Deserialize)]
#[HostIO(MessageTypeId::HostCommandConfigurePeripheral)]
struct GoodCommand {
    data: String,
}
```

### Missing MessageTypeId Parameter
```rust
// ❌ This will fail - missing MessageTypeId parameter
#[derive(Serialize, Deserialize)]
#[HostIO]
struct BadCommand {
    data: String,
}

// ✅ This will work
#[derive(Serialize, Deserialize)]
#[HostIO(MessageTypeId::HostCommandConfigurePeripheral)]
struct GoodCommand {
    data: String,
}
```

## Integration Example

Here's a complete example showing how to integrate with the protocol system:

```rust
// In your Cargo.toml
[dependencies]
protocol = { path = "../protocol" }
protocol_io = { path = "../protocol_io" }
serde = { version = "1.0", features = ["derive"] }

// In your code
use protocol_io::{HostIO, PluginIO};
use protocol::{MessageTypeId, IO};
use serde::{Serialize, Deserialize};

// Host command
#[derive(Serialize, Deserialize)]
#[HostIO(MessageTypeId::HostCommandStartAdvertisement)]
struct StartAdvertising {
    allow_multi_connect: bool,
}

// Plugin response
#[derive(Serialize, Deserialize)]
#[PluginIO(MessageTypeId::PluginConfigurationError)]
struct ConfigurationError {
    error_type: String,
    message: String,
}

// Usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = StartAdvertising {
        allow_multi_connect: true,
    };
    
    // Serialize command with protocol header
    let serialized = command.to_bytes::<256>()?;
    
    // Deserialize response
    let response_data: &[u8] = &[/* received data */];
    let error = ConfigurationError::from_bytes(response_data)?;
    
    Ok(())
}
```

## Testing

The crate includes comprehensive integration tests covering all lifetime scenarios:

```bash
cargo test
```

Tests include:
- Zero lifetimes (simple structs and enums)
- Single lifetime parameters
- Multiple lifetime parameters  
- Generic types with lifetimes
- Edge cases and error conditions

## Contributing

When contributing to this crate:

1. **Add tests** for new functionality in the integration test suite
2. **Update documentation** for any API changes
3. **Ensure backward compatibility** when possible
4. **Follow Rust API guidelines** for procedural macros

## Implementation Details

### Lifetime Parameter Detection

The attribute macros inspect the generic parameters of the target type and detect lifetime parameters. If a lifetime is found, it uses the first lifetime parameter for the trait implementations. This ensures compatibility with both simple types and complex generic types.

### Generated Code Quality

The generated code is minimal and efficient:
- No runtime overhead
- Minimal trait bounds
- Correct lifetime handling
- Clean, readable output

### Error Messages

The attribute macros are designed to produce helpful error messages when used incorrectly, guiding developers toward correct usage patterns.

## Dependencies

- **proc-macro2**: Core procedural macro utilities
- **quote**: Code generation and token stream manipulation
- **syn**: Rust AST parsing with full feature support

## License

Copyright © 2025 Wanyeki Technologies LLC. All rights reserved.

This project is private and proprietary.