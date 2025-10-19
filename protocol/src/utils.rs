//! Common utils for the protocol

use strum::IntoEnumIterator;

use crate::{protocol::MessageTypeId, MESSAGE_HEADER_SIZE, MESSAGE_MAGIC, MESSAGE_MAGIC_BYTES};

/// Convert a slice to an array of fixed size
pub fn slice_to_array<const N: usize>(slice: &[u8]) -> crate::errors::Result<[u8; N]> {
    <[u8; N]>::try_from(slice).map_err(|_| {
        return crate::errors::Error::InvalidDataLength {
            expected: N,
            got: slice.len(),
        };
    })
}

/// Extracting the message type id from raw bytes
#[inline(always)]
pub fn extract_message_type_id(data: &[u8]) -> crate::errors::Result<MessageTypeId> {
    // Check if we have enough bytes for a valid header
    if data.len() < MESSAGE_HEADER_SIZE {
        return Err(crate::errors::Error::InvalidDataLengthForHeader);
    }

    // Verify magic number
    let magic = data[0];
    if magic != MESSAGE_MAGIC {
        return Err(crate::errors::Error::InvalidMagicNumber);
    }

    // Extract message type ID
    let type_id = u16::from_le_bytes([data[MESSAGE_MAGIC_BYTES], data[MESSAGE_MAGIC_BYTES + 1]]);

    let message_type_id =
        MessageTypeId::iter().find(|message_type_id| (*message_type_id as i32) == (type_id as i32));

    match message_type_id {
        Some(id) => Ok(id),
        None => Err(crate::errors::Error::InvalidMessageType),
    }
}
