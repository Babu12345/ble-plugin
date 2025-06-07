#![no_std]
#![no_main]
#![feature(never_type)]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, timer::systimer::SystemTimer};
use host_no_std::configs::initalize_logger;

// BLE no-std example: https://github.com/embassy-rs/trouble/blob/main/examples/apps/src/ble_bas_peripheral_sec.rs
// USB device example: https://github.com/esp-rs/esp-hal/blob/main/examples/src/bin/embassy_usb_serial.rs
#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::heap_allocator!(72 * 1024);

    initalize_logger().ok();

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);
}
