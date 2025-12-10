// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! BLE configs used for initializations
#![allow(missing_docs)]
use bt_hci::{controller::ExternalController, transport::Transport};
use log::info;
use rand_core::{CryptoRng, RngCore};
use trouble_host::prelude::*;
use trouble_host::{Address, HostResources};

use crate::mk_static;
const MAX_WRITE_SIZE: usize = 512;
const L2CAP_MTU: usize = MAX_WRITE_SIZE + 3 + 4;
const CONNECTIONS_MAX: usize = 1;
/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 8;

/// Number of slots for the BLE connector
const NUM_SLOTS: usize = 40;

/// Simplified reusable controller type
pub type TController<T> = ExternalController<T, NUM_SLOTS>;

/// GATT service name
const GATT_SERVICE_NAME: &str = "Plugin service";

#[gatt_server]
/// GATT Server definition
pub struct Server {
    battery_service: DefaultService,
}

/// Battery service
#[gatt_service(uuid = BluetoothUuid16::new(0x1801))]
struct DefaultService {
    #[characteristic(uuid = "408813df-5dd4-1f87-ec11-cdb001100000", write, read, notify)]
    status: bool,
}

/// Start the BLE device
pub fn ble_config<T, RNG>(
    connector: T,
    random_generator: &'static mut RNG,
) -> (Stack<'static, TController<T>>, Server<'static>)
where
    T: Transport,
    RNG: RngCore + CryptoRng,
{
    // Using a fixed "random" address can be useful for testing. In real scenarios, one would
    // use e.g. the MAC 6 byte array as the address (how to get that varies by the platform).
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    info!("Our address = {}", address);

    let controller: TController<_> = ExternalController::new(connector);

    let resources = &mut *mk_static!(HostResources<CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU>, HostResources::new());

    let stack = trouble_host::new(controller, resources)
        .set_random_address(address)
        .set_random_generator_seed(random_generator);

    info!("Start GATT service");
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: GATT_SERVICE_NAME,
        appearance: &appearance::power_device::GENERIC_POWER_DEVICE,
    }))
    .unwrap();

    (stack, server)
}
