use std::{
    sync::{
        OnceLock,
        mpsc::{Receiver, SyncSender},
    },
    time::Duration,
};

use esp_idf_sys::cherry_host::{
    usbh_cdc_acm, usbh_cdc_acm_bulk_in_transfer, usbh_cdc_acm_bulk_out_transfer,
};
use heapless::String;

static LOCKER: OnceLock<ThreadSafeWrapper> = OnceLock::new();
pub type T = String<256>;

#[derive(Debug)]
struct ThreadSafeWrapper(*mut usbh_cdc_acm);
unsafe impl Send for ThreadSafeWrapper {}
unsafe impl Sync for ThreadSafeWrapper {}

/// Strong reference to the cdc runner
#[unsafe(no_mangle)]
pub extern "C" fn usbh_cdc_acm_run(cdc_acm_class: *mut usbh_cdc_acm) {
    log::info!("This is from rust");
    LOCKER.set(ThreadSafeWrapper(cdc_acm_class)).unwrap();
}

pub unsafe fn receive_usb_data(sender: SyncSender<T>) {
    LOCKER.wait();
    loop {
        let mut buffer = [0u8; 10];
        // TODO: Validate with return value
        let _length = unsafe {
            usbh_cdc_acm_bulk_in_transfer(
                LOCKER.get().unwrap().0,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                u32::MAX,
            );
        };
        // TODO: Fix stack overflow here
        // let data: Vec<u8, 256> = Vec::from_slice(&buffer).unwrap();
        // sender.send(String::from_utf8(data).unwrap()).unwrap();
        log::info!("The data is {:?}", buffer);
        std::thread::sleep(Duration::from_millis(50));
    }
}

pub unsafe fn send_usb_data(receiver: Receiver<T>) {
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
        let _length = unsafe {
            usbh_cdc_acm_bulk_out_transfer(
                LOCKER.get().unwrap().0,
                data.as_mut_ptr(),
                data.len() as u32,
                u32::MAX,
            );
        };
        log::info!("Data transmitted: {data}");
    }
}
