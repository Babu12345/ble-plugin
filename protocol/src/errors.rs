//! Contains library errors
use serde::Deserialize;

/// Result type with the custom error
pub type Result<T> = core::result::Result<T, Error>;

/// Library errors
#[derive(Deserialize, Debug)]
pub enum Error {
    // Serialization
    // RMP
    /// Unable to serialize to the message pack type
    UnableToSerializeToRMP,
    /// Unable to deserialize from the message pack type
    UnableToDeserializeFromRMP,
    /// Unable to fit the serialized bytes into the send buffer
    SerializationBufferOverflow,
    // Bincode
    /// Unable to serialize to bincode
    UnableToSerializeToBincode,
    /// Unable to deserialize from bincode
    UnableToDeserializeFromBincode,
    // Transfers
    /// Send error
    ReceiveError,
    /// Send error
    SendError,
}
