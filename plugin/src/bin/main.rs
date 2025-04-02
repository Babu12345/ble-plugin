use esp32_nimble::{
    enums::{AuthReq, SecurityIOCap},
    BLEDevice,
};
use esp_idf_sys::{tinyusb_config_t, tinyusb_driver_install, ESP_OK};

// See https://github.com/taks/esp32-nimble/blob/main/examples/ble_secure_server.rs
// See https://github.com/espressif/esp-idf/blob/v5.4.1/examples/peripherals/usb/device/tusb_serial_device/main/tusb_serial_device_main.c
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

    let tusb_config = tinyusb_config_t {
        external_phy: false,
        self_powered: false,
        ..Default::default()
    };

    unsafe {
        let res = tinyusb_driver_install(&tusb_config);
        if res != ESP_OK {
            log::error!("Error installing driver")
        }
    }
}
