//! Processor tasks
use crate::configs::{Server, TController};

use embassy_usb::class::cdc_acm::CdcAcmClass;
use esp_hal::otg_fs::asynch::Driver;
use esp_wifi::ble::controller::BleConnector;
use trouble_host::prelude::Peripheral;

#[embassy_executor::task]
/// Integrated USB and BLE processor
pub async fn usb_processor(class: CdcAcmClass<'static, Driver<'static>>) {
    crate::usb_device::processor(class).await
}

#[embassy_executor::task]
/// BLE processor
pub async fn ble_processor(
    server: Server<'static>,
    peripheral: Peripheral<'static, TController<BleConnector<'static>>>,
) {
    crate::ble::processor(server, peripheral).await
}
