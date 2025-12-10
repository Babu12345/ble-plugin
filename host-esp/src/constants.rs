// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Store constants

/// Maximum delay for the usb host events
pub const USB_LIB_EVENT_MAX_DELAY: u32 = 0xffffffff;
/// Device connection timeout
pub const CONNECTION_TIMEOUT_MS: u32 = 1000;
/// Send timeout
pub const TX_TIMEOUT_MS: u32 = 1000;
/// Send buffer size
pub const TX_BUFFER_SIZE: usize = 512;
/// Receive buffer size
pub const RX_BUFFER_SIZE: usize = 512;
/// USB VID device
pub const USB_DEVICE_VID: u16 = 0x303A;
/// USB PID device
pub const USB_DEVICE_PID: u16 = 0x3001;
/// Default bits/sec ie. baudrate
pub const DEFAULT_DW_DTE_RATE: u32 = 921600;
