//! Library for the usb device embassy implementation

#![no_std]
#![deny(missing_docs)]
#![cfg(all(target_arch = "xtensa", target_os = "none"))]

pub mod errors;
pub mod processors;
