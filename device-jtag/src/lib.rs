// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Library for the usb host serial jtag implementation

#![cfg(all(target_arch = "xtensa", target_os = "espidf"))]

pub mod errors;
#[deny(missing_docs)]
// https://github.com/search?q=esp_idf_svc%3A%3Ahal%3A%3Ausb_serial&type=code
mod processors;

pub use processors::*;
