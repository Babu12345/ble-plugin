# Protocol IO - Procedural Macros for BLE Plugin Protocol

A procedural macro crate providing convenient derive macros for implementing protocol I/O traits in the BLE plugin communication system. This crate automatically generates trait implementations for `HostIO` and `PluginIO`, reducing boilerplate and ensuring consistent protocol handling.

## Overview

The BLE plugin protocol distinguishes between two types of communication:
- **Host I/O**: Messages sent from host devices (PCs, mobile) to plugin devices
- **Plugin I/O**: Messages sent from plugin devices back to host devices

This crate provides derive macros that automatically implement the appropriate I/O traits based on the message direction, handling lifetime parameters and generic constraints correctly.

## Key Features

- **Automatic Trait Implementation**: Derives `IO`, `HostIO`, and `PluginIO` traits
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

## Derive Macros

### `#[derive(HostIO)]`

Implements `IO<'a>` and `HostIO<'a>` traits for message types sent from hosts to plugins. Use this for command messages that configure or control the BLE plugin device.

**Example:**
```rust
use protocol_io::HostIO;
use serde::{Serialize, Deserialize};
use protocol::{MessageType, MessageTypeId};

#[derive(Serialize, Deserialize, HostIO)]
struct ConfigurePeripheralCommand {
    name: String,
    uuid: String,
}

impl MessageType for ConfigurePeripheralCommand {
    fn message_type_id() -> MessageTypeId {
        MessageTypeId::HostCommandConfigurePeripheral
    }
}

// Now automatically implements IO<'a> and HostIO<'a>
```

### `#[derive(PluginIO)]`

Implements `IO<'a>` and `PluginIO<'a>` traits for message types sent from plugins to hosts. Use this for response messages, data forwarding, and error notifications.

**Example:**
```rust
use protocol_io::PluginIO;
use serde::{Serialize, Deserialize};
use protocol::{MessageType, MessageTypeId};

#[derive(Serialize, Deserialize, PluginIO)]
struct ServiceInfoResponse {
    service_uuid: String,
    characteristics: Vec<String>,
    exists: bool,
}

impl MessageType for ServiceInfoResponse {
    fn message_type_id() -> MessageTypeId {
        MessageTypeId::PluginServiceInfoResponse
    }
}

// Now automatically implements IO<'a> and PluginIO<'a>
```

## Advanced Usage

### Generic Types with Lifetimes

The macros correctly handle generic types with lifetime parameters:

```rust
use protocol_io::HostIO;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, HostIO)]
struct GenericCommand<'a> {
    data: &'a [u8],
    name: &'a str,
}

// Correctly implements IO<'a> and HostIO<'a> with proper lifetime bounds
```

### Complex Generic Types

The macros also work with more complex generic constraints:

```rust
use protocol_io::PluginIO;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PluginIO)]
struct GenericResponse<'a, T>
where
    T: Serialize + Deserialize<'a>,
{
    data: T,
    message: &'a str,
}
```

## Code Generation

The derive macros generate implementations that look like this:

### For `#[derive(HostIO)]`:
```rust
impl<'a> IO<'a> for YourType {}
impl<'a> HostIO<'a> for YourType {}
```

### For `#[derive(PluginIO)]`:
```rust
impl<'a> IO<'a> for YourType {}
impl<'a> PluginIO<'a> for YourType {}
```

### With Existing Lifetimes:
```rust
// For a type like: struct MyType<'a> { data: &'a [u8] }
impl<'a> IO<'a> for MyType<'a> {}
impl<'a> HostIO<'a> for MyType<'a> {}
```

## Usage Guidelines

### When to Use `#[derive(HostIO)]`

Use this derive for:
- Configuration commands (peripheral, service, characteristic setup)
- Control commands (start/stop advertising, notifications)
- Query commands (get service/characteristic information)
- Any message sent from host to plugin device

### When to Use `#[derive(PluginIO)]`

Use this derive for:
- Configuration responses (success/error status)
- Data forwarding from BLE clients to host
- Service and characteristic information responses
- Error notifications and status updates
- Any message sent from plugin device to host

## Requirements

To use these derive macros, your types must:

1. **Implement Serde traits**: `#[derive(Serialize, Deserialize)]`
2. **Implement MessageType**: From the protocol crate
3. **Use appropriate derive**: `HostIO` for host→plugin, `PluginIO` for plugin→host

## Error Handling

The macros perform compile-time validation and will produce clear error messages if used incorrectly. Common issues include:

### Missing Serde Derives
```rust
// ❌ This will fail
#[derive(HostIO)]
struct BadCommand {
    data: String,
}

// ✅ This will work
#[derive(Serialize, Deserialize, HostIO)]
struct GoodCommand {
    data: String,
}
```

### Missing MessageType Implementation
```rust
use protocol::{MessageType, MessageTypeId};

#[derive(Serialize, Deserialize, HostIO)]
struct MyCommand {
    data: String,
}

// ❌ Don't forget this!
impl MessageType for MyCommand {
    fn message_type_id() -> MessageTypeId {
        MessageTypeId::HostCommandConfigurePeripheral
    }
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
use protocol::{MessageType, MessageTypeId, IO};
use serde::{Serialize, Deserialize};

// Host command
#[derive(Serialize, Deserialize, HostIO)]
struct StartAdvertising {
    allow_multi_connect: bool,
}

impl MessageType for StartAdvertising {
    fn message_type_id() -> MessageTypeId {
        MessageTypeId::HostCommandStartAdvertisement
    }
}

// Plugin response
#[derive(Serialize, Deserialize, PluginIO)]
struct ConfigurationError {
    error_type: String,
    message: String,
}

impl MessageType for ConfigurationError {
    fn message_type_id() -> MessageTypeId {
        MessageTypeId::PluginConfigurationError
    }
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

The crate includes comprehensive tests using `trybuild` for compile-time validation:

```bash
cargo test
```

## Contributing

When contributing to this crate:

1. **Add tests** for new functionality using `trybuild`
2. **Update documentation** for any API changes
3. **Ensure backward compatibility** when possible
4. **Follow Rust API guidelines** for procedural macros

## Implementation Details

### Lifetime Parameter Detection

The macros inspect the generic parameters of the target type and detect lifetime parameters. If a lifetime is found, it uses the first lifetime parameter for the trait implementations. This ensures compatibility with both simple types and complex generic types.

### Generated Code Quality

The generated code is minimal and efficient:
- No runtime overhead
- Minimal trait bounds
- Correct lifetime handling
- Clean, readable output

### Error Messages

The macros are designed to produce helpful error messages when used incorrectly, guiding developers toward correct usage patterns.

## Dependencies

- **proc-macro2**: Core procedural macro utilities
- **quote**: Code generation and token stream manipulation  
- **syn**: Rust AST parsing with full feature support