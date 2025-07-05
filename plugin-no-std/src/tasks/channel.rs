//! Defines the communication channels

use crate::configs::BUFFER_SIZE;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

type T = [u8; BUFFER_SIZE];
const CHANNEL_SIZE: usize = 100;

/// USB to BLE channel
pub static USB_TO_BLE: Channel<CriticalSectionRawMutex, T, CHANNEL_SIZE> = Channel::new();

/// BLE to USB channel
pub static BLE_TO_USB: Channel<CriticalSectionRawMutex, T, CHANNEL_SIZE> = Channel::new();
