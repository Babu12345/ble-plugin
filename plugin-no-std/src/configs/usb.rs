//! Configs to initialize the usb device

use embassy_usb::{
    Builder,
    class::cdc_acm::{CdcAcmClass, State},
    driver::EndpointError,
};
use esp_hal::otg_fs::{
    Usb,
    asynch::{Config, Driver},
};
use log::info;

/// Control buffer size
pub const BUFFER_SIZE: usize = 64;
const DESCRIPTOR_BUFFER_SIZE: usize = 256;

/// Function to start the usb device. Returns the CDC class handler and the device
pub fn usb_device_config<'class>(
    usb: Usb<'class>,
    cdc_state: &'class mut State<'class>,
    config_descriptor: &'class mut [u8; DESCRIPTOR_BUFFER_SIZE],
    bos_descriptor: &'class mut [u8; DESCRIPTOR_BUFFER_SIZE],
    control_buffer: &'class mut [u8; BUFFER_SIZE as usize],
    ep_out_buffer: &'class mut [u8; 1024],
) -> (
    CdcAcmClass<'class, Driver<'class>>,
    embassy_usb::UsbDevice<'class, Driver<'class>>,
) {
    // Create the driver, from the HAL.
    let config = Config::default();

    let driver = Driver::new(usb, ep_out_buffer, config);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0x303A, 0x3001);
    config.manufacturer = Some("Espressif");
    config.product = Some("USB-serial example");
    config.serial_number = Some("2101");

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    config.max_power = 100;
    config.max_packet_size_0 = BUFFER_SIZE as u8;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.

    let mut builder: Builder<'class, Driver<'class>> = Builder::new(
        driver,
        config,
        config_descriptor,
        bos_descriptor,
        &mut [], // no msos descriptors
        control_buffer,
    );

    let class = CdcAcmClass::new(&mut builder, cdc_state, BUFFER_SIZE as u16);
    info!("Device initialized!");
    (class, builder.build())
}

/// Type to map errors to the USB disconnect type if applicable
pub struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}
