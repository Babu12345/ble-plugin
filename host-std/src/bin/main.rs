use esp_idf_svc::hal::{
    prelude::Peripherals,
    spi::{
        config::{Config, DriverConfig},
        SpiDeviceDriver, SpiDriver,
    },
    units::Hertz,
};
use esp_idf_sys::cherry_host::usbh_initialize;
use heapless::String;
use host_esp::usb_host;
use std::{str::FromStr, time::Duration};

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let mosi = peripherals.pins.gpio9;
    let miso = peripherals.pins.gpio8;
    let sclk = peripherals.pins.gpio7;
    let cs = peripherals.pins.gpio1;

    let _spi: SpiDeviceDriver<'_, SpiDriver<'_>> = SpiDeviceDriver::new_single(
        peripherals.spi2,
        sclk,
        mosi,
        Some(miso),
        Some(cs),
        &DriverConfig::default(),
        &Config::default().baudrate(Hertz(80_000_000)),
    )
    .unwrap();

    std::thread::scope(|scope| unsafe {
        let io = usb_host(scope, 100);
        scope.spawn(move || {
            let mut i = 0;
            loop {
                io.sender
                    .send(String::from_str(format!("{i}").as_str()).unwrap())
                    .ok();
                std::thread::sleep(Duration::from_millis(50));
                i = i + 1;
            }
        });
    });
} // See https://github.com/espressif/esp-idf/blob/v5.4.1/examples/peripherals/usb/host/cdc/cdc_acm_host/main/usb_cdc_example_main.c
