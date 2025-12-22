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
- **Robust Reconnection**: Automatic recovery from rapid device restarts and stuck states
- **Self-Healing**: USB stack re-initialization on connection failures

## Architecture

### Components

1. **CdcAcmHost**: Main host struct implementing `HostProcessor` for standard host communication
2. **CdcAcmHostDevice**: Host struct implementing `PluginProcessor` for plugin communication
3. **USB Host Initialization**: Sets up the USB host controller with type-safe state management
4. **CDC-ACM Class Driver**: Manages CDC device communication
5. **Thread-Safe Wrapper**: Ensures safe access from multiple threads
6. **Channel Processors**: Separate threads for sending and receiving data
7. **Protocol Adapters**: Support for both host and plugin communication protocols

### State Machine

Both host types use a type-state pattern to ensure proper initialization:

```rust
// Host processor
CdcAcmHost<PREINIT> -> init() -> CdcAcmHost<POSTINIT>

// Plugin processor
CdcAcmHostDevice<PREINIT> -> init() -> CdcAcmHostDevice<POSTINIT>
```

### Threading Model

The library uses a multi-threaded architecture:
- Main thread: Initialization and setup
- Send thread: Processes outgoing USB data
- Receive thread: Handles incoming USB data
- USB event thread: Manages device events (handled by CherryUSB)

## Usage

### Host Mode Example

```rust
use host_cherry::{CdcAcmHost, PREINIT};
use protocol::devices::host::HostProcessor;
use std::time::Duration;

fn main() -> Result<(), ()> {
    // Create new host in PREINIT state
    let host = CdcAcmHost::<PREINIT>::new();
    
    // Initialize with bus ID and register base
    let host = host.init(0, 0x60080000)?;
    
    // Start host processors
    std::thread::scope(|scope| {
        let throttle_info = (Duration::from_millis(10), 1000);
        let (host_sender, host_receiver) = host.processors(
            scope,
            100, // channel buffer size
            throttle_info
        )?;
        
        // Send data to connected USB device
        let data = [0x01, 0x02, 0x03, 0x00]; // Must match DEFAULT_PACKET_SIZE
        host_sender.send(data)?;
        
        // Receive data from connected USB device
        if let Ok(data) = host_receiver.try_recv() {
            println!("Host received: {:?}", data);
        }
        
        Ok(())
    })
}
```

### Plugin Mode Example

```rust
use host_cherry::{CdcAcmHostDevice, PREINIT};
use protocol::devices::plugin::PluginProcessor;
use std::time::Duration;

fn main() -> Result<(), ()> {
    // Create new host device for plugin protocol
    let host_device = CdcAcmHostDevice::<PREINIT>::new();
    
    // Initialize with bus ID and register base
    let host_device = host_device.init(0, 0x60080000)?;
    
    // Start plugin processors
    std::thread::scope(|scope| {
        let throttle_info = (Duration::from_millis(10), 1000);
        let (plugin_sender, plugin_receiver) = host_device.processors(
            scope,
            100, // channel buffer size
            throttle_info
        )?;
        
        // Send data as plugin to host
        let data = [0x04, 0x05, 0x06, 0x00]; // Must match DEFAULT_PACKET_SIZE
        plugin_sender.send(data)?;
        
        // Receive data from host
        if let Ok(data) = plugin_receiver.try_recv() {
            println!("Plugin received: {:?}", data);
        }
        
        Ok(())
    })
}
```

## API Reference

### Structures

#### `CdcAcmHost<STATE>`

Main USB host implementation for standard host communication.

**Methods:**
- `new()` - Creates a new host in PREINIT state
- `init(busid, reg_base)` - Initializes the USB host controller
- `sleep(duration)` - Utility method for delays
- `processors(scope, buffer_size, throttle_info)` - Creates communication channels

#### `CdcAcmHostDevice<STATE>`

USB host implementation for plugin protocol communication.

**Methods:**
- `new()` - Creates a new host device in PREINIT state
- `init(busid, reg_base)` - Initializes the USB host controller
- `sleep(duration)` - Utility method for delays
- `processors(scope, buffer_size, throttle_info)` - Creates plugin communication channels

#### State Types

- `PREINIT` - Uninitialized state, allows calling `init()`
- `POSTINIT` - Initialized state, allows calling `processors()`

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

1. **AtomicBool**: Initialization state tracking to prevent double initialization
2. **Type-State Pattern**: Compile-time guarantees for proper initialization order
3. **Channel Isolation**: Send and receive operations use separate channels
4. **Atomic Operations**: Device state changes are handled atomically
5. **Safe Wrappers**: Raw pointers are wrapped in `Send`/`Sync` types
6. **RwLock Protection**: CDC device handle is protected by a read-write lock (in processors module)

## Error Handling

The library provides simple but effective error handling:

### Initialization Errors
- **Double Initialization**: Prevented by `AtomicBool` state tracking
- **USB Controller Failure**: Returns `Err(())` if host controller initialization fails
- **Type Safety**: State machine prevents calling methods in wrong initialization state

### Runtime Errors
- **Device Not Connected**: Operations wait for device connection (handled in processors)
- **Buffer Full**: Channel operations handle backpressure automatically
- **Transfer Errors**: Logged by the underlying processor threads
- **Disconnection**: Gracefully handled by CDC driver callbacks

### Error Types
- Simple `Result<T, ()>` for most operations
- Detailed error logging through the `log` crate
- Automatic recovery for transient USB errors

## Reliability and Reconnection Handling

The library includes robust handling for device disconnection and reconnection scenarios, particularly important for rapid device restarts.

### Automatic Reconnection

When a USB device disconnects and reconnects, the library automatically:

1. **Detects Disconnection**: The `usbh_cdc_acm_stop` callback is invoked by the USB stack
2. **Clears Device State**: CDC device handle is set to `None` and ready signal is reset
3. **Waits for Reconnection**: Both send and receive threads enter a wait loop
4. **Detects Connection**: The `usbh_cdc_acm_run` callback signals device availability
5. **Reconfigures Device**: CDC line state is configured (DTR=true, RTS=false)
6. **Resumes Operations**: Normal data transfer operations continue

### Self-Healing USB Stack

To handle cases where the USB hardware controller gets stuck (e.g., during rapid device restarts), the library implements automatic USB stack re-initialization:

#### Stuck State Detection
- Monitors reconnection attempts in the receive thread
- Triggers after `REINIT_THRESHOLD` (default: 10) failed attempts
- Typically occurs after ~1 second of waiting (10 attempts × 100ms timeout)

#### Re-initialization Process
```rust
const REINIT_THRESHOLD: u32 = 10;

// When threshold is reached:
1. Call usbh_deinitialize() to tear down the USB stack
2. Wait 100ms for hardware to settle
3. Call usbh_initialize() with original parameters
4. Reset reconnection attempt counter
5. Wait 200ms for stack to stabilize
```

#### Benefits
- **Clears Hardware State**: Resets any stuck USB controller state
- **Recovers from Missed Interrupts**: Handles cases where hardware doesn't generate connection events
- **Automatic Recovery**: No manual intervention required
- **Preserves Configuration**: Uses stored initialization parameters

### Rapid Restart Handling

The implementation specifically handles the challenging case of rapid device restarts:

**Problem**: When a device restarts very quickly (< 1 second), the USB hardware controller may:
- Miss the reconnection interrupt
- Have stale state from the previous connection
- Fail to re-enumerate the device

**Solution**: Multi-layered approach:
1. **Short Enumeration Wait**: 100ms timeout for checking USB enumeration status
2. **Error-Based Detection**: USB transfer errors trigger immediate reconnection checks
3. **Active Wait Loop**: Threads continuously check device status every 100ms
4. **USB Stack Reset**: After 10 failed attempts (~1 second), completely reset the USB stack

### Thread Synchronization

Reconnection handling uses careful synchronization between threads:

#### CdcReadySignal
```rust
struct CdcReadySignal {
    ready: Mutex<bool>,          // Device availability flag
    condvar: Condvar,            // Thread notification
    configured: AtomicBool,      // Configuration guard
}
```

- **Signal on Connect**: `usbh_cdc_acm_run` sets ready=true and notifies all waiting threads
- **Reset on Disconnect**: `usbh_cdc_acm_stop` sets ready=false
- **Atomic Configuration**: Only one thread configures the device using compare-exchange
- **Wait with Timeout**: Threads wait up to 100ms before checking again

### Diagnostic Logging

The library provides detailed logging for debugging connection issues:

```
INFO  usbh_cdc_acm_run callback invoked (ptr: 0x3fca3c10)
INFO  CDC ACM device enumerated and ready
INFO  CDC device signal received, verifying...
INFO  CDC device reconnected and reconfigured
```

On disconnect:
```
WARN  usbh_cdc_acm_stop callback invoked (ptr: 0x3fca3c10)
INFO  CDC ACM device disconnected and signal reset
WARN  CDC device not connected, waiting for device...
```

During stuck state recovery:
```
INFO  Still waiting for CDC device... (10 attempts)
WARN  Device not reconnecting after 10 attempts, re-initializing USB stack
WARN  Re-initializing USB stack to recover from stuck state...
INFO  USB stack re-initialized successfully
```

### Best Practices

For optimal reliability:

1. **Allow Stabilization Time**: After device connect, the code waits 100ms before operations
2. **Monitor Logs**: Watch for re-initialization events which may indicate hardware issues
3. **Consider Timing**: Devices that restart very rapidly (< 100ms) may trigger re-initialization
4. **Test Reconnection**: Verify your application handles the brief pause during re-initialization

## Performance Considerations

### USB Transfer Optimizations

- **Polling Interval**: 10ms sleep when no device is connected
- **Timeout**: Infinite timeout for USB transfers (u32::MAX)
- **Buffer Size**: Configurable channel buffer size for backpressure management
- **Zero-Copy**: Direct buffer transfers to minimize memory overhead
- **Fixed Packet Size**: Uses `DEFAULT_PACKET_SIZE` for predictable performance

### Threading Performance

- **Separate Send/Receive Threads**: Independent processing of USB I/O operations
- **Non-blocking Operations**: Uses `try_recv()` and `try_send()` to avoid blocking
- **Channel-based Communication**: Efficient inter-thread communication
- **Throttling Support**: Built-in throttling parameters (though not actively used in host mode)

## Dependencies

### Rust Crates

- **`esp-idf-sys`**: ESP-IDF system bindings with CherryUSB host support
- **`heapless`**: Fixed-size collections for internal buffers
- **`log`**: Logging framework for debugging and monitoring
- **`protocol`**: Communication protocol definitions
  - Uses `alloc::Vec` and `alloc::String` for Protocol Buffer compatibility
  - Defines `DEFAULT_PACKET_SIZE` and processor traits
  - Provides `HostSender`/`HostReceiver` and `PluginSender`/`PluginReceiver` types

### C Components

- **CherryUSB**: Full-featured USB host stack implementation
  - CDC-ACM host class driver
  - Device enumeration and management
  - Bulk transfer support for high-speed data
  - Event handling for connect/disconnect
- **ESP-IDF**: Espressif IoT Development Framework
  - USB OTG peripheral drivers
  - DMA support for efficient transfers
  - Interrupt handling and power management

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

### Hardware Limitations
- Only supports one CDC device at a time
- Requires ESP32 variants with USB OTG support (S2, S3, C3, C6)
- Fixed register base address (0x60080000 for ESP32-S3)

### Software Limitations
- Fixed packet size determined by `DEFAULT_PACKET_SIZE` at compile time
- No dynamic device configuration
- Single bus ID support (typically bus 0)
- Simple error handling with unit type `()` errors

### Protocol Limitations
- Separate types for host vs plugin protocols (not interchangeable)
- No runtime protocol switching
- Fixed channel buffer sizes (set at initialization)

## References

The implementation is based on CherryUSB examples:
- [CherryUSB CDC Host](https://github.com/cherry-embedded/CherryUSB)
- [ESP32 USB Host Examples](https://github.com/CherryUSB/cherryusb_esp32)
- [Zephyr SDK Integration](https://github.com/hpmicro/zephyr_sdk_glue)

## License

Copyright © 2025 Wanyeki Technologies LLC. All rights reserved.

This project is private and proprietary.