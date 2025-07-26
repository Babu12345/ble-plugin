use std::time::Duration;

use device_cherry::CdcAcmDevice;
use esp32_nimble::{
    enums::{AuthReq, SecurityIOCap},
    BLEDevice,
};
use esp_idf_sys::cherry_device::ESP_USBD_BASE;
use protocol::types::{HostCommandConfigurePeripheral, PluginData};
use uuid::{self, Uuid};
// Examples: https://github.com/taks/esp32-nimble/tree/main/examples
fn main() {
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
        let device = CdcAcmDevice::new()
            .init(0, ESP_USBD_BASE)
            .unwrap()
            .set_dtr(0, true);
        let processors = device.processors(scope, 20);

        scope.spawn(move || loop {
            let received_data = processors.1.receive().unwrap();
            let data: Option<HostCommandConfigurePeripheral> = received_data.decode().ok();

            log::info!("Data aquired: {:?}", data);
        });
        scope.spawn(move || loop {
            processors
                .0
                .send(PluginData {
                    src_id: Uuid::from_u128(0x01),
                    send_type: protocol::types::PluginDataSendType::Notify,
                    data: b"Hello\n",
                })
                .ok();
            std::thread::sleep(Duration::from_secs(1));
        });
    });
}
