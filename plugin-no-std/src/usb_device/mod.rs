//! USB device runner

use crate::configs::{BUFFER_SIZE, Disconnected};
use crate::tasks::CHANNEL_SIZE;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_usb::{UsbDevice, class::cdc_acm::CdcAcmClass};
use esp_hal::otg_fs::asynch::Driver;
use log::info;
use protocol::plugin::PluginReceivedData;
use protocol::plugin::{AsyncPluginReceiver, AsyncPluginSender};
use protocol::types::HostCommandConfigurePeripheral;
/// Usb runner
pub async fn run(usb_device: &mut UsbDevice<'static, Driver<'static>>) -> ! {
    loop {
        usb_device.run().await;
    }
}

/// Usb device processor
pub async fn processor(
    mut class: CdcAcmClass<'static, Driver<'static>>,
    _receiver: &AsyncPluginReceiver<'_, CriticalSectionRawMutex, BUFFER_SIZE, CHANNEL_SIZE>,
    sender: &AsyncPluginSender<'_, CriticalSectionRawMutex, BUFFER_SIZE, CHANNEL_SIZE>,
) -> ! {
    loop {
        class.wait_connection().await;
        esp_println::println!("Connected");
        //TODO: Create a mutex to ensure non-concurrent access sends or receives
        echo(&mut class, sender).await.ok();
        esp_println::println!("Disconnected");
    }
}

async fn echo<'d>(
    class: &mut CdcAcmClass<'d, Driver<'d>>,
    sender: &AsyncPluginSender<'_, CriticalSectionRawMutex, BUFFER_SIZE, CHANNEL_SIZE>,
) -> Result<(), Disconnected> {
    let mut buf = [0; BUFFER_SIZE as usize];
    loop {
        let _n = class.read_packet(&mut buf).await?;
        let decoded_cmd: Option<HostCommandConfigurePeripheral> =
            PluginReceivedData::new(buf).decode().ok();
        if let Some(cmd) = decoded_cmd {
            match sender.borrow_send_async(&cmd).await.ok() {
                Some(_) => {}
                None => log::error!("Unable to send the data to the BLE side"),
            }
            info!("{:?}", &cmd);
        }
        // USB_TO_BLE.send(buf).await;
        // let info = USB_TO_BLE.receive().await;
        // let data = &info[..n];
        // class.write_packet(data).await?;
    }
}
