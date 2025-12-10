# Plugin Host-or-Device Standard

A Rust application that creates a flexible ESP32-based BLE-USB bridge capable of operating in either USB host or USB device mode. The mode is determined at runtime by GPIO pin state, allowing the same firmware to work in different deployment scenarios.

## Overview

This application combines the BLE plugin state machine with dynamic USB mode selection to create a versatile IoT communication bridge that can adapt to different hardware configurations.

## Features

- **Dynamic USB Mode Selection**: GPIO pin determines USB host vs device mode at startup
- **BLE-USB Bridge**: Full bidirectional communication between BLE peripherals and USB hosts
- **Runtime Adaptability**: Single firmware supports multiple deployment scenarios
- **LED Status Indication**: Visual feedback via GPIO21 indicator
- **Persistent Configuration**: NVS-backed settings survive power cycles

## Architecture

```
GPIO9 State → USB Mode Selection → BLE Plugin State Machine
     ↓              ↓                        ↓
   Low/High   Host/Device Mode      USB-BLE Bridge
```

## GPIO Configuration

- **GPIO9 (Input)**: USB mode selector with pull-down
  - `LOW` → USB Host Mode (connect to USB devices)
  - `HIGH` → USB Device Mode (connect to USB hosts)
- **GPIO21 (Output)**: Status indicator LED

## Usage

The application automatically:
1. Reads GPIO9 pin state at startup
2. Initializes appropriate USB stack (host or device)
3. Starts BLE plugin state machine
4. Bridges communication between USB and BLE domains

## Dependencies

- `device-cherry`: USB device mode implementation
- `host-cherry`: USB host mode implementation
- `plugin_state_machine_std`: Core BLE bridge logic
- `plugin-nvs`: Persistent configuration storage
- `protocol`: Shared communication protocol

## Build & Flash

```bash
cargo build --release
cargo run
```

Requires ESP32-S3 with USB OTG support.

## License

Copyright © 2025 Wanyeki Technologies LLC. All rights reserved.

This project is private and proprietary.