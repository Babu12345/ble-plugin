//! USB host implementation library of cherry usb
mod constants;
#[deny(missing_docs)]
mod processors;

#[allow(unused_imports)]
use esp_idf_sys::cherry_host::usbh_initialize;
use esp_idf_sys::esp_netif_init;
// pub unsafe fn cherry_usb_host() {
//     unsafe {
//         let res = esp_netif_init();
//         usbh_initialize();
//     }
// }
