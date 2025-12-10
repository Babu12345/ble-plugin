// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

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
    UnableToDeserializeFromBincode(&'static str),
    // Protocol Buffers
    /// Unable to serialize to protocol buffers
    UnableToSerializeToProtobuf,
    /// Unable to deserialize from protocol buffers
    UnableToDeserializeFromProtobuf,
    // Quick Protocol Buffers
    /// Unable to serialize from quick protocol buffers
    UnableToSerializeToQuickProtobuf,
    /// Unable to deserialize from quick protocol buffers
    UnableToDeserializeFromQuickProtobuf,
    // Transfers
    /// Send error
    ReceiveError,
    /// Send error
    SendError,
    /// Invalid data length
    InvalidDataLength {
        /// Expected length
        expected: usize,
        /// Actual length
        got: usize,
    },
    /// Invalid data length for header
    InvalidDataLengthForHeader,
    /// Invalid magic number
    InvalidMagicNumber,
    /// Unknown message type
    InvalidMessageType,
    /// Payload size exceeds allowable limit
    InvalidPayloadSize,
}
