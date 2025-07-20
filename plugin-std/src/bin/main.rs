use std::time::Duration;

use esp32_nimble::{
    enums::{AuthReq, SecurityIOCap},
    BLEDevice,
};
use esp_idf_sys::cherry_device::ESP_USBD_BASE;
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

    let _device = unsafe { CdcAcmDevice::new().init(0, ESP_USBD_BASE) }.unwrap();

    std::thread::scope(|_scope| {

        // scope.spawn(|| loop {
        //     log::info!("Data start");
        //     let mut send_buffer: [u8; 64] = [0; 64];
        //     unsafe { send_data(&mut send_buffer) };
        //     std::thread::sleep(Duration::from_secs(1));
        //     log::info!("Data sent");
        // });
    });

    loop {
        std::thread::sleep(Duration::from_secs(1));
        println!("Hello");
    }
}
