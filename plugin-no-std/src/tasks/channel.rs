//! Defines the communication channels

use crate::configs::BUFFER_SIZE;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

/// USB to BLE channel
pub static USB_TO_BLE: Channel<CriticalSectionRawMutex, [u8; BUFFER_SIZE], 100> = Channel::new();

/// BLE to USB channel
pub static BLE_TO_USB: Channel<CriticalSectionRawMutex, [u8; BUFFER_SIZE], 100> = Channel::new();
