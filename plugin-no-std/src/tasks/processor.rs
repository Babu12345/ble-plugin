// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Processor tasks
use crate::configs::{Server, TController};

use crate::tasks::{BUFFER_SIZE, CHANNEL_SIZE};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_usb::class::cdc_acm::CdcAcmClass;
use esp_hal::otg_fs::asynch::Driver;
use esp_wifi::ble::controller::BleConnector;
use protocol::plugin::{AsyncPluginReceiver, AsyncPluginSender};
use trouble_host::prelude::Peripheral;

#[embassy_executor::task]
/// Integrated USB and BLE processor
pub async fn usb_processor(
    class: CdcAcmClass<'static, Driver<'static>>,
    receiver: AsyncPluginReceiver<'static, CriticalSectionRawMutex, BUFFER_SIZE, CHANNEL_SIZE>,
    sender: AsyncPluginSender<'static, CriticalSectionRawMutex, BUFFER_SIZE, CHANNEL_SIZE>,
) {
    crate::usb_device::processor(class, &receiver, &sender).await
}

#[embassy_executor::task]
/// BLE processor
pub async fn ble_processor(
    server: Server<'static>,
    peripheral: Peripheral<'static, TController<BleConnector<'static>>>,
    receiver: AsyncPluginReceiver<'static, CriticalSectionRawMutex, BUFFER_SIZE, CHANNEL_SIZE>,
    sender: AsyncPluginSender<'static, CriticalSectionRawMutex, BUFFER_SIZE, CHANNEL_SIZE>,
) {
    crate::ble::processor(server, peripheral, &receiver, &sender).await
}
