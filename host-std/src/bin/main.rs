use esp_idf_svc::hal::{
    prelude::Peripherals,
    spi::{
        config::{Config, DriverConfig},
        SpiDeviceDriver, SpiDriver,
    },
    units::Hertz,
};
use host_cherry::cherry_usb_host;
use host_esp::usb_host;
use protocol::host::{HostCommand, HostCommandTypes};
use uuid::Uuid;

/**
 * General protocal is as follows:
 * There will be 2 tasks. One task is responsible for sending commands and data.
 * The other is responsible for processing such commands and data. The commands
 * and data bytes will each be structured and typesafe as well as the responses to
 * any of the commands. By using a channel we can also dictate how large the buffer should
 * be as well as make sure the order of the commands and data is what we expect.
 */
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

    // std::thread::scope(|scope| {
    //     scope.spawn(move || {
    //         let io = unsafe { usb_host(scope, 100) };
    //         loop {
    //             io.send(HostCommand {
    //                 cmd: HostCommandTypes::ConfigPeripheral("Default name", 0u32),
    //             })
    //             .ok();
    //         }
    //     });
    // });

    std::thread::scope(|scope| {
        scope.spawn(move || {
            let io = unsafe { cherry_usb_host(scope, 100) };
            loop {
                io.send(HostCommand {
                    cmd: HostCommandTypes::ConfigPeripheral("Default name", Uuid::from_u128(0xff)),
                })
                .ok();
            }
        });
    });
} // See https://github.com/espressif/esp-idf/blob/v5.4.1/examples/peripherals/usb/host/cdc/cdc_acm_host/main/usb_cdc_example_main.c
