// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

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
