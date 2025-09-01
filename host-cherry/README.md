# Host-Cherry

A USB host implementation for ESP32 using CherryUSB, providing CDC-ACM (Communication Device Class - Abstract Control Model) host capabilities. This library enables ESP32 devices to act as USB hosts, communicating with USB CDC devices like serial adapters, modems, and other USB serial devices.

## Overview

Host-Cherry implements a USB host controller that can enumerate and communicate with USB CDC-ACM devices. It provides a Rust wrapper around the CherryUSB C library, offering safe abstractions for USB host operations on ESP32 platforms.

### Key Features

- **USB Host Mode**: Enables ESP32 to act as a USB host controller
- **CDC-ACM Support**: Communicates with USB serial devices
- **Thread-Safe Operations**: Safe concurrent access to USB resources
- **Channel-Based API**: Simple send/receive interface using Rust channels
- **Protocol Integration**: Compatible with both host and plugin protocol variants
- **Automatic Device Detection**: Handles USB device connection/disconnection events

## Architecture

### Components

1. **USB Host Initialization**: Sets up the USB host controller
2. **CDC-ACM Class Driver**: Manages CDC device communication
3. **Thread-Safe Wrapper**: Ensures safe access from multiple threads
4. **Channel Processors**: Separate threads for sending and receiving data
5. **Protocol Adapters**: Support for both host and plugin communication protocols

### Threading Model

The library uses a multi-threaded architecture:
- Main thread: Initialization and setup
- Send thread: Processes outgoing USB data
- Receive thread: Handles incoming USB data
- USB event thread: Manages device events (handled by CherryUSB)

## Usage

### Basic Host Mode

```rust
use host_cherry::cherry_usb_host;
use std::thread;

fn main() {
    thread::scope(|scope| {
        // Initialize USB host with channel buffer size of 10
        let (sender, receiver) = unsafe {
            cherry_usb_host(scope, 10)
        };
        
        // Send data to USB device
        sender.send(b"Hello USB Device").unwrap();
        
        // Receive data from USB device
        if let Ok(data) = receiver.try_recv() {
            println!("Received: {:?}", data);
        }
    });
}
```

### Plugin Mode Support

```rust
use host_cherry::cherry_usb_host_for_plugin;
use std::thread;

fn main() {
    thread::scope(|scope| {
        // Initialize for plugin protocol
        let (sender, receiver) = unsafe {
            cherry_usb_host_for_plugin(scope, 10)
        };
        
        // Use plugin protocol for communication
        // sender and receiver are typed for plugin messages
    });
}
```

## API Reference

### Functions

#### `cherry_usb_host`
```rust
pub unsafe fn cherry_usb_host<'a, 'b>(
    scope: &'a Scope<'a, 'b>,
    channel_buffer_size: usize,
) -> (HostSender<DEFAULT_PACKET_SIZE>, HostReceiver<DEFAULT_PACKET_SIZE>)
```
Initializes USB host for standard host protocol communication.

#### `cherry_usb_host_for_plugin`
```rust
pub unsafe fn cherry_usb_host_for_plugin<'a, 'b>(
    scope: &'a Scope<'a, 'b>,
    channel_buffer_size: usize,
) -> (PluginSender<DEFAULT_PACKET_SIZE>, PluginReceiver<DEFAULT_PACKET_SIZE>)
```
Initializes USB host for plugin protocol communication.

### Callbacks

The library exports C-compatible callbacks that are called by CherryUSB:

- `usbh_cdc_acm_run`: Called when a CDC device is connected and ready
- `usbh_cdc_acm_stop`: Called when a CDC device is disconnected

## Configuration

### ESP-IDF Settings

The module includes `sdkconfig.defaults` with required ESP-IDF configurations:
- USB OTG peripheral support
- Host mode configuration
- DMA buffer allocation
- Interrupt priorities

### USB Parameters

- **Base Address**: `ESP_USBH_BASE` (0x60080000 for ESP32-S3)
- **Bus ID**: 0 (default USB bus)
- **Packet Size**: Configured via `DEFAULT_PACKET_SIZE` from protocol crate

## Thread Safety

The implementation ensures thread safety through:

1. **RwLock Protection**: CDC device handle is protected by a read-write lock
2. **Channel Isolation**: Send and receive operations use separate channels
3. **Atomic Operations**: Device state changes are handled atomically
4. **Safe Wrappers**: Raw pointers are wrapped in `Send`/`Sync` types

## Error Handling

The library handles various error conditions:

- **Device Not Connected**: Operations wait for device connection
- **Buffer Full**: Warns when receive buffer is full
- **Transfer Errors**: Logs USB transfer failures
- **Disconnection**: Gracefully handles device removal

## Performance Considerations

- **Polling Interval**: 10ms sleep when no device is connected
- **Timeout**: Infinite timeout for USB transfers (u32::MAX)
- **Buffer Size**: Configurable channel buffer size
- **Zero-Copy**: Direct buffer transfers to minimize overhead

## Dependencies

### Rust Crates

- `esp-idf-sys`: ESP-IDF system bindings
- `heapless`: Fixed-size collections (internal use)
- `log`: Logging framework
- `protocol`: Communication protocol definitions (uses `alloc` for Protocol Buffer types)

### C Components

- CherryUSB: USB host stack implementation
- ESP-IDF: USB OTG driver and HAL

## Building

This is an ESP-IDF component that must be built as part of an ESP32 project:

```bash
# From your ESP-IDF project root
idf.py build
```

## Debugging

Enable debug logging to see USB operations:

```rust
env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
```

Common debug points:
- Device enumeration in `usbh_cdc_acm_run`
- Data transfers in send/receive threads
- Error conditions in transfer operations

## Limitations

- Only supports one CDC device at a time
- Requires ESP32 variants with USB OTG support (S2, S3, C3)
- Fixed packet size determined at compile time
- No dynamic device configuration

## References

The implementation is based on CherryUSB examples:
- [CherryUSB CDC Host](https://github.com/cherry-embedded/CherryUSB)
- [ESP32 USB Host Examples](https://github.com/CherryUSB/cherryusb_esp32)
- [Zephyr SDK Integration](https://github.com/hpmicro/zephyr_sdk_glue)

## License

This module follows the same license as the parent BLE plugin project.