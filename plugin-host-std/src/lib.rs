//! Host device interface. This will use spi as interface that will communicate with the primary but it can also use i2c. USB directly or any other type of interface.
#![deny(missing_docs)]

/// Common util functions
pub mod utils;

pub mod errors;
