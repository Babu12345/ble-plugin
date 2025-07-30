use std::time::Duration;

use device_cherry::CdcAcmDevice;
use esp32_nimble::{
    enums::{AuthReq, SecurityIOCap},
    utilities::BleUuid,
    BLEAdvertisementData, BLEDevice, NimbleProperties,
};
use esp_idf_sys::cherry_device::ESP_USBD_BASE;
use protocol::types::{HostCommandConfigurePeripheral, PluginData};
use uuid::{self, Uuid};

fn get_device(name: &'static str) -> &'static mut BLEDevice {
    let device = BLEDevice::take();
    let ble_advertising = device.get_advertising();

    device
        .security()
        .set_auth(AuthReq::all())
        .set_passkey(123456)
        .set_io_cap(SecurityIOCap::DisplayOnly)
        .resolve_rpa();

    let server = device.get_server();
    server.on_connect(|server, desc| {
        ::log::info!("Client connected: {:?}", desc);

        if server.connected_count() < (esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _) {
            ::log::info!("Multi-connect support: start advertising");
            ble_advertising.lock().start().unwrap();
        }
    });
    server.on_disconnect(|_desc, reason| {
        ::log::info!("Client disconnected ({:?})", reason);
    });
    server.on_authentication_complete(|_, desc, result| {
        ::log::info!("AuthenticationComplete({:?}): {:?}", result, desc);
    });

    let service = server.create_service(BleUuid::Uuid16(0xABCD));
    let service2 = server.create_service(BleUuid::Uuid16(0xBABC));

    let non_secure_characteristic = service
        .lock()
        .create_characteristic(BleUuid::Uuid16(0x1234), NimbleProperties::READ);
    non_secure_characteristic
        .lock()
        .set_value("non_secure_characteristic".as_bytes());

    let secure_characteristic = service.lock().create_characteristic(
        BleUuid::Uuid16(0x1235),
        NimbleProperties::READ | NimbleProperties::READ_ENC | NimbleProperties::READ_AUTHEN,
    );
    secure_characteristic
        .lock()
        .set_value("secure_characteristic".as_bytes());

    let service_uuid = service.lock().uuid();
    let service2_uuid = service2.lock().uuid();

    ble_advertising
        .lock()
        .set_data(
            BLEAdvertisementData::new()
                .name(name)
                .add_service_uuid(service_uuid)
                .add_service_uuid(service2_uuid),
        )
        .unwrap();
    ble_advertising.lock().start().unwrap();

    ::log::info!("bonded_addresses: {:?}", device.bonded_addresses());
    return device;
}
// Examples: https://github.com/taks/esp32-nimble/tree/main/examples
fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let _device = get_device("ESP32-GATT-Server");

    let usb_device = CdcAcmDevice::new()
        .init(0, ESP_USBD_BASE)
        .unwrap()
        .set_dtr(0, true);

    std::thread::scope(|scope| {
        let usb_processors = usb_device.processors(scope, 20).unwrap();

        scope.spawn(move || loop {
            let received_data = usb_processors.1.receive().unwrap();
            let data: Option<HostCommandConfigurePeripheral> = received_data.decode().ok();

            log::info!("Data aquired: {:?}", data);
        });
        scope.spawn(move || loop {
            usb_processors
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
