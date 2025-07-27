//! Defines the host and plug-in protocols that they must adhere to in order to
//! transfer data and commands between each other. Note when this library references Host it refers to
//! the the device that is accessing the capabilitiese of the plugin. It does not refer to the USB host protocol.
//! This is because it's technically correct for the plugin to implement the USB host protocol for "hosts" that implement
//! the USB device protocol only

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

pub mod errors;
pub mod host;
mod io;
pub mod plugin;
pub mod types;
use io::*;

const MAX_NAME_SIZE: usize = 30;
/// Represents the transfer size
pub const DEFAULT_TRANSFER_SIZE: usize = 256;

#[cfg(test)]
mod tests {
    use crate::DEFAULT_TRANSFER_SIZE;

    #[test]
    fn test_max_transfer_size() {
        assert!(
            DEFAULT_TRANSFER_SIZE <= 512,
            "The max transfer size is 512 for high speed usb"
        )
    }
}
