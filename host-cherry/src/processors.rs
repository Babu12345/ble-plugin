use std::{
    sync::{
        RwLock,
        mpsc::{Receiver, SyncSender},
    },
    time::Duration,
};

use esp_idf_sys::cherry_host::{
    usbh_cdc_acm, usbh_cdc_acm_bulk_in_transfer, usbh_cdc_acm_bulk_out_transfer,
};

static CDC_LOCKER: RwLock<Option<ThreadSafeCDCWrapper>> = RwLock::new(None);
pub type T = [u8; 20];

#[derive(Debug)]
struct ThreadSafeCDCWrapper(*mut usbh_cdc_acm);
unsafe impl Send for ThreadSafeCDCWrapper {}
unsafe impl Sync for ThreadSafeCDCWrapper {}

/// Initialization
/// https://github.com/zleihao/CherryUSB-CDC-MSC/blob/50095e0b63bbdf6f2d5597e71edfa45dd8be6c1d/cdc_msc/middlewares/CherryUSB-1.4.0/class/cdc/usbh_cdc_acm.c#L170
/// https://github.com/cherry-embedded/CherryUSB/blob/f23f5494920b64987350abc87c8154f410c6f5f9/platform/nuttx/usbh_serial.c#L180
/// https://github.com/search?q=repo%3Acherry-embedded%2FCherryUSB%20usbh_cdc_acm_run&type=code
/// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/host/cdc_acm/src/cdc_acm_chost.c#L33

/// Strong reference to the cdc runner defined in C
#[unsafe(no_mangle)]
pub extern "C" fn usbh_cdc_acm_run(cdc_acm_class: *mut usbh_cdc_acm) {
    *CDC_LOCKER.write().unwrap() = Some(ThreadSafeCDCWrapper(cdc_acm_class));
}

/// Strong reference to the cdc acm stopper defined in C
#[unsafe(no_mangle)]
#[allow(unused_variables)]
pub extern "C" fn usbh_cdc_acm_stop(cdc_acm_class: *mut usbh_cdc_acm) {
    *CDC_LOCKER.write().unwrap() = None;
}

pub unsafe fn receive_usb_data(sender: SyncSender<T>) {
    let mut buffer = [0; size_of::<T>()];
    loop {
        let cdc_acm_class: *mut usbh_cdc_acm = match CDC_LOCKER.read().unwrap().as_ref() {
            Some(wrapper) => wrapper,
            None => {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
        }
        .0;
        match unsafe {
            usbh_cdc_acm_bulk_in_transfer(
                cdc_acm_class,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                u32::MAX,
            )
        } {
            x if x < 0 => {
                log::error!("Unable to receive the data");
                continue;
            }
            _ => {}
        };
        log::info!("The data is {:?}", String::from_utf8(Vec::from(&buffer)));

        match sender.try_send(buffer) {
            Ok(_) => {}
            Err(std::sync::mpsc::TrySendError::Full(_)) => {
                log::warn!(
                    "Receive buffer is full. You must ingest in order to receive additional information."
                );
            }
            Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                log::error!("Disconnect error");
            }
        }
    }
}

pub unsafe fn send_usb_data(receiver: Receiver<T>) {
    loop {
        let cdc_acm_class: *mut usbh_cdc_acm = match CDC_LOCKER.read().unwrap().as_ref() {
            Some(wrapper) => wrapper,
            None => {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
        }
        .0;

        let mut data = match receiver.recv() {
            Ok(data) => data,
            Err(e) => {
                log::error!("Error occurred {e}");
                continue;
            }
        };

        match unsafe {
            usbh_cdc_acm_bulk_out_transfer(
                cdc_acm_class,
                data.as_mut_ptr(),
                data.len() as u32,
                u32::MAX,
            )
        } {
            x if x < 0 => {
                log::error!("Unable to send the data {:?}", x);
                continue;
            }
            _ => {}
        };

        log::info!("Data transmitted: {:?}", data);
    }
}
