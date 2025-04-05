//! Async task initializations

use embassy_usb::{UsbDevice, class::cdc_acm::CdcAcmClass};
use esp_hal::otg_fs::asynch::Driver;

use crate::usb_device::processor;

#[embassy_executor::task]
/// Runner for the usb device. Must always be running if you want to use the usb peripheral
pub async fn usb_device_runner(mut usb_device: UsbDevice<'static, Driver<'static>>) {
    crate::usb_device::run(&mut usb_device).await;
}

#[embassy_executor::task]
/// Processor task for the usb device
pub async fn usb_device_processor(class: CdcAcmClass<'static, Driver<'static>>) {
    processor(class).await;
}
