//! Processor crate

use core::{future::Future, marker::PhantomData};

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use esp_hal::otg_fs::asynch::Driver;
use lib_utils::mk_static;
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

impl<'a, const CH_SIZE: usize, const BUFFER_SIZE: usize>
    AsyncHostProcessor<CH_SIZE, BUFFER_SIZE, NoopRawMutex, crate::errors::Error>
    for CdcAcmDeviceHost<'a, CH_SIZE, BUFFER_SIZE>
{
    async fn processors<'ch>(
        self,
        channel_buffer_size: usize,
    ) -> Result<(
        protocol::host::AsyncHostSender<'ch, NoopRawMutex, BUFFER_SIZE, CH_SIZE>,
        protocol::host::AsyncHostReceiver<'ch, NoopRawMutex, BUFFER_SIZE, CH_SIZE>,
    )> {
        todo!()
    }
}
