use std::time::Duration;

use esp32_nimble::{
    enums::{AuthReq, SecurityIOCap},
    BLEDevice,
};

use host_cherry::cherry_usb_host_for_plugin;
use protocol::protocol::{HostCommandConfigurePeripheral, HostCommandConfigureService, PluginData};

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
        let io = unsafe { cherry_usb_host_for_plugin(scope, 200) };

        scope.spawn(move || loop {
            io.0.send(PluginData {
                src_addr: Vec::from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
                src_addr_type: protocol::protocol::BluetoothAddressType::Public as _,
                send_type: protocol::protocol::PluginDataSendType::NotifyType as _,
                characteristic_uuid: 0x2A29,
                service_uuid: 0x180A,
                data: Vec::from(b"Data incoming\0"),
            })
            .ok();

            std::thread::sleep(Duration::from_millis(20));
        });

        scope.spawn(move || loop {
            std::thread::sleep(Duration::from_millis(10));

            let data = io.1.receive().unwrap();

            let host_cmd: Option<HostCommandConfigurePeripheral> = data.decode().ok();
            if let Some(cmd) = host_cmd {
                log::info!("{:?}", cmd)
            }

            let host_cmd: Option<HostCommandConfigureService> = data.decode().ok();
            if let Some(cmd) = host_cmd {
                log::info!("{:?}", cmd)
            }
        });
    });

    Ok(())
} // See https://github.com/espressif/esp-idf/blob/v5.4.1/examples/peripherals/usb/host/cdc/cdc_acm_host/main/usb_cdc_example_main.c
