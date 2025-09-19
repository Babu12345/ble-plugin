//! Contains information about which usb type. Device or host we should select

/// Contains the usb type we should use
pub enum USBTypeResolver {
    /// Usb host implementation
    UsbHost,
    /// Usb device implementation
    UsbDevice,
}
