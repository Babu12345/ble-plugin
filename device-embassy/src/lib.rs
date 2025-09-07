//! Library for the usb device embassy implementation

#![no_std]
#![deny(missing_docs)]
#![cfg(all(
    any(target_arch = "xtensa", target_arch = "riscv32"),
    target_os = "none"
))]

pub mod errors;
pub mod processors;
