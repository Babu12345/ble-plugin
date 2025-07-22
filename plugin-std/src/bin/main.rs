use std::time::Duration;

use esp32_nimble::{
    enums::{AuthReq, SecurityIOCap},
    BLEDevice,
};
use esp_idf_sys::cherry_device::ESP_USBD_BASE;
use heapless::{String, Vec};
use lib_utils::MatchSliceLengths;
use plugin_std::usb_device::CdcAcmDevice;
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
            .set_dtr(0, 0, true);
        let processors = device.processors(0, scope, 20).unwrap();

        scope.spawn(move || loop {
            let data = processors.1.recv().unwrap();
            log::info!(
                "Data aquired: {:?}",
                String::from_utf8(Vec::<u8, 64>::from_slice(&data).unwrap())
            );
        });
        scope.spawn(move || loop {
            processors.0.send(b"Hello\n".match_size(0)).ok();
            std::thread::sleep(Duration::from_secs(1));
        });
    });
}
