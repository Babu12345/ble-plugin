//! Contains the basic types to reuse
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{MAX_NAME_SIZE, MAX_VEC_SIZE};

/// Acutal host command type
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum HostCommandTypes {
    /// Configure the BLE name
    ConfigPeripheral(String<MAX_NAME_SIZE>, Uuid),
    /// Configure the service
    ConfigService,
}

/// Host command data
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HostCommand {
    /// Unique command id
    pub uuid: Uuid,
    /// Actual command type
    pub cmd: HostCommandTypes,
}

/// Host command data in bulk
#[derive(Debug, Deserialize, Serialize)]
pub struct BulkHostCommand {
    /// Commands
    pub commands: Vec<HostCommand, MAX_VEC_SIZE>,
}

/// Host data
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HostData<'a> {
    /// Actual command type
    pub data: &'a [u8],
}

/// Host  data in bulk
#[derive(Debug, Deserialize, Serialize)]
pub struct BulkHostData<'a> {
    /// Data
    #[serde(borrow)]
    data: Vec<HostData<'a>, MAX_VEC_SIZE>,
}
