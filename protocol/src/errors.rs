//! Contains library errors

/// Result type with the custom error
pub type Result<T> = core::result::Result<T, Error>;

/// Library errors
pub enum Error {
    // Serialization
    /// Unable to serialize to the message pack type
    UnableToSerializeToRMP,
    /// Unable to deserialize from the message pack type
    UnableToDeserializeFromRMP,
    /// Unable to fit the serialized bytes into the send buffer
    SerializationBufferOverflow,
}
