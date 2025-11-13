//! Contains code for jtag processing of the host

use esp_idf_svc::hal::usb_serial::UsbSerialDriver;

/// Represents the host jtag device
pub struct HostJtagDevice<'d> {
    _driver: UsbSerialDriver<'d>,
}

impl<'d> HostJtagDevice<'d> {
    /// Create new instance of the jtag device
    pub fn new(driver: UsbSerialDriver<'d>) -> Self {
        Self { _driver: driver }
    }
}
