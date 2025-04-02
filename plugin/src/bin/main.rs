use esp32_nimble::{
    enums::{AuthReq, SecurityIOCap},
    BLEDevice,
};

// See https://github.com/taks/esp32-nimble/blob/main/examples/ble_secure_server.rs
// See https://github.com/espressif/esp-idf/blob/v5.4.1/examples/peripherals/usb/device/tusb_serial_device/main/tusb_serial_device_main.c
fn main() {
    // Can I re-use embassy_usb and NimBLE for usb device data transfer
    println!("Hello, world!");
    let device = BLEDevice::take();
    let _ble_advertising = device.get_advertising();

    device
        .security()
        .set_auth(AuthReq::all())
        .set_passkey(123456)
        .set_io_cap(SecurityIOCap::DisplayOnly)
        .resolve_rpa();
}
