//! Host device interface. This will use spi as interface that will communicate with the primary but it can also use i2c. USB directly or any other type of interface.
#![deny(missing_docs)]

/// Common util functions
pub mod utils;

use std::{
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        OnceLock,
    },
    thread::Scope,
    time::Duration,
};

use esp_idf_sys::cherry_host::{
    usbh_cdc_acm, usbh_cdc_acm_bulk_in_transfer, usbh_cdc_acm_bulk_out_transfer, usbh_initialize,
    ESP_USBH_BASE,
};
use heapless::String;

type T = String<256>;

/// Usb host input/output
pub struct IO {
    /// USB sender
    pub sender: SyncSender<T>,
    /// USB receiver
    pub receiver: Receiver<T>,
}

static LOCKER: OnceLock<ThreadSafeWrapper> = OnceLock::new();

#[derive(Debug)]
struct ThreadSafeWrapper(*mut usbh_cdc_acm);
unsafe impl Send for ThreadSafeWrapper {}
unsafe impl Sync for ThreadSafeWrapper {}

/// Initialization
/// https://github.com/zleihao/CherryUSB-CDC-MSC/blob/50095e0b63bbdf6f2d5597e71edfa45dd8be6c1d/cdc_msc/middlewares/CherryUSB-1.4.0/class/cdc/usbh_cdc_acm.c#L170
/// https://github.com/cherry-embedded/CherryUSB/blob/f23f5494920b64987350abc87c8154f410c6f5f9/platform/nuttx/usbh_serial.c#L180
/// https://github.com/search?q=repo%3Acherry-embedded%2FCherryUSB%20usbh_cdc_acm_run&type=code
/// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/host/cdc_acm/src/cdc_acm_chost.c#L33
pub unsafe fn cherry_usb_host<'a, 'b>(scope: &'a Scope<'a, 'b>, channel_buffer_size: usize) -> IO {
    let to_usb = sync_channel(channel_buffer_size);
    let from_usb = sync_channel(channel_buffer_size);

    usbh_initialize(0, ESP_USBH_BASE as usize);

    scope.spawn(move || send_usb_data(to_usb.1));
    scope.spawn(move || receive_usb_data(from_usb.0));

    IO {
        sender: to_usb.0,
        receiver: from_usb.1,
    }
}

/// Strong reference to the cdc runner
#[unsafe(no_mangle)]
pub extern "C" fn usbh_cdc_acm_run(cdc_acm_class: *mut usbh_cdc_acm) {
    log::info!("This is from rust");
    LOCKER.set(ThreadSafeWrapper(cdc_acm_class)).unwrap();
}

unsafe fn receive_usb_data(sender: SyncSender<T>) {
    LOCKER.wait();
    loop {
        let mut buffer = [0u8; 10];
        // TODO: Validate with return value
        let _length = usbh_cdc_acm_bulk_in_transfer(
            LOCKER.get().unwrap().0,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            u32::MAX,
        );
        // TODO: Fix stack overflow here
        // let data: Vec<u8, 256> = Vec::from_slice(&buffer).unwrap();
        // sender.send(String::from_utf8(data).unwrap()).unwrap();
        log::info!("The data is {:?}", buffer);
        std::thread::sleep(Duration::from_millis(50));
    }
}

unsafe fn send_usb_data(receiver: Receiver<T>) {
    LOCKER.wait();
    loop {
        let mut data = match receiver.recv() {
            Ok(data) => data,
            Err(e) => {
                log::info!("Error occurred {e}");
                continue;
            }
        };
        // TODO: Validate with return value
        let _length = usbh_cdc_acm_bulk_out_transfer(
            LOCKER.get().unwrap().0,
            data.as_mut_ptr(),
            data.len() as u32,
            u32::MAX,
        );
        log::info!("Data transmitted: {data}");
    }
}
