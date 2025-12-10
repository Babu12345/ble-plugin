// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Host device interface. This will use spi as interface that will communicate with the primary but it can also use i2c. USB directly or any other type of interface.
#![deny(missing_docs)]

/// Common util functions
pub mod utils;

pub mod errors;
