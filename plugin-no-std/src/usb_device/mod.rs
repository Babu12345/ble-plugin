//! USB device runner

use crate::{
    configs::{BUFFER_SIZE, Disconnected},
    tasks::USB_TO_BLE,
};
use embassy_usb::{UsbDevice, class::cdc_acm::CdcAcmClass};
use esp_hal::otg_fs::asynch::Driver;

/// Usb runner
pub async fn run(usb_device: &mut UsbDevice<'static, Driver<'static>>) -> ! {
    loop {
        usb_device.run().await;
    }
}

/// Usb device processor
pub async fn processor(mut class: CdcAcmClass<'static, Driver<'static>>) -> ! {
    loop {
        class.wait_connection().await;
        esp_println::println!("Connected");
        echo(&mut class).await.ok();
        esp_println::println!("Disconnected");
    }
}

async fn echo<'d>(class: &mut CdcAcmClass<'d, Driver<'d>>) -> Result<(), Disconnected> {
    let mut buf = [0; BUFFER_SIZE as usize];
    loop {
        let n = class.read_packet(&mut buf).await?;
        USB_TO_BLE.send(buf).await;
        // Echo back in upper case
        for c in buf[0..n].iter_mut() {
            if 0x61 <= *c && *c <= 0x7a {
                *c &= !0x20;
            }
        }
        let info = USB_TO_BLE.receive().await;
        let data = &info[..n];
        class.write_packet(data).await?;
    }
}
