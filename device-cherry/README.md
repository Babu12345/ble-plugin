# Device-Cherry

A USB CDC-ACM (Communication Device Class - Abstract Control Model) device implementation for ESP32 using CherryUSB. This library provides a Rust wrapper around the CherryUSB C library, enabling USB serial communication capabilities for embedded devices.

## Overview

Device-Cherry implements a USB device that appears as a virtual serial port when connected to a host computer. It's built on top of the CherryUSB library and designed specifically for ESP-IDF based ESP32 projects.

### Key Features

- **USB CDC-ACM Device**: Implements a standard USB serial device
- **State Machine Pattern**: Type-safe initialization with PREINIT/POSTINIT states
- **Async Support**: Embassy-based async operations for non-blocking I/O
- **Protocol Integration**: Built-in support for the BLE plugin protocol
- **Thread-Safe**: Uses atomic operations and synchronization primitives
- **ESP-IDF Integration**: Seamless integration with ESP-IDF build system

## Architecture

### Components

1. **CdcAcmDevice**: Main device struct implementing `PluginProcessor` for plugin communication
2. **CdcAcmDeviceHost**: Device struct implementing `HostProcessor` for host communication
3. **AlignedBuffer**: Memory-aligned buffer for USB DMA operations
4. **Error Types**: Comprehensive error handling
5. **USB Descriptors**: Device, configuration, and CDC-ACM descriptors
6. **Event Handlers**: USB event processing (connect, disconnect, suspend, resume)

### State Machine

Both device types use a type-state pattern to ensure proper initialization:

```rust
// Plugin device
CdcAcmDevice<PREINIT> -> init() -> CdcAcmDevice<POSTINIT>

// Host device
CdcAcmDeviceHost<PREINIT> -> init() -> CdcAcmDeviceHost<POSTINIT>
```

## Usage

### Plugin Device Example

```rust
use device_cherry::processors::{CdcAcmDevice, PREINIT};
use protocol::devices::plugin::PluginProcessor;
use std::time::Duration;

fn main() -> Result<()> {
    // Create new plugin device in PREINIT state
    let device = CdcAcmDevice::<PREINIT>::new();
    
    // Initialize with bus ID and register base
    let device = device.init(0, 0x60080000)?;
    
    // Start device processors with throttling
    std::thread::scope(|scope| {
        let throttle_info = (Duration::from_millis(10), 1000);
        let (plugin_sender, plugin_receiver) = device.processors(
            scope, 
            100, // channel buffer size
            throttle_info
        )?;
        
        // Use plugin_sender and plugin_receiver for communication
        Ok(())
    })
}
```

### Host Device Example

```rust
use device_cherry::processors::{CdcAcmDeviceHost, PREINIT};
use protocol::devices::host::HostProcessor;
use std::time::Duration;

fn main() -> Result<()> {
    // Create new host device in PREINIT state
    let device_host = CdcAcmDeviceHost::<PREINIT>::new();
    
    // Initialize with bus ID and register base
    let device_host = device_host.init(0, 0x60080000)?;
    
    // Start host processors with throttling
    std::thread::scope(|scope| {
        let throttle_info = (Duration::from_millis(10), 1000);
        let (host_sender, host_receiver) = device_host.processors(
            scope,
            100, // channel buffer size
            throttle_info
        )?;
        
        // Use host_sender and host_receiver for communication
        Ok(())
    })
}
```
```

### Device Communication

#### Plugin Device Communication

```rust
// Sending data from plugin to host
let data = [0x01, 0x02, 0x03, 0x00]; // Must match DEFAULT_PACKET_SIZE
plugin_sender.send(data)?;

// Receiving data from host
if let Ok(data) = plugin_receiver.try_recv() {
    println!("Plugin received: {:?}", data);
}
```

#### Host Device Communication

```rust
// Sending data from host to plugin
let data = [0x04, 0x05, 0x06, 0x00]; // Must match DEFAULT_PACKET_SIZE
host_sender.send(data)?;

// Receiving data from plugin
if let Ok(data) = host_receiver.try_recv() {
    println!("Host received: {:?}", data);
}
```

## Configuration

### ESP-IDF Configuration

The module includes `sdkconfig.defaults` for proper ESP-IDF configuration. Key settings include:

- USB peripheral support
- CherryUSB component integration
- Memory alignment requirements

### USB Descriptors

- **VID/PID**: 0xFFFF/0xFFFF (development values)
- **Max Power**: 100mA
- **Endpoints**:
  - IN: 0x81 (bulk data to host)
  - OUT: 0x02 (bulk data from host)
  - INT: 0x83 (interrupt endpoint)

## Dependencies

### Rust Crates

- `esp-idf-sys`: ESP-IDF system bindings
- `embassy-sync`: Async synchronization primitives
- `embassy-futures`: Async utilities
- `heapless`: Static memory allocation
- `ringbuffer`: Circular buffer implementation
- `protocol`: BLE plugin protocol support
- `lib_utils`: Common utilities

### C Components

- **CherryUSB**: Full-featured USB device stack implementation
  - CDC-ACM class support
  - High-speed USB 2.0 operations
  - Multiple endpoint management
- **ESP-IDF**: Espressif IoT Development Framework
  - USB peripheral drivers
  - DMA support
  - Interrupt handling

## Error Handling

The library provides comprehensive error types:

- `DeviceAlreadyInitialized`: Preventing double initialization
- `InitializationFailure`: USB initialization errors
- `BusidUndefined`: Missing bus ID configuration
- `CustomError`: Generic error with message

## Thread Safety

The implementation uses multiple thread-safety mechanisms:

- **`AtomicBool`**: For device initialization state tracking
- **`AtomicUsize`**: For active buffer index management in double buffering
- **`Signal`**: Embassy-based async event notification for USB data
- **`LazyLock`**: Thread-safe lazy initialization of USB descriptors
- **Critical sections**: For interrupt-safe operations
- **Channel-based communication**: Thread-safe message passing between USB and application layers
- **Atomic ordering**: Uses `Acquire`/`Release` ordering for proper memory synchronization

## Performance Considerations

### High-Speed Optimizations

- **Double Buffering**: Uses two alternating buffers (`READ_BUFFER_A` and `READ_BUFFER_B`) for continuous USB data flow
- **DMA-Aligned Buffers**: All buffers are properly aligned for efficient DMA transfers
- **Aggressive Retry Logic**: Up to 20 retries with microsecond delays for USB endpoint operations
- **Throttling Support**: Built-in rate limiting to prevent overwhelming the communication channels
- **Zero-Copy Operations**: Minimizes memory copies in the data path
- **Atomic Buffer Switching**: Race-free buffer management using atomic operations

### Throughput Features

- **Immediate USB Read Restart**: Minimizes gaps between USB transfers for continuous communication
- **Non-blocking Channel Operations**: Uses `try_send` to avoid blocking on full channels
- **Burst Handling**: Optimized for high-speed data bursts with sliding window drop rate tracking
- **Default packet size**: Fixed at `DEFAULT_PACKET_SIZE` (typically 64 bytes) for predictable performance

## Building

This is an ESP-IDF component that must be built as part of an ESP32 project:

```bash
# From your ESP-IDF project root
idf.py build
```

## Testing

Run the standard Rust tests:

```bash
cargo test
```

For hardware testing, flash to an ESP32 device and monitor:

```bash
idf.py flash monitor
```

## References

The implementation is based on several CherryUSB examples:
- [CDC-MSC Example](https://github.com/zleihao/CherryUSB-CDC-MSC)
- [ESP32 Examples](https://github.com/CherryUSB/cherryusb_esp32)

## License

Copyright © 2025 Wanyeki Technologies LLC. All rights reserved.

This project is private and proprietary.