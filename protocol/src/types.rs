//! Contains the basic types to reuse
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{MAX_NAME_SIZE, MAX_VEC_SIZE};

/// Acutal host command type
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum HostCommandTypes {
    /// Configure the BLE name and id
    ConfigPeripheral(String<MAX_NAME_SIZE>, Uuid),
    /// Configure the service
    ConfigService,
}

/// Host command data
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct HostCommand {
    /// Command id
    pub uuid: Uuid,
    /// Actual command type
    pub cmd: HostCommandTypes,
}

/// Host command data in bulk
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct BulkHostCommand {
    /// Commands
    pub commands: Vec<HostCommand, MAX_VEC_SIZE>,
}

/// Host data
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct HostData<'a> {
    /// Source peripheral id that this data is orginating from.
    pub src_id: Uuid,
    /// Actual command type
    pub data: &'a [u8],
}

/// Host  data in bulk
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct BulkHostData<'a> {
    /// Data
    #[serde(borrow)]
    pub data: Vec<HostData<'a>, MAX_VEC_SIZE>,
}
