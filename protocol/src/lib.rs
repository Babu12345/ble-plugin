//! Defines the host and plug-in protocols that they must adhere to in order to
//! transfer data and commands between each other.
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

pub mod errors;
pub mod host;
pub mod plugin;
pub mod primary;
pub mod types;

const MAX_NAME_SIZE: usize = 30;
const MAX_TRANSFER_SIZE: usize = 512;
const MAX_VEC_SIZE: usize = 2;
