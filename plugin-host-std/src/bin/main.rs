use std::{str::FromStr, time::Duration};

use esp32_nimble::{
    enums::{AuthReq, SecurityIOCap},
    BLEDevice,
};
use heapless::String;
use host_cherry::cherry_usb_host;
use plugin_host_std::utils::random_uuid;
use protocol::types::{
    HostCommandConfigurePeripheral, HostCommandConfigureService, HostReceivedData, PluginData,
};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let device = BLEDevice::take();
    let _ble_advertising = device.get_advertising();

    device
        .security()
        .set_auth(AuthReq::all())
        .set_passkey(123456)
        .set_io_cap(SecurityIOCap::DisplayOnly)
        .resolve_rpa();

    std::thread::scope(|scope| {
        let io = unsafe { cherry_usb_host(scope, 200) };

        scope.spawn(move || loop {
            io.0.send(HostCommandConfigurePeripheral {
                uuid: unsafe { random_uuid() },
                name: String::from_str("Portrait").unwrap(),
            })
            .ok();
            io.0.send(HostCommandConfigureService {}).ok();

            std::thread::sleep(Duration::from_millis(20));
        });

        scope.spawn(move || loop {
            std::thread::sleep(Duration::from_millis(10));

            let data = io.1.receive().unwrap();

            let bulk_cmd: Option<HostCommandConfigurePeripheral> = data.decode().ok();
            if let Some(cmd) = bulk_cmd {
                log::info!("{:?}", cmd)
            }

            let bulk_data: Option<HostCommandConfigureService> = data.decode().ok();
            if let Some(data) = bulk_data {
                log::info!("{:?}", data)
            }
        });
    });

    Ok(())
} // See https://github.com/espressif/esp-idf/blob/v5.4.1/examples/peripherals/usb/host/cdc/cdc_acm_host/main/usb_cdc_example_main.c
