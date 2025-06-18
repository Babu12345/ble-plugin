//! Host device interface. This will use spi as interface that will communicate with the primary but it can also use i2c. USB directly or any other type of interface.
#![deny(missing_docs)]

/// Common util functions
pub mod utils;

use std::{
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, LazyLock, Mutex,
    },
    thread::Scope,
};

use esp_idf_sys::{
    cherry_host::{
        cdc_line_coding, usbh_cdc_acm, usbh_cdc_acm_bulk_in_transfer,
        usbh_cdc_acm_bulk_out_transfer, usbh_cdc_acm_set_line_coding, usbh_cdc_acm_set_line_state,
        usbh_initialize, ESP_USBH_BASE,
    },
    esp_netif_init,
    host::ESP_OK,
};
use heapless::{String, Vec};

type T = String<256>;

/// Usb host input/output
pub struct IO {
    sender: SyncSender<T>,
    receiver: Receiver<T>,
}

static LOCKER: LazyLock<Mutex<Wrapper>> = LazyLock::new(|| Mutex::new(Wrapper::default()));

struct Wrapper(usbh_cdc_acm);
unsafe impl Send for Wrapper {}
unsafe impl Sync for Wrapper {}
impl Default for Wrapper {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Initialization
/// https://github.com/zleihao/CherryUSB-CDC-MSC/blob/50095e0b63bbdf6f2d5597e71edfa45dd8be6c1d/cdc_msc/middlewares/CherryUSB-1.4.0/class/cdc/usbh_cdc_acm.c#L170
/// https://github.com/cherry-embedded/CherryUSB/blob/f23f5494920b64987350abc87c8154f410c6f5f9/platform/nuttx/usbh_serial.c#L180
/// https://github.com/search?q=repo%3Acherry-embedded%2FCherryUSB%20usbh_cdc_acm_run&type=code
/// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/host/cdc_acm/src/cdc_acm_host.c#L33
pub unsafe fn cherry_usb_host<'a, 'b>(scope: &'a Scope<'a, 'b>, channel_buffer_size: usize) -> IO {
    let to_usb = sync_channel(channel_buffer_size);
    let from_usb = sync_channel(channel_buffer_size);

    // TODO: Might not be needed
    let res = esp_netif_init();
    if res != ESP_OK {
        panic!("Bad response")
    }
    usbh_initialize(0, ESP_USBH_BASE as usize);

    {
        let mut lock = LOCKER.lock().unwrap();
        let cdc_acm_class = &mut lock.0;
        let mut line_coding = cdc_line_coding::default();
        line_coding.dwDTERate = 115200;
        line_coding.bDataBits = 8;
        line_coding.bParityType = 0;
        line_coding.bCharFormat = 0;
        usbh_cdc_acm_set_line_coding(cdc_acm_class, &mut line_coding);
        usbh_cdc_acm_set_line_state(cdc_acm_class, true, false);
    }

    scope.spawn(|| send_usb_data(to_usb.1));
    scope.spawn(|| receive_usb_data(from_usb.0));

    IO {
        sender: to_usb.0,
        receiver: from_usb.1,
    }
}

unsafe fn receive_usb_data(sender: SyncSender<T>) {
    let timeout = 0xfffffff;
    loop {
        let mut buffer = [0u8; 256];
        let _ret = usbh_cdc_acm_bulk_in_transfer(
            &mut LOCKER.lock().unwrap().0,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            timeout,
        );
        let data = Vec::from_slice(&buffer).unwrap();
        sender.send(String::from_utf8(data).unwrap()).unwrap();
    }
}

unsafe fn send_usb_data(receiver: Receiver<T>) {
    let timeout = 0xfffffff;
    loop {
        let mut data = match receiver.recv() {
            Ok(data) => data,
            Err(e) => {
                log::info!("Error occurred {e}");
                continue;
            }
        };
        usbh_cdc_acm_bulk_out_transfer(
            &mut LOCKER.lock().unwrap().0,
            data.as_mut_ptr(),
            data.len() as u32,
            timeout,
        );
    }
}
