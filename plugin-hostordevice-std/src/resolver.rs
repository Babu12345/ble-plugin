// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Contains information about which usb type. Device or host we should select

use esp_idf_svc::hal::gpio::Level;

/// Contains the usb type we should use
#[derive(Debug)]
pub enum USBTypeResolver {
    /// Usb host implementation
    UsbHost,
    /// Usb device implementation
    UsbDevice,
}

impl Into<USBTypeResolver> for Level {
    fn into(self) -> USBTypeResolver {
        match self {
            Level::Low => USBTypeResolver::UsbHost,
            Level::High => USBTypeResolver::UsbDevice,
        }
    }
}
