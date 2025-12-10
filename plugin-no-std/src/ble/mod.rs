// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! BLE runner

use crate::configs::{Server, TController};
use crate::tasks::{BUFFER_SIZE, CHANNEL_SIZE};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp_wifi::ble::controller::BleConnector;
use log::{error, info};
use protocol::plugin::{AsyncPluginReceiver, AsyncPluginSender};
use trouble_host::{
    BleHostError, Controller,
    gatt::GattConnection,
    prelude::{
        AdStructure, Advertisement, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE, Peripheral,
        Runner,
    },
};

const BLE_ADVERTISEMENT_NAME: &str = "Plugin";
const BLE_SERVICE_UUID: [[u8; 2]; 1] = [[0x0f, 0x18]];
/// Run the Bluetooth peripheral
pub async fn run<'runner, C>(mut runner: Runner<'runner, C>)
where
    C: Controller,
{
    loop {
        runner
            .run()
            .await
            .inspect_err(|_| log::error!("BLE runner error occurred"))
            .ok();
    }
}

/// Run the integrated processor
pub async fn processor(
    server: Server<'_>,
    mut peripheral: Peripheral<'static, TController<BleConnector<'static>>>,
    _receiver: &AsyncPluginReceiver<'static, CriticalSectionRawMutex, BUFFER_SIZE, CHANNEL_SIZE>,
    _sender: &AsyncPluginSender<'static, CriticalSectionRawMutex, BUFFER_SIZE, CHANNEL_SIZE>,
) {
    loop {
        match advertise(BLE_ADVERTISEMENT_NAME, &mut peripheral, &server).await {
            Ok(_conn) => todo!(),
            Err(_) => error!("Error connecting during BLE advertisement"),
        }
    }
}

/// Create an advertiser to use to connect to a BLE Central, and wait for it to connect.
async fn advertise<'a, 'b, C: Controller>(
    name: &'a str,
    peripheral: &mut Peripheral<'a, C>,
    server: &'b Server<'_>,
) -> Result<GattConnection<'a, 'b>, BleHostError<C::Error>> {
    let mut advertiser_data = [0; 31];
    AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&BLE_SERVICE_UUID),
            AdStructure::CompleteLocalName(name.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;
    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..],
                scan_data: &[],
            },
        )
        .await?;
    info!("[adv] advertising");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("[adv] connection established");
    Ok(conn)
}
