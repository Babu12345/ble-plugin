#![no_std]
// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.
#![no_main]
#![feature(never_type)]

use device_embassy::processors::CdcAcmDeviceHost;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_usb::class::cdc_acm::State;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, otg_fs::Usb, timer::systimer::SystemTimer};
use host_no_std::configs::initalize_logger;
use protocol::DEFAULT_PACKET_SIZE;
use protocol::devices::host::AsyncHostProcessor;
use protocol::host::AsyncHostSender;
use protocol::protocol::{
    HostCommandConfigurePeripheral, HostCommandConfigureProfile, HostCommandConfigureService,
    HostCommandStartAdvertisement,
};

// BLE no-std example: https://github.com/embassy-rs/trouble/blob/main/examples/apps/src/ble_bas_peripheral_sec.rs
// USB device example: https://github.com/esp-rs/esp-hal/blob/main/examples/src/bin/embassy_usb_serial.rs
#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    esp_alloc::heap_allocator!(size: 72 * 1024);

    initalize_logger().ok();

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);

    let mut ep_out_buffer = [0; 1024];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut state = State::new();

    let connection_signal = Signal::new();

    let device_host = CdcAcmDeviceHost::<'_, 20, DEFAULT_PACKET_SIZE, NoopRawMutex>::new(
        usb,
        &mut ep_out_buffer,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut control_buf,
        &mut state,
        true,
    )
    .add_connection_signal(&connection_signal);

    let to = Channel::<NoopRawMutex, _, 20>::new();
    let from = Channel::<NoopRawMutex, _, 20>::new();

    let (processor_fn, sender, _receiver) = device_host
        .processors(
            (to.sender(), to.receiver()),
            (from.sender(), from.receiver()),
        )
        .unwrap();

    let check_conn_fn = async {
        loop {
            match connection_signal.wait().await {
                true => {
                    log::info!("Connected");
                    test_queue_setup_commands(&sender).await.ok();
                }
                false => log::info!("Disonnected"),
            }
        }
    };

    join(check_conn_fn, processor_fn).await;
}

/// Configures the plugin
pub async fn test_queue_setup_commands<const CH_SIZE: usize>(
    sender: &AsyncHostSender<'_, NoopRawMutex, DEFAULT_PACKET_SIZE, CH_SIZE>,
) -> Result<(), ()> {
    extern crate alloc;
    use alloc::{string::String, vec::Vec};
    sender
        .borrow_send_async(&HostCommandConfigurePeripheral {
            name: String::from_utf8(Vec::from(b"Bab's device")).unwrap(),
            addr: Vec::from(&[1, 1, 1, 1, 1, 1]),
        })
        .await
        .unwrap();

    sender
        .borrow_send_async(&HostCommandConfigureService { uuid: 0x1800 as _ })
        .await
        .unwrap();

    sender
        .borrow_send_async(&HostCommandConfigureProfile {
            profile: protocol::protocol::BleProfile::Custom,
            ..Default::default()
        })
        .await
        .unwrap();

    sender
        .borrow_send_async(&HostCommandStartAdvertisement {
            allow_multi_connect: true,
        })
        .await
        .unwrap();
    Ok(())
}
