//! Processor crate

use core::{future::Future, marker::PhantomData};

use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::Channel,
};
use esp_hal::otg_fs::asynch::Driver;
use protocol::{
    devices::host::AsyncHostProcessor,
    host::{AsyncHostReceiver, AsyncHostSender},
};

use crate::errors::{Error, Result};
use embassy_futures::join::join3;
use embassy_usb::{
    class::cdc_acm::{CdcAcmClass, State},
    driver::EndpointError,
    Builder, UsbDevice,
};
use esp_hal::otg_fs::{asynch::Config, Usb};

const BUFFER_SIZE: u16 = 64;

/// Cdc acm device that implements a AsyncHostProcessor. Pre init.
pub struct CdcAcmDeviceHost<'a, const CH_SIZE: usize, const BUFFER_SIZE: usize> {
    usb_device: UsbDevice<'a, Driver<'a>>,
    class: CdcAcmClass<'a, Driver<'a>>,
}

impl<'a, const CH_SIZE: usize, const BUFFER_SIZE: usize>
    CdcAcmDeviceHost<'a, CH_SIZE, BUFFER_SIZE>
{
    /// Initializes and creates a new instance of the device
    pub fn new(
        usb: Usb<'a>,
        ep_out_buffer: &'a mut [u8; 1024],
        config_descriptor: &'a mut [u8; 256],
        bos_descriptor: &'a mut [u8; 256],
        control_buf: &'a mut [u8; 64],
        state: &'a mut State<'a>,
    ) -> Self {
        let config = Config::default();
        let driver = Driver::new(usb, ep_out_buffer, config);

        // Create embassy-usb Config
        let mut config = embassy_usb::Config::new(0x303A, 0x3001);
        config.manufacturer = Some("Espressif");
        config.product = Some("USB-serial example");
        config.serial_number = Some("12345678");

        // Required for windows compatibility.
        // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
        config.device_class = 0xEF;
        config.device_sub_class = 0x02;
        config.device_protocol = 0x01;
        config.composite_with_iads = true;

        // Create embassy-usb DeviceBuilder using the driver and config.
        // It needs some buffers for building the descriptors.

        let mut builder = Builder::new(
            driver,
            config,
            config_descriptor,
            bos_descriptor,
            &mut [], // no msos descriptors
            control_buf,
        );

        // Create classes on the builder.
        let class = CdcAcmClass::new(&mut builder, state, 64);

        // Build the builder.
        let usb_device = builder.build();

        Self { usb_device, class }
    }
}

// impl<'a, const CH_SIZE: usize, const BUFFER_SIZE: usize>
//     AsyncHostProcessor<CH_SIZE, BUFFER_SIZE, CriticalSectionRawMutex, crate::errors::Error>
//     for CdcAcmDeviceHost<'a, CH_SIZE, BUFFER_SIZE>
// {
//     async fn processors<'ch>(
//         mut self,
//     ) -> Result<(
//         protocol::host::AsyncHostSender<'ch, CriticalSectionRawMutex, BUFFER_SIZE, CH_SIZE>,
//         protocol::host::AsyncHostReceiver<'ch, CriticalSectionRawMutex, BUFFER_SIZE, CH_SIZE>,
//     )> {
//         let sender =
//             Channel::<CriticalSectionRawMutex, [u8; BUFFER_SIZE], channel_buffer_size>::new();
//         let usb_future = self.usb_device.run();

//         let sender = async {
//             loop {
//                 self.class.wait_connection().await;
//                 let mut buf = [0; 64];
//                 loop {

//                     // match self.class.read_packet(&mut buf).await {
//                     //     Ok(_) => {}
//                     //     Err(_) => {}
//                     // }
//                 }
//             }
//         };

//         join3(usb_future, sender, async {}).await;

//         todo!()
//     }
// }
