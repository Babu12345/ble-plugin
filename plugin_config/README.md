# Plugin Config

Hardware-agnostic BLE profile configuration library for the BLE Plugin system.

## Overview

`plugin_config` provides a trait-based abstraction layer for configuring BLE (Bluetooth Low Energy) peripherals and managing standard Bluetooth SIG profiles. This crate is designed to work with any BLE stack implementation (ESP32-Nimble, BlueZ, nRF, etc.) through a unified interface.

## Features

- **Hardware Agnostic**: Profile definitions work with any BLE stack implementation
- **Standard BLE Profiles**: Pre-configured implementations of 14 Bluetooth SIG standard profiles
- **Custom Profiles**: Support for user-defined services and characteristics
- **Type-Safe**: Rust's type system ensures correct profile configuration
- **Comprehensive Testing**: Each profile includes unit tests validating structure and default values

## Supported BLE Profiles

### Health & Fitness
- **Heart Rate Monitor** (0x180D) - Heart rate measurement and body sensor location
- **Blood Pressure** (0x1810) - Blood pressure monitoring with measurement and feature characteristics
- **Glucose Monitoring** (0x1808) - Continuous glucose monitoring with context and record access
- **Weight Scale** (0x181D) - Weight measurements with BMI support and multi-user capability
- **Health Thermometer** (0x1809) - Temperature measurement with type and interval characteristics
- **Cycling Speed and Cadence** (0x1816) - Bike computer data with sensor location

### IoT & Sensors
- **Environmental Sensing** (0x181A) - Temperature, humidity, and pressure sensors
- **Battery Service** (0x180F) - Battery level monitoring
- **Proximity Profile** (0x1802/0x1803/0x1804) - Item finders and asset tracking

### Device Information & Time
- **Device Information** (0x180A) - Manufacturer name, model number, firmware revision
- **Current Time Service** (0x1805) - Time synchronization for watches and devices

### User Interface
- **HID over GATT** (0x1812) - Keyboards, mice, game controllers, remote controls
- **Phone Alert Status** (0x180E) - Smartwatch notifications and wearable alerts

### Custom
- **Custom Profile** - User-defined services and characteristics configured via commands

## Architecture

### Core Trait: `PluginConfig`

The `PluginConfig` trait defines the interface that BLE stack implementations must provide:

```rust
pub trait PluginConfig<ERROR: Debug> {
    // Peripheral configuration
    fn handle_configure_peripheral(&mut self, cmd: HostCommandConfigurePeripheral)
        -> Result<(), ERROR>;

    // Service and characteristic configuration
    fn handle_configure_service(&mut self, cmd: HostCommandConfigureService)
        -> Result<(), ERROR>;
    fn handle_configure_characteristic(&mut self, cmd: HostCommandConfigureCharacteristic)
        -> Result<(), ERROR>;

    // Profile configuration (with default implementation)
    fn handle_configure_profile(&mut self, cmd: HostCommandConfigureProfile)
        -> Result<(), ERROR> {
        // Default implementation handles all standard profiles
    }

    // Implementation-specific hooks
    fn restart_server_with_profile(&mut self, save_on_disconnect: bool)
        -> Result<(), ERROR>;
    fn handle_unknown_profile(&mut self) -> Result<(), ERROR>;
}
```

### Profile Definition Structure

Profiles are defined using a hierarchical structure:

```rust
ProfileDefinition {
    services: Vec<ServiceDefinition>
}

ServiceDefinition {
    uuid: u16,
    characteristics: Vec<CharacteristicDefinition>
}

CharacteristicDefinition {
    uuid: u16,
    properties: Vec<i32>,
    default_value: Option<Vec<u8>>
}
```

## Usage

### Implementing for a BLE Stack

```rust
use plugin_config::{PluginConfig, BlinkState, HardwareAccessories};

struct MyBleStack {
    // Your BLE stack state
}

impl PluginConfig<MyError> for MyBleStack {
    fn handle_configure_peripheral(&mut self, cmd: HostCommandConfigurePeripheral)
        -> Result<(), MyError> {
        // Configure your BLE peripheral
        Ok(())
    }

    fn handle_configure_service(&mut self, cmd: HostCommandConfigureService)
        -> Result<(), MyError> {
        // Add service to your BLE stack
        Ok(())
    }

    fn handle_configure_characteristic(&mut self, cmd: HostCommandConfigureCharacteristic)
        -> Result<(), MyError> {
        // Add characteristic to your BLE stack
        Ok(())
    }

    fn restart_server_with_profile(&mut self, save_on_disconnect: bool)
        -> Result<(), MyError> {
        // Restart your BLE server to apply configuration
        Ok(())
    }

    fn handle_unknown_profile(&mut self) -> Result<(), MyError> {
        Err(MyError::UnknownProfile)
    }

    fn handle_clear_all_services(&mut self) -> Result<(), MyError> {
        // Clear all services from your BLE stack
        // Reset internal metadata tracking
        Ok(())
    }
}
```

The `handle_configure_profile` method has a default implementation that automatically handles all standard profiles. You only need to implement the low-level methods and the hooks.

### Using a Standard Profile

```rust
use protocol::protocol::{HostCommandConfigureProfile, BleProfile};

// Configure the device as a Heart Rate Monitor
let cmd = HostCommandConfigureProfile {
    profile: BleProfile::HeartRateMonitor as i32,
    save_on_disconnect: false,
};

ble_stack.handle_configure_profile(cmd)?;
```

### Creating a Custom Profile

```rust
// Configure services and characteristics individually
ble_stack.handle_configure_service(HostCommandConfigureService {
    uuid: 0x1234,
})?;

ble_stack.handle_configure_characteristic(HostCommandConfigureCharacteristic {
    uuid: 0x5678,
    service_uuid: 0x1234,
    properties: vec![PROPERTY_READ, PROPERTY_NOTIFY],
})?;

// Apply the custom profile
ble_stack.handle_configure_profile(HostCommandConfigureProfile {
    profile: BleProfile::Custom as i32,
    save_on_disconnect: false,
})?;
```

### Clearing Services

Clear all configured services and metadata to start fresh:

```rust
// Clear all existing services and metadata
ble_stack.handle_clear_all_services()?;

// Now configure a new profile from scratch
ble_stack.handle_configure_profile(HostCommandConfigureProfile {
    profile: BleProfile::HeartRateMonitor,
    save_on_disconnect: false,
})?;
```

This is useful when:
- Switching between different profiles dynamically
- Recovering from configuration errors
- Implementing profile hot-swapping without device restart
- Testing different configurations during development

## Profile Definitions Module

All standard profile definitions are located in the `profiles` module:

```rust
use plugin_config::profiles::heart_rate::heart_rate_profile;
use plugin_config::profiles::battery_service::battery_service_profile;
use plugin_config::profiles::hid_over_gatt::hid_over_gatt_profile;

let hr_profile = heart_rate_profile();
// Returns a ProfileDefinition with Heart Rate Service configured
```

Each profile module includes:
- Service and characteristic UUID constants
- Profile-specific enums (e.g., `BodySensorLocation`, `ProtocolMode`)
- Profile factory function (e.g., `heart_rate_profile()`)
- Comprehensive unit tests

## Hardware Abstraction

The `plugin_config` crate is completely hardware-agnostic:

- **No platform dependencies**: Works on embedded (no_std), desktop, and mobile
- **Stack-independent**: Compatible with ESP32-Nimble, BlueZ, nRF SoftDevice, etc.
- **Protocol-driven**: Uses protocol buffers for cross-language compatibility
- **Trait-based**: BLE stack implementations provide concrete behavior

This design allows:
1. Profile definitions to be shared across different hardware platforms
2. Testing and development on desktop systems
3. Easy integration with existing BLE stacks
4. Future-proofing against BLE stack changes

## Testing

Run the test suite:

```bash
cargo test
```

Each profile includes tests for:
- Profile structure validation
- Characteristic properties and UUIDs
- Default value correctness
- Enum value mappings

## Integration Example: ESP32-Nimble

See the `esp_nimble_plugin_config` crate for a complete implementation using the ESP32-Nimble BLE stack.

## License

Copyright © 2025 Wanyeki Technologies LLC. All rights reserved.

This source code is proprietary and confidential. Unauthorized copying, modification, distribution, or use of this software is strictly prohibited.

## Documentation

For detailed API documentation:

```bash
cargo doc --open
```

## Related Crates

- `protocol` - Protocol buffer definitions for BLE commands and messages
- `esp_nimble_plugin_config` - ESP32-Nimble BLE stack implementation
