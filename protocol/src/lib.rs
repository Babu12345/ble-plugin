//! Defines the host and plug-in protocols that they must adhere to in order to
//! transfer data and commands between each other.
//!
//! Defines Input, Output, and a combination IO trait for each module for facilitating data transfer

#![deny(missing_docs)]

pub mod errors;
pub mod host;
pub mod plugin;
pub mod primary;

const MAX_NAME_SIZE: usize = 30;
