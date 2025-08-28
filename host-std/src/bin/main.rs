use std::{str::FromStr, time::Duration};

use esp_idf_svc::hal::{
    prelude::Peripherals,
    spi::{
        config::{Config, DriverConfig},
        SpiDeviceDriver, SpiDriver,
    },
    units::Hertz,
};
use heapless::String;
use host_cherry::cherry_usb_host;
use protocol::io_types::{HostCommandConfigurePeripheral, HostCommandConfigureService, PluginData};

/**
 * General protocal is as follows:
 * There will be 2 tasks. One task is responsible for sending commands and data.
 * The other is responsible for processing such commands and data. The commands
 * and data bytes will each be structured and typesafe as well as the responses to
 * any of the commands. By using a channel we can also dictate how large the buffer should
 * be as well as make sure the order of the commands and data is what we expect.
 */
fn main() -> anyhow::Result<()> {
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

    // use host_esp::usb_host;
    // std::thread::scope(|scope| {
    //     scope.spawn(move || {
    //         let io = unsafe { usb_host(scope, 100) };
    //     });
    // });

    std::thread::scope(|scope| {
        let io = unsafe { cherry_usb_host(scope, 200) };

        scope.spawn(move || loop {
            io.0.send(HostCommandConfigurePeripheral {
                addr: &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
                name: String::from_str("Portrait").unwrap(),
            })
            .ok();
            io.0.send(HostCommandConfigureService { uuid: 0xAAAA }).ok();

            std::thread::sleep(Duration::from_millis(20));
        });

        scope.spawn(move || loop {
            std::thread::sleep(Duration::from_millis(10));

            let data = io.1.receive().unwrap();

            let plugin_data: Option<PluginData> = data.decode().ok();
            if let Some(data) = plugin_data {
                log::info!("{:?}", data)
            }
        });
    });

    Ok(())
} // See https://github.com/espressif/esp-idf/blob/v5.4.1/examples/peripherals/usb/host/cdc/cdc_acm_host/main/usb_cdc_example_main.c
