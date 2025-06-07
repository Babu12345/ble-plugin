//! Processor tasks
use crate::configs::{BUFFER_SIZE, Server, TController};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_usb::class::cdc_acm::CdcAcmClass;
use esp_hal::otg_fs::asynch::Driver;
use esp_wifi::ble::controller::BleConnector;
use trouble_host::prelude::Peripheral;

/// Channel for communicating between the usb and the ble processors
pub static CHANNEL: Channel<CriticalSectionRawMutex, [u8; BUFFER_SIZE], 100> = Channel::new();

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
