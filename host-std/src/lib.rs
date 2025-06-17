//! Host device interface. This will use spi as interface that will communicate with the primary but it can also use i2c. USB directly or any other type of interface.
#![deny(missing_docs)]

/// Common util functions
pub mod utils;

use std::thread::Scope;

use esp_idf_sys::{
    cherry_host::{
        self, cdc_line_coding, usbh_cdc_acm, usbh_cdc_acm_bulk_out_transfer, usbh_cdc_acm_run,
        usbh_cdc_acm_set_line_coding, usbh_cdc_acm_set_line_state, usbh_find_class_instance,
        usbh_initialize, ESP_USBH_BASE,
    },
    esp_netif_init,
    host::ESP_OK,
};

/// Initialization
/// https://github.com/zleihao/CherryUSB-CDC-MSC/blob/50095e0b63bbdf6f2d5597e71edfa45dd8be6c1d/cdc_msc/middlewares/CherryUSB-1.4.0/class/cdc/usbh_cdc_acm.c#L170
/// https://github.com/cherry-embedded/CherryUSB/blob/f23f5494920b64987350abc87c8154f410c6f5f9/platform/nuttx/usbh_serial.c#L180
/// https://github.com/search?q=repo%3Acherry-embedded%2FCherryUSB%20usbh_cdc_acm_run&type=code
/// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/host/cdc_acm/src/cdc_acm_host.c#L33
pub unsafe fn cherry_usb_host<'a, 'b>(_scope: &Scope<'a, 'b>) {
    unsafe {
        // TODO: Might not be needed
        let res = esp_netif_init();
        if res != ESP_OK {}
        usbh_initialize(0, ESP_USBH_BASE as usize);
        // esp_idf_sys::cherry_host::usbh_cdc_acm_run(cdc_acm_class);
        // usbh_cdc_acm_set_line_state(cdc_acm_class, dtr, rts)
        let mut cdc_acm_class = usbh_cdc_acm::default();
        let mut line_coding = cdc_line_coding::default();
        line_coding.dwDTERate = 115200;
        // usbh_find_class_instance("")
        usbh_cdc_acm_set_line_coding(&mut cdc_acm_class, &mut line_coding);
        usbh_cdc_acm_set_line_state(&mut cdc_acm_class, true, false);
        // usbh_cdc_acm_bulk_out_transfer(cdc_acm_class, buffer, buflen, timeout);
        // usbh_int_u
        // usbh_bulk
        // usbh_cdc_
        usbh_cdc_acm_run(&mut cdc_acm_class);
    }
}
