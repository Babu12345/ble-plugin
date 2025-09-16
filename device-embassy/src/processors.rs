//! Processor crate

use core::future::Future;

use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    channel::{Receiver, Sender},
};
use esp_hal::otg_fs::asynch::Driver;
use protocol::{
    devices::host::AsyncHostProcessor,
    host::{AsyncHostReceiver, AsyncHostSender},
};

use crate::errors::Result;
use embassy_futures::join::join3;
use embassy_usb::{
    class::cdc_acm::{CdcAcmClass, State},
    driver::EndpointError,
    Builder, UsbDevice,
};
use esp_hal::otg_fs::{asynch::Config, Usb};

const BUFFER_SIZE: usize = 64;

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
        let class = CdcAcmClass::new(&mut builder, state, BUFFER_SIZE as u16);

        // Build the builder.
        let usb_device = builder.build();

        Self { usb_device, class }
    }
}

impl<'a, const CH_SIZE: usize, M: RawMutex>
    AsyncHostProcessor<CH_SIZE, BUFFER_SIZE, M, crate::errors::Error>
    for CdcAcmDeviceHost<'a, CH_SIZE, BUFFER_SIZE>
{
    type T<'ch>
        = (
        Sender<'ch, M, [u8; BUFFER_SIZE], CH_SIZE>,
        Receiver<'ch, M, [u8; BUFFER_SIZE], CH_SIZE>,
    )
    where
        M: 'ch;

    fn processors<'ch>(
        mut self,
        to: Self::T<'ch>,
        from: Self::T<'ch>,
    ) -> Result<(
        impl Future<Output = ()>,
        AsyncHostSender<'ch, M, BUFFER_SIZE, CH_SIZE>,
        AsyncHostReceiver<'ch, M, BUFFER_SIZE, CH_SIZE>,
    )> {
        let (mut sender, mut receiver) = self.class.split();

        let processor_runner = async move {
            let usb_runner = self.usb_device.run();

            let to_usb_fn = async {
                'conn: loop {
                    sender.wait_connection().await;
                    'process: loop {
                        let data = to.1.receive().await;
                        match sender.write_packet(&data).await {
                            Ok(_) => {}
                            Err(e) => match e {
                                EndpointError::BufferOverflow => continue 'process,
                                EndpointError::Disabled => {
                                    log::warn!("USB Disconnected. Retrying");
                                    continue 'conn;
                                }
                            },
                        }
                    }
                }
            };

            let from_usb_fn = async {
                'conn: loop {
                    receiver.wait_connection().await;
                    let mut buf = [0; BUFFER_SIZE];
                    'process: loop {
                        match receiver.read_packet(&mut buf).await {
                            Ok(_) => {}
                            Err(e) => match e {
                                EndpointError::BufferOverflow => continue 'process,
                                EndpointError::Disabled => {
                                    log::warn!("USB Disconnected. Retrying");
                                    continue 'conn;
                                }
                            },
                        }

                        from.0.send(buf).await;
                        buf = [0; BUFFER_SIZE];
                    }
                }
            };

            join3(usb_runner, to_usb_fn, from_usb_fn).await;
        };

        Ok((
            processor_runner,
            AsyncHostSender::new(to.0),
            AsyncHostReceiver::new(from.1),
        ))
    }
}
