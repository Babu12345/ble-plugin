//! USB device runner

use embassy_usb::UsbDevice;
use esp_hal::otg_fs::asynch::Driver;

/// Usb runner
pub async fn run(usb_device: &mut UsbDevice<'static, Driver<'static>>) -> ! {
    loop {
        usb_device.run().await;
    }
}
