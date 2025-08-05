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

1. **CdcAcmDevice**: Main device struct with state machine pattern
2. **AlignedBuffer**: Memory-aligned buffer for USB DMA operations
3. **Error Types**: Comprehensive error handling
4. **USB Descriptors**: Device, configuration, and CDC-ACM descriptors
5. **Event Handlers**: USB event processing (connect, disconnect, suspend, resume)

### State Machine

The device uses a type-state pattern to ensure proper initialization:

```rust
CdcAcmDevice<PREINIT> -> init() -> CdcAcmDevice<POSTINIT>
```

## Usage

### Basic Example

```rust
use device_cherry::{CdcAcmDevice, Result};
use std::time::Duration;

fn main() -> Result<()> {
    // Create new device in PREINIT state
    let device = CdcAcmDevice::new();
    
    // Initialize with bus ID and register base
    let device = device.init(0, 0x60080000)?;
    
    // Create channels for communication
    let (tx_send, tx_recv) = std::sync::mpsc::sync_channel(10);
    let (rx_send, rx_recv) = std::sync::mpsc::channel();
    
    // Start device processors
    std::thread::scope(|scope| {
        device.processors(scope, tx_recv, rx_send)?;
        Ok(())
    })
}
```

### Sending Data

```rust
// Send data through the tx channel
tx_send.send(vec![0x01, 0x02, 0x03])?;
```

### Receiving Data

```rust
// Receive data from the rx channel
if let Ok(data) = rx_recv.try_recv() {
    println!("Received: {:?}", data);
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

- CherryUSB: USB device stack implementation
- ESP-IDF: Espressif IoT Development Framework

## Error Handling

The library provides comprehensive error types:

- `DeviceAlreadyInitialized`: Preventing double initialization
- `InitializationFailure`: USB initialization errors
- `BusidUndefined`: Missing bus ID configuration
- `CustomError`: Generic error with message

## Thread Safety

The implementation uses:
- `AtomicBool` for initialization state
- `Signal` for async event notification
- `LazyLock` for static initialization
- Critical sections for interrupt safety

## Performance Considerations

- Uses DMA-aligned buffers for efficient transfers
- Default packet size: Configured via `DEFAULT_PACKET_SIZE`
- Non-blocking async operations minimize CPU usage
- Zero-copy data paths where possible

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

This module follows the same license as the parent BLE plugin project.