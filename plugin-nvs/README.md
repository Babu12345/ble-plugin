# plugin-nvs

A Rust crate providing non-volatile storage (NVS) functionality for ESP32-based BLE plugin systems. This crate offers a type-safe interface for managing persistent configuration data using the ESP-IDF NVS API.

## Features

- Type-safe NVS namespace management
- Structured key-value storage for configuration data
- Error handling for NVS operations
- Support for peripheral configuration storage
- Built on top of `esp-idf-svc` for reliable ESP32 integration

## Usage

### Basic Example

```rust
use plugin_nvs::{namespace, namespaces::ConfigNamespace};
use esp_idf_svc::nvs::EspNvsPartition;

// Create an NVS partition handle
let nvs_partition = EspNvsPartition::take()?;

// Get a namespace handle
let mut config_namespace = namespace::<ConfigNamespace>(nvs_partition)?;

// Access peripheral configuration
let mut peripheral_config = config_namespace.peripheral_config_key();

// Write configuration data
let config_data = b"example_config";
peripheral_config.write(config_data)?;

// Read configuration data
let mut buffer = [0u8; 256];
if let Some(data) = peripheral_config.read(&mut buffer)? {
    // Process the configuration data
    println!("Config: {:?}", data);
}
```

### Namespace Architecture

The crate uses a trait-based approach for organizing NVS data:

- **`NvsNamespaceTrait`**: Defines NVS namespaces (logical groupings of related data)
- **`NvsKeyTrait`**: Defines individual keys within a namespace
- **`ConfigNamespace`**: Pre-defined namespace for configuration data

### Error Handling

The crate provides comprehensive error types through `PluginNvcError`:

- `NamespaceAcquisitionError`: Failed to acquire namespace handle
- `NamespaceNotFound`: Specified namespace doesn't exist
- `NvsReadError`: Failed to read from NVS
- `NvsWriteError`: Failed to write to NVS
- `NvsEraseError`: Failed to erase NVS data
- `NvsPartitionFull`: NVS partition has no free space
- `NvsGenericError`: Other NVS-related errors

## Dependencies

- `esp-idf-sys`: ESP-IDF system bindings
- `esp-idf-svc`: High-level ESP-IDF service wrappers
- `lib_utils`: Internal utility library
- `log`: Logging facade

## Requirements

- ESP32 or compatible microcontroller
- ESP-IDF development environment
- Rust toolchain configured for ESP32 development

## License

Copyright © 2025 Wanyeki Technologies LLC. All rights reserved.

This project is private and proprietary.