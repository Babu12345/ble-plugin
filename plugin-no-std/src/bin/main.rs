#![no_std]
#![no_main]
#![feature(never_type)]

use embassy_executor::Spawner;
use embassy_usb::{
    class::cdc_acm::{CdcAcmClass, State},
    driver::EndpointError,
};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    otg_fs::{Usb, asynch::Driver},
    rng::Rng,
    timer::systimer::SystemTimer,
};
use esp_wifi::{EspWifiController, ble::controller::BleConnector};
use plugin_no_std::{
    configs::{BUFFER_SIZE, StartUsbDeviceInput, initalize_logger, start_usb_device},
    mk_static,
    tasks::usb_device_runner,
};

use esp_hal_embassy::main;

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::heap_allocator!(size: 72 * 1024);

    initalize_logger().ok();

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    let rng = Rng::new(peripherals.RNG);
    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let init = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timer1.timer0, rng, peripherals.RADIO_CLK,).unwrap()
    );
    let _connector = BleConnector::new(&init, peripherals.BT);

    let usb: Usb<'static> = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);

    let (mut class, device) = start_usb_device(StartUsbDeviceInput {
        usb,
        cdc_state: &mut *mk_static!(State<'static>, State::new()),
        config_descriptor: &mut *mk_static!([u8; 256], [0; 256]),
        bos_descriptor: &mut *mk_static!([u8; 256], [0; 256]),
        control_buffer: &mut *mk_static!([u8; BUFFER_SIZE], [0; BUFFER_SIZE]),
        ep_out_buffer: &mut *mk_static!([u8; 1024], [0; 1024]),
    });

    spawner.must_spawn(usb_device_runner(device));

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

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}
