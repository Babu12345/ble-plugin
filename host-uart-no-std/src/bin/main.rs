#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{clock::CpuClock, timer::systimer::SystemTimer};
use host_uart_no_std::configs::initalize_logger;
use protocol::DEFAULT_PACKET_SIZE;
use protocol::devices::host::AsyncHostProcessor;

// USB serial device example: https://github.com/esp-rs/esp-hal/blob/main/examples/async/embassy_usb_serial_jtag/src/main.rs
#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    esp_alloc::heap_allocator!(size: 72 * 1024);

    initalize_logger().ok();

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);
}
