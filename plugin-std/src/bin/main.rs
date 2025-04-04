use std::thread::Scope;

use esp32_nimble::{
    enums::{AuthReq, SecurityIOCap},
    BLEDevice,
};
use esp_idf_sys::{tinyusb_config_t, tinyusb_driver_install, ESP_OK};
use log::*;
// void tinyusb_cdc_rx_callback(int itf, cdcacm_event_t *event)
// {
//     /* initialization */
//     size_t rx_size = 0;

//     /* read */
//     esp_err_t ret = tinyusb_cdcacm_read(itf, rx_buf, CONFIG_TINYUSB_CDC_RX_BUFSIZE, &rx_size);
//     if (ret == ESP_OK) {

//         app_message_t tx_msg = {
//             .buf_len = rx_size,
//             .itf = itf,
//         };

//         memcpy(tx_msg.buf, rx_buf, rx_size);
//         xQueueSend(app_queue, &tx_msg, 0);
//     } else {
//         ESP_LOGE(TAG, "Read Error");
//     }
// }

#[no_mangle]
unsafe extern "C" fn data_rx_handle(device_index: u8, event: cdcacm_event) -> bool {
    let args = args as *mut u8;
    let input_args = core::slice::from_raw_parts(args, 10);
    info!("Input arguments: {:?}", input_args);
    let data = core::slice::from_raw_parts(data, data_len);
    info!("Data received: {:?}", data);
    true
}

unsafe fn start_usb_device<'a, 'b>(s: &'a Scope<'a, 'b>) {
    let tusb_config = tinyusb_config_t {
        external_phy: false,
        self_powered: false,
        ..Default::default()
    };

    let res = tinyusb_driver_install(&tusb_config);
    if res != ESP_OK {
        log::error!("Error installing driver")
    }
}

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

    unsafe {
        std::thread::scope(|s| {
            start_usb_device(s);
            // s.spawn(move || process_usb_cdc_host(spi));
        });
    }
}
