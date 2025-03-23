use esp32_nimble::{
    enums::{AuthReq, SecurityIOCap},
    BLEDevice,
};

// See https://github.com/taks/esp32-nimble/blob/main/examples/ble_secure_server.rs
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
