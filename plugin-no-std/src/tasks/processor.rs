//! Processor tasks
use crate::configs::{Server, TController};
use embassy_usb::class::cdc_acm::CdcAcmClass;
use esp_hal::otg_fs::asynch::Driver;
use esp_wifi::ble::controller::BleConnector;
use trouble_host::prelude::Peripheral;

#[embassy_executor::task]
/// Integrated processor
pub async fn usb_and_ble_processor(
    class: CdcAcmClass<'static, Driver<'static>>,
    server: Server<'static>,
    peripheral: Peripheral<'static, TController<BleConnector<'static>>>,
) {
    crate::usb_and_ble::processor(class, server, peripheral).await
}
