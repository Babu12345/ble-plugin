//! USB host implementation library of cherry usb
#![deny(missing_docs)]
#![cfg(all(target_arch = "xtensa", target_os = "espidf"))]
mod constants;
mod errors;
mod processors;
mod types;
mod utils;

pub use errors::*;
pub use processors::*;
pub use types::*;
pub use utils::*;
// Initialization - host
// https://github.com/zleihao/CherryUSB-CDC-MSC/blob/50095e0b63bbdf6f2d5597e71edfa45dd8be6c1d/cdc_msc/middlewares/CherryUSB-1.4.0/class/cdc/usbh_cdc_acm.c#L170
// https://github.com/cherry-embedded/CherryUSB/blob/f23f5494920b64987350abc87c8154f410c6f5f9/platform/nuttx/usbh_serial.c#L180
// https://github.com/search?q=repo%3Acherry-embedded%2FCherryUSB%20usbh_cdc_acm_run&type=code
// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/host/cdc_acm/src/cdc_acm_chost.c#L33
// https://github.com/CherryUSB/cherryusb_esp32/blob/main/examples/host/sdkconfig

// Initialization - device
// https://github.com/hpmicro/zephyr_sdk_glue/tree/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device
