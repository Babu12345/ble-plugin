//! Contains the basic types to reuse
use heapless::String;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{host::THostIO, MAX_NAME_SIZE};

/// Host command. Configure peripheral
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct HostCommandConfigurePeripheral {
    /// Peripheral name
    pub name: String<MAX_NAME_SIZE>,
    /// Peripheral UUID
    pub uuid: Uuid,
}

/// Host command. Configure peripheral
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct HostCommandConfigureService {}

/// Represents the send type of the data. Was it due to a
/// write event (central -> peripheral), notify event (peripheral -> client),
/// or read attempt (central -> peripheral). Depending on which a response
/// might be expected or sent
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[repr(u8)]
pub enum HostDataSendType {
    /// Notified from the central bluetooth device
    Notify,
    /// Read attempt from the central bluetooth device
    Read,
    /// Written from the central bluetooth device
    Write,
}

/// Host data
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct HostData<'a> {
    /// Source peripheral id that this data is orginating from.
    pub src_id: Uuid,
    /// Actual command type
    pub data: &'a [u8],
    /// Send type of the data
    pub send_type: HostDataSendType,
}

impl<'a> THostIO<'a> for HostCommandConfigurePeripheral {}
impl<'a> THostIO<'a> for HostCommandConfigureService {}
impl<'a> THostIO<'a> for HostData<'a> {}
