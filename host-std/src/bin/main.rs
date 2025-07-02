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
use host_esp::usb_host;
use protocol::host::{BulkHostCommand, HostCommand, HostCommandTypes::*};
use uuid::Uuid;

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

    // std::thread::scope(|scope| {
    //     scope.spawn(move || {
    //         let io = unsafe { usb_host(scope, 100) };
    //         loop {
    //             io.0.send(BulkHostCommand {
    //                 commands: vec![HostCommand {
    //                     id: Uuid::from_u128(0x00),
    //                     cmd: ConfigPeripheral(
    //                         String::from_str("Default").unwrap(),
    //                         Uuid::from_u128(0xff),
    //                     ),
    //                 }],
    //             })
    //             .ok();
    //         }
    //     });
    // });

    std::thread::scope(|scope| {
        let io = unsafe { cherry_usb_host(scope, 100) };
        scope.spawn(move || loop {
            io.0.send(BulkHostCommand {
                commands: vec![0x02, 0x01, 0x00]
                    .into_iter()
                    .enumerate()
                    .map(|x| HostCommand {
                        id: Uuid::from_u128(x.0 as u128),
                        cmd: ConfigPeripheral(
                            String::from_str(format!("Name w/ uuid: {}", x.1).as_str()).unwrap(),
                            Uuid::from_u128(x.1),
                        ),
                    })
                    .collect(),
            })
            .ok();
        });

        scope.spawn(move || loop {
            let _data: BulkHostCommand = io.1.receive().unwrap().decode().unwrap();
            std::thread::sleep(Duration::from_millis(10));
        });
    });

    Ok(())
} // See https://github.com/espressif/esp-idf/blob/v5.4.1/examples/peripherals/usb/host/cdc/cdc_acm_host/main/usb_cdc_example_main.c
