#![no_std]
#![no_main]
#![feature(never_type)]

use bt_hci::controller::ExternalController;
use embassy_executor::Spawner;
use embassy_usb::class::cdc_acm::State;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, otg_fs::Usb, rng::Trng, timer::systimer::SystemTimer};
use esp_wifi::{EspWifiController, ble::controller::BleConnector};
use log::error;
use plugin_no_std::{
    ble,
    configs::{self, BUFFER_SIZE, ble_config, initalize_logger, usb_device_config},
    mk_static,
    tasks::{ble_processor, ble_runner, usb_device_processor, usb_device_runner},
    utils::await_indefinitely,
};
use trouble_host::{Host, Stack};

// BLE no-std example: https://github.com/embassy-rs/trouble/blob/main/examples/apps/src/ble_bas_peripheral_sec.rs
// USB device example: https://github.com/esp-rs/esp-hal/blob/main/examples/src/bin/embassy_usb_serial.rs
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

    let crypto_random_generator = mk_static!(Trng, Trng::new(peripherals.RNG, peripherals.ADC1));

    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let init = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(
            timer1.timer0,
            crypto_random_generator.rng.clone(),
            peripherals.RADIO_CLK,
        )
        .unwrap()
    );

    let (class, device) = usb_device_config(
        Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19),
        &mut *mk_static!(State<'static>, State::new()),
        &mut *mk_static!([u8; 256], [0; 256]),
        &mut *mk_static!([u8; 256], [0; 256]),
        &mut *mk_static!([u8; BUFFER_SIZE], [0; BUFFER_SIZE]),
        &mut *mk_static!([u8; 1024], [0; 1024]),
    );

    let (stack, server) = ble_config(
        BleConnector::new(&init, peripherals.BT),
        crypto_random_generator,
    );
    let Host { runner, .. } = mk_static!(
        Stack<'static, ExternalController<BleConnector<'static>, 40>>,
        stack
    )
    .build();

    spawner.must_spawn(usb_device_runner(device));
    spawner.must_spawn(ble_runner(runner));
    spawner
        .spawn(usb_device_processor(class))
        .inspect_err(|_| error!("Failed to spawn the usb device processor"))
        .ok();
    spawner
        .spawn(ble_processor(server))
        .inspect_err(|_| error!("Failed to spawn the BLE processor"))
        .ok();

    await_indefinitely().await
}
