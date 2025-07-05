//! USB device runner

use crate::configs::{BUFFER_SIZE, Disconnected};
use crate::tasks::CHANNEL_SIZE;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_usb::{UsbDevice, class::cdc_acm::CdcAcmClass};
use esp_hal::otg_fs::asynch::Driver;
use log::info;
use protocol::plugin::{PluginReceiver, PluginSender};
/// Usb runner
pub async fn run(usb_device: &mut UsbDevice<'static, Driver<'static>>) -> ! {
    loop {
        usb_device.run().await;
    }
}

/// Usb device processor
pub async fn processor(
    mut class: CdcAcmClass<'static, Driver<'static>>,
    receiver: PluginReceiver<'static, CriticalSectionRawMutex, BUFFER_SIZE, CHANNEL_SIZE>,
    sender: PluginSender<'static, CriticalSectionRawMutex, BUFFER_SIZE, CHANNEL_SIZE>,
) -> ! {
    loop {
        class.wait_connection().await;
        esp_println::println!("Connected");
        //TODO: Create a mutex to ensure non-concurrent access sends or receives
        echo(&mut class).await.ok();
        esp_println::println!("Disconnected");
    }
}

async fn echo<'d>(class: &mut CdcAcmClass<'d, Driver<'d>>) -> Result<(), Disconnected> {
    let mut buf = [0; BUFFER_SIZE as usize];
    loop {
        // let n = class.read_packet(&mut buf).await?;
        // USB_TO_BLE.send(buf).await;
        // let decoded_cmd: Option<BulkHostCommand> = ReceivedData::new(buf).decode().ok();
        // if let Some(cmd) = decoded_cmd {
        //     info!("{:?}", cmd)
        // }
        // let info = USB_TO_BLE.receive().await;
        // let data = &info[..n];
        // class.write_packet(data).await?;
    }
}
