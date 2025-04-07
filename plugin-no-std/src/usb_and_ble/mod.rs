//! Integrated USB and BLE processor

use crate::configs::{BUFFER_SIZE, Disconnected};
use crate::configs::{Server, TController};
use embassy_usb::class::cdc_acm::CdcAcmClass;
use esp_hal::otg_fs::asynch::Driver;
use esp_wifi::ble::controller::BleConnector;
use log::{error, info};
use trouble_host::{
    BleHostError, Controller,
    gatt::GattConnection,
    prelude::{
        AdStructure, Advertisement, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE, Peripheral,
    },
};
const BLE_ADVERTISEMENT_NAME: &str = "Plugin";

/// Run the integrated processor
pub async fn processor(
    _class: CdcAcmClass<'static, Driver<'static>>,
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

/// Usb device processor
pub async fn usb_processor(mut class: CdcAcmClass<'static, Driver<'static>>) -> ! {
    // Echo function
    loop {
        class.wait_connection().await;
        esp_println::println!("Connected");
        echo(&mut class).await.ok();
        esp_println::println!("Disconnected");
    }
}

async fn echo<'d>(class: &mut CdcAcmClass<'d, Driver<'d>>) -> Result<(), Disconnected> {
    let mut buf = [0; BUFFER_SIZE as usize];
    loop {
        let n = class.read_packet(&mut buf).await?;
        // Echo back in upper case
        for c in buf[0..n].iter_mut() {
            if 0x61 <= *c && *c <= 0x7a {
                *c &= !0x20;
            }
        }
        let data = &buf[..n];
        class.write_packet(data).await?;
    }
}
