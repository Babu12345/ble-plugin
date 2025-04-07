use embassy_usb::UsbDevice;
use esp_hal::otg_fs::asynch::Driver;
use esp_wifi::ble::controller::BleConnector;
use trouble_host::prelude::Runner;

use crate::configs::TController;

#[embassy_executor::task]
/// Runner for the usb device. Must always be running if you want to use the usb peripheral
pub async fn usb_device_runner(mut usb_device: UsbDevice<'static, Driver<'static>>) {
    crate::usb_device::run(&mut usb_device).await;
}

#[embassy_executor::task]
/// Runner for the usb device. Must always be running if you want to use the usb peripheral
pub async fn ble_runner(runner: Runner<'static, TController<BleConnector<'static>>>) {
    crate::ble::run(runner).await;
}
