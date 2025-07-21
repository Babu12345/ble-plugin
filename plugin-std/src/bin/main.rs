use std::time::Duration;

use esp32_nimble::{
    enums::{AuthReq, SecurityIOCap},
    BLEDevice,
};
use esp_idf_sys::cherry_device::ESP_USBD_BASE;
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
        let processors = unsafe { CdcAcmDevice::new().init(0, ESP_USBD_BASE) }
            .unwrap()
            .processors(0, scope, 50)
            .unwrap();

        scope.spawn(move || loop {
            let data = processors.1.recv().unwrap();
            log::info!("Data aquired: {:?}", data);
        });
        scope.spawn(move || loop {
            processors.0.send(b"Hello\n".match_size(0)).ok();
            std::thread::sleep(Duration::from_secs(1));
        });
    });
}
