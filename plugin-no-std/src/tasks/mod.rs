//! Async task initializations

use embassy_usb::UsbDevice;
use esp_hal::otg_fs::asynch::Driver;

#[embassy_executor::task]
/// Runner for the usb device. Must always be running if you want to use the usb peripheral
pub async fn usb_device_runner(mut usb_device: UsbDevice<'static, Driver<'static>>) {
    loop {
        usb_device.run().await
    }
}
