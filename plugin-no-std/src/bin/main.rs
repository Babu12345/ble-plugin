#![no_std]
#![no_main]
#![feature(never_type)]

use embassy_executor::Spawner;
use embassy_usb::class::cdc_acm::State;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, otg_fs::Usb, rng::Rng, timer::systimer::SystemTimer};
use esp_wifi::{EspWifiController, ble::controller::BleConnector};
use log::error;
use plugin_no_std::{
    configs::{BUFFER_SIZE, initalize_logger, start_usb_device},
    mk_static,
    tasks::{usb_device_processor, usb_device_runner},
    utils::await_indefinitely,
};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::heap_allocator!(size: 72 * 1024);

    initalize_logger().ok();

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let init = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(
            timer1.timer0,
            Rng::new(peripherals.RNG),
            peripherals.RADIO_CLK,
        )
        .unwrap()
    );
    let _connector = BleConnector::new(&init, peripherals.BT);

    let (class, device) = start_usb_device(
        Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19),
        &mut *mk_static!(State<'static>, State::new()),
        &mut *mk_static!([u8; 256], [0; 256]),
        &mut *mk_static!([u8; 256], [0; 256]),
        &mut *mk_static!([u8; BUFFER_SIZE], [0; BUFFER_SIZE]),
        &mut *mk_static!([u8; 1024], [0; 1024]),
    );

    spawner.must_spawn(usb_device_runner(device));
    spawner
        .spawn(usb_device_processor(class))
        .inspect_err(|_| error!("Failed to spawn the usb device processor"))
        .ok();

    await_indefinitely().await
}
