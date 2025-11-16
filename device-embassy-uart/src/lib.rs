//! Library for the usb device embassy uart implementation

#![no_std]
#![deny(missing_docs)]
#![cfg(all(
    any(target_arch = "xtensa", target_arch = "riscv32"),
    target_os = "none"
))]
// Example: https://github.com/esp-rs/esp-hal/blob/main/examples/async/embassy_serial/src/main.rs

mod processors;

pub mod errors;
pub use processors::*;
