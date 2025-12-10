# Device-Embassy

A `no_std` USB CDC-ACM (Communication Device Class - Abstract Control Model) device implementation for ESP32 using Embassy async runtime. This library provides a fully async USB serial communication interface for embedded ESP32 systems running with Embassy executor.

## Overview

Device-Embassy implements a USB device that appears as a virtual serial port when connected to a host computer. It's built on top of the Embassy USB stack and designed specifically for bare-metal ESP32 projects using the Embassy async runtime.

### Key Features

- **Fully Async**: Built with Embassy's async runtime for efficient task scheduling
- **No-STD Compatible**: Designed for embedded bare-metal environments
- **CDC-ACM Support**: Standard USB serial device implementation
- **Embassy Integration**: Uses Embassy USB, sync, and time primitives
- **Protocol Integration**: Implements `AsyncHostProcessor` for seamless protocol support
- **Configurable Buffers**: Compile-time buffer size configuration
- **Connection Management**: Automatic handling of USB connect/disconnect events

## Architecture

### Components

1. **CdcAcmDeviceHost**: Main device struct implementing `AsyncHostProcessor`
2. **Embassy Integration**: Full integration with Embassy executor and async runtime
3. **USB Management**: Automatic USB device enumeration and connection handling
4. **Error Handling**: Comprehensive error types and recovery mechanisms
5. **Channel-Based Communication**: Embassy channels for async message passing

### Async Architecture

The library uses Embassy's async architecture with three concurrent tasks:
- **USB Runner**: Handles USB device events and enumeration
- **Write Task**: Processes outgoing data to USB host
- **Read Task**: Handles incoming data from USB host

## Usage

### Basic Example

```rust
#![no_std]
#![no_main]

use device_embassy::processors::CdcAcmDeviceHost;
use embassy_executor::Spawner;
use embassy_sync::channel::Channel;
use embassy_usb::class::cdc_acm::State;
use esp_hal::{clock::CpuClock, otg_fs::Usb, timer::systimer::SystemTimer};
use protocol::devices::host::AsyncHostProcessor;
use protocol::DEFAULT_PACKET_SIZE;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // Initialize ESP32 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    // Initialize heap allocator
    esp_alloc::heap_allocator!(size: 72 * 1024);

    // Initialize Embassy timer
    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    // Initialize USB peripheral
    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);

    // Allocate required buffers (on stack)
    let mut ep_out_buffer = [0; 1024];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut state = State::new();

    // Create device with const generics: <CHANNEL_SIZE, BUFFER_SIZE>
    let device_host: CdcAcmDeviceHost<'_, 20, DEFAULT_PACKET_SIZE> = CdcAcmDeviceHost::new(
        usb,
        &mut ep_out_buffer,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut control_buf,
        &mut state,
    );

    // Create communication channels
    let to = Channel::new();
    let from = Channel::new();

    // Start processors - returns a future that handles USB operations
    let (processor_fn, _sender, _receiver) = device_host
        .processors(
            (to.sender(), to.receiver()),
            (from.sender(), from.receiver()),
        )
        .unwrap();

    // Run the processor (this handles all USB operations internally)
    processor_fn.await;
}
```

### Advanced Usage with Error Handling

```rust
use embassy_futures::join::join;

async fn usb_communication_task(
    host_sender: AsyncHostSender<'_, CriticalSectionRawMutex, BUFFER_SIZE, CHANNEL_SIZE>,
    host_receiver: AsyncHostReceiver<'_, CriticalSectionRawMutex, BUFFER_SIZE, CHANNEL_SIZE>,
) {
    let send_task = async {
        let mut counter = 0u8;
        loop {
            let mut data = [0u8; BUFFER_SIZE];
            data[0] = counter;
            
            host_sender.send(data).await;
            counter = counter.wrapping_add(1);
            
            embassy_time::Timer::after(embassy_time::Duration::from_millis(500)).await;
        }
    };
    
    let receive_task = async {
        loop {
            match host_receiver.receive().await {
                Ok(data) => {
                    log::info!("Received {} bytes", data.len());
                    // Process received data
                }
                Err(e) => {
                    log::error!("Receive error: {:?}", e);
                }
            }
        }
    };
    
    join(send_task, receive_task).await;
}
```

## Configuration

### USB Descriptor Configuration

The device uses the following USB configuration:

```rust
// USB Vendor/Product ID
const VID: u16 = 0x303A; // Espressif VID
const PID: u16 = 0x3001; // Custom PID

// Device information
manufacturer: "Espressif"
product: "USB-serial example" 
serial_number: "12345678"

// CDC-ACM composite device configuration
device_class: 0xEF
device_sub_class: 0x02
device_protocol: 0x01
composite_with_iads: true
```

### Buffer Configuration

Buffer sizes are configured at compile time using const generics:

```rust
const BUFFER_SIZE: usize = 64;    // USB packet size
const CHANNEL_SIZE: usize = 10;   // Embassy channel depth

type Device = CdcAcmDeviceHost<CHANNEL_SIZE, BUFFER_SIZE>;
```

### ESP32 Variant Support

The crate supports ESP32 variants with USB OTG capabilities. Ex. esp32s2, esp32s3, etc.

## API Reference

### CdcAcmDeviceHost

Main USB device structure implementing async host processor.

#### Type Parameters

- `CH_SIZE`: Embassy channel buffer size
- `BUFFER_SIZE`: USB packet buffer size (typically 64 bytes)

#### Methods

##### `new()`

```rust
pub fn new(
    usb: Usb<'a>,
    ep_out_buffer: &'a mut [u8; 1024],
    config_descriptor: &'a mut [u8; 256],
    bos_descriptor: &'a mut [u8; 256], 
    control_buf: &'a mut [u8; 64],
    state: &'a mut State<'a>,
) -> Self
```

Creates a new CDC-ACM device with the specified buffers and USB peripheral.

##### `processors()`

```rust
fn processors<'ch>(
    mut self,
    to: Self::T<'ch>,
    from: Self::T<'ch>,
) -> Result<(
    impl Future<Output = ()>,
    AsyncHostSender<'ch, CriticalSectionRawMutex, BUFFER_SIZE, CH_SIZE>,
    AsyncHostReceiver<'ch, CriticalSectionRawMutex, BUFFER_SIZE, CH_SIZE>,
)>
```

Initializes the device processors and returns the async runner future and communication channels.

### AsyncHostProcessor Trait

The device implements the `AsyncHostProcessor` trait from the protocol crate:

```rust
impl AsyncHostProcessor<CH_SIZE, BUFFER_SIZE, CriticalSectionRawMutex, Error>
    for CdcAcmDeviceHost<CH_SIZE, BUFFER_SIZE>
```

## Thread Safety and Concurrency

### Embassy Channels

The implementation uses Embassy's channel primitives for safe async communication:

- **CriticalSectionRawMutex**: Provides thread-safety for embedded systems
- **Channel**: Type-safe async message passing
- **Sender/Receiver**: Split channel interfaces for unidirectional communication

### Async Safety

All operations are designed to be await-safe:

- **Connection Handling**: Automatic reconnection on USB disconnect
- **Buffer Management**: Safe buffer sharing with mutex protection
- **Error Recovery**: Graceful handling of endpoint errors

## Error Handling

### Error Types

Currently minimal error handling with room for expansion:

```rust
pub enum Error {}
pub type Result<T> = core::result::Result<T, Error>;
```

### USB Error Recovery

The implementation handles common USB errors:

- **EndpointError::BufferOverflow**: Continues processing, skips corrupted packet
- **EndpointError::Disabled**: Waits for reconnection
- **Timeout Errors**: Continues with timeout retry logic

### Connection Management

Automatic handling of USB connection lifecycle:

```rust
// Wait for connection before processing
class.wait_connection().await;

// Handle disconnection gracefully
EndpointError::Disabled => {
    log::warn!("USB Disconnected. Retrying");
    continue 'conn;
}
```

## Performance Considerations

### Async Performance

- **Zero-Copy Operations**: Direct buffer operations where possible
- **Concurrent Tasks**: Separate async tasks for read/write operations
- **Efficient Timeouts**: 1ms read timeout for responsive operation
- **Channel Buffering**: Configurable channel depth for backpressure management

### Memory Usage

- **Stack-Allocated Buffers**: All buffers are statically allocated
- **No Dynamic Allocation**: Fully no_std compatible
- **Compile-Time Configuration**: Buffer sizes known at compile time

### Real-Time Characteristics

- **Predictable Latency**: Embassy async runtime provides predictable scheduling
- **Interrupt-Driven**: USB operations handled via interrupts
- **Low Overhead**: Minimal runtime overhead compared to blocking implementations

## Dependencies

### Embassy Crates

- **embassy-executor**: Async task executor with configurable arena size
- **embassy-time**: Time utilities with generic queue support  
- **embassy-usb**: USB device stack implementation
- **embassy-sync**: Synchronization primitives (channels, mutexes)
- **embassy-futures**: Async utilities and combinators

### ESP32 Crates

- **esp-hal**: Hardware abstraction layer with USB OTG support
- **esp-alloc**: Memory allocator for ESP32 targets
- **log**: Logging framework for debugging

### Protocol Integration

- **protocol**: Communication protocol definitions with quick protobuf support
- **lib_utils**: Common utilities and helpers

## Memory Requirements

### Static Allocations

The implementation requires several static buffers:

```rust
static mut EP_OUT_BUFFER: [u8; 1024] = [0; 1024];    // USB endpoint buffer
static mut CONFIG_DESCRIPTOR: [u8; 256] = [0; 256];   // USB config descriptor
static mut BOS_DESCRIPTOR: [u8; 256] = [0; 256];      // USB BOS descriptor  
static mut CONTROL_BUF: [u8; 64] = [0; 64];          // USB control buffer
static mut STATE: State = State::new();               // CDC-ACM state
```

### Runtime Memory

- **Channel Buffers**: `CH_SIZE * BUFFER_SIZE` bytes per channel
- **Task Stacks**: Managed by Embassy executor
- **USB Buffers**: Additional USB stack requirements

## Building

This crate is designed for bare-metal ESP32 projects. Add to your `Cargo.toml`:

```toml
[dependencies]
device-embassy = { path = "../device-embassy" }
embassy-executor = { version = "0.7.0", features = ["task-arena-size-65536"] }
esp-hal = { version = "0.23.0", features = ["esp32s3", "unstable"] }
```

Build with the appropriate target:

```bash
# For ESP32-S3
cargo build --target xtensa-esp32s3-none-elf --release

# For ESP32-S2  
cargo build --target xtensa-esp32s2-none-elf --release
```

## Debugging

Enable logging for USB operations:

```rust
log::info!("USB device initialized");
log::warn!("USB Disconnected. Retrying");
log::error!("Timeout error: {:?}", e);
```

Common debug points:
- USB device enumeration and connection
- Data transfer success/failure
- Channel send/receive operations
- Endpoint error conditions

## Limitations

### Hardware Limitations

- Only supports ESP32 variants with USB OTG (S2, S3)
- Single USB device per application
- Fixed USB configuration at compile time

### Software Limitations

- Fixed buffer sizes (compile-time configuration only)
- No dynamic descriptor modification
- Limited error type granularity
- Requires Embassy async runtime

### Protocol Limitations

- Only implements AsyncHostProcessor (not plugin processor)
- Fixed packet size determined by BUFFER_SIZE
- Channel sizes must be configured at compile time

## Examples

The crate includes several usage patterns:

1. **Basic Communication**: Simple send/receive operations
2. **Connection Handling**: Robust USB connection management
3. **Error Recovery**: Handling various USB error conditions
4. **Performance Optimization**: Efficient async task coordination

## References

The implementation builds on:

- [Embassy Project](https://embassy.dev/) - Async embedded framework
- [Embassy USB](https://docs.embassy.dev/embassy-usb/) - USB device stack
- [ESP-HAL](https://github.com/esp-rs/esp-hal) - ESP32 hardware abstraction
- [USB CDC-ACM Specification](https://www.usb.org/sites/default/files/CDC1.2_WMC1.1_012011.pdf)

## License

Copyright © 2025 Wanyeki Technologies LLC. All rights reserved.

This project is private and proprietary.