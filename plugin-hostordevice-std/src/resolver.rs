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
