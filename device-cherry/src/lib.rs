//! USB host implementation library of cherry usb
#[deny(missing_docs)]
mod constants;
mod processors;
mod utils;
use processors::*;
use protocol::host::{HostReceiver, HostSender};

use std::{sync::mpsc::sync_channel, thread::Scope};

use esp_idf_sys::cherry_device::ESP_USBD_BASE;
// Initialization - host
// https://github.com/zleihao/CherryUSB-CDC-MSC/blob/50095e0b63bbdf6f2d5597e71edfa45dd8be6c1d/cdc_msc/middlewares/CherryUSB-1.4.0/class/cdc/usbh_cdc_acm.c#L170
// https://github.com/cherry-embedded/CherryUSB/blob/f23f5494920b64987350abc87c8154f410c6f5f9/platform/nuttx/usbh_serial.c#L180
// https://github.com/search?q=repo%3Acherry-embedded%2FCherryUSB%20usbh_cdc_acm_run&type=code
// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/host/cdc_acm/src/cdc_acm_chost.c#L33
// https://github.com/CherryUSB/cherryusb_esp32/blob/main/examples/host/sdkconfig

// Initialization - device
// https://github.com/hpmicro/zephyr_sdk_glue/tree/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device

/// Initialize the usb host and send out receivers and senders to process and send information to the connected usb device via the cdc acm driver class.
pub unsafe fn cherry_usb_host<'a, 'b>(
    scope: &'a Scope<'a, 'b>,
    channel_buffer_size: usize,
) -> (HostSender<256>, HostReceiver<256>) {
    unsafe { cdc_init(0, ESP_USBD_BASE) };
    todo!()
}
