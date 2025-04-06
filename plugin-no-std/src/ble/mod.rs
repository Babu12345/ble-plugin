//! BLE processor and runner

use crate::configs::{Server, TController};
use esp_wifi::ble::controller::BleConnector;
use log::{error, info};
use trouble_host::{
    BleHostError, Controller,
    gatt::GattConnection,
    prelude::{
        AdStructure, Advertisement, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE, Peripheral,
        Runner,
    },
};

const BLE_ADVERTISEMENT_NAME: &str = "Plugin";

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

/// Run the Bluetooth peripheral
pub async fn processor(
    server: Server<'_>,
    mut peripheral: Peripheral<'static, TController<BleConnector<'static>>>,
) {
    loop {
        match advertise(BLE_ADVERTISEMENT_NAME, &mut peripheral, &server).await {
            Ok(_) => todo!(),
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
            AdStructure::ServiceUuids16(&[[0x0f, 0x18]]),
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
