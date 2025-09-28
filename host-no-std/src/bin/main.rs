#![no_std]
#![no_main]
#![feature(never_type)]

use device_embassy::processors::CdcAcmDeviceHost;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_usb::class::cdc_acm::State;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, otg_fs::Usb, timer::systimer::SystemTimer};
use host_no_std::configs::initalize_logger;
use protocol::DEFAULT_PACKET_SIZE;
use protocol::devices::host::AsyncHostProcessor;

// BLE no-std example: https://github.com/embassy-rs/trouble/blob/main/examples/apps/src/ble_bas_peripheral_sec.rs
// USB device example: https://github.com/esp-rs/esp-hal/blob/main/examples/src/bin/embassy_usb_serial.rs
#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    esp_alloc::heap_allocator!(size: 72 * 1024);

    initalize_logger().ok();

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);

    let mut ep_out_buffer = [0; 1024];
    let mut config_descriptor = [0; 512];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 128];
    let mut state = State::new();

    let device_host: CdcAcmDeviceHost<'_, 20, DEFAULT_PACKET_SIZE, NoopRawMutex> =
        CdcAcmDeviceHost::new(
            usb,
            &mut ep_out_buffer,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut control_buf,
            &mut state,
        );

    let to = Channel::<NoopRawMutex, _, 20>::new();
    let from = Channel::<NoopRawMutex, _, 20>::new();

    let (processor_fn, _sender, _receiver) = device_host
        .processors(
            (to.sender(), to.receiver()),
            (from.sender(), from.receiver()),
        )
        .unwrap();

    processor_fn.await;
}
