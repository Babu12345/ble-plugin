#![no_std]

// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

#![no_main]

use device_embassy_uart::DeviceHostUart;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use esp_backtrace as _;
use esp_hal::{
    Async,
    clock::CpuClock,
    timer::systimer::SystemTimer,
    uart::{Config, Uart},
};
use host_uart_no_std::configs::initalize_logger;
use protocol::DEFAULT_PACKET_SIZE;
use protocol::devices::host::AsyncHostProcessor;

// UART serial device example: https://github.com/esp-rs/esp-hal/blob/main/examples/async/embassy_serial/src/main.rs
#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    esp_alloc::heap_allocator!(size: 72 * 1024);

    initalize_logger().ok();

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    let uart = Uart::new(peripherals.UART0, Config::default())
        .unwrap()
        .with_tx(peripherals.GPIO21)
        .with_rx(peripherals.GPIO20)
        .into_async();

    let device_host_uart =
        DeviceHostUart::<'_, Async, 20, DEFAULT_PACKET_SIZE, NoopRawMutex>::new(uart);

    let to = Channel::<NoopRawMutex, _, 20>::new();
    let from = Channel::<NoopRawMutex, _, 20>::new();

    let (processor_fn, _sender, _receiver) = device_host_uart
        .processors(
            (to.sender(), to.receiver()),
            (from.sender(), from.receiver()),
        )
        .unwrap();

    processor_fn.await;
}
