//! Core I/O types and message serialization for the BLE plugin protocol
//!
//! This module provides the fundamental building blocks for protocol communication:
//! message type definitions, serialization traits, and header format handling.
//! It implements the core protocol specification including magic number validation,
//! type-safe message identification, and configurable serialization formats.
//!
//! ## Message Protocol
//!
//! All protocol messages follow a standardized format with a 5-byte header:
//!
//! ```text
//! ┌─────────────┬─────────────┬─────────────┬─────────────────┐
//! │   Magic     │   Type ID   │   Length    │     Payload     │
//! │  (2 bytes)  │  (1 byte)   │  (2 bytes)  │  (limited size) │
//! └─────────────┴─────────────┴─────────────┴─────────────────┘
//! ```
//!
//! **Size Constraints**: The total message size (header + payload) cannot exceed
//! [`DEFAULT_PACKET_SIZE`]. With a [`MESSAGE_HEADER_SIZE`] header, the maximum payload
//! size is [`DEFAULT_PACKET_SIZE`] - [`MESSAGE_HEADER_SIZE`] bytes.
//!
//! ## Serialization Configuration
//!
//! The serialization format is controlled at compile-time through feature flags.
//! **Exactly one** serialization method must be enabled:
//!
//! - **`bincode_serialization`** (default): Uses bincode for efficient binary serialization
//!   - Compact binary format with minimal overhead
//!   - Best for performance-critical embedded systems
//!   - Requires serde `Serialize`/`Deserialize` traits
//!
//! - **`protocol_buffers`**: Uses Protocol Buffers for serialization
//!   - Language-agnostic format with schema evolution support
//!   - Better for cross-platform/cross-language communication
//!   - Requires `prost::Message` trait implementation
//!   - Must implement `Default` trait for protobuf deserialization
//!
//! **Important**: Exactly one serialization method must be enabled. The crate will
//! fail to compile if both features are enabled simultaneously or if neither is
//! enabled. This is enforced by compile-time checks in `lib.rs`.
//!
//! ## Key Features
//!
//! - **Type Safety**: Compile-time message type verification
//! - **Configurable Serialization**: Choose between bincode or protobuf
//! - **Header Validation**: Magic number and length checking
//! - **Cross-Platform**: Works in both std and no_std environments
//! - **Zero-Copy**: Minimizes allocations in embedded contexts
//!
//! ## Usage
//!
//! ```rust,no_run
//! use protocol_io::HostIO;
//! use serde::{Serialize, Deserialize};
//! use protocol::{IO, MessageTypeId, DEFAULT_PACKET_SIZE};
//!
//! #[derive(Serialize, Deserialize)]
//! #[HostIO(MessageTypeId::TypeHostCommandConfigurePeripheral)]
//! struct MyCommand {
//!     data: u32,
//! }
//!
//! // IO trait methods are now automatically available
//! let cmd = MyCommand { data: 42 };
//! let serialized = cmd.to_bytes::<DEFAULT_PACKET_SIZE>()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::protocol::MessageTypeId;
use crate::{
    errors::{Error, Result},
    DEFAULT_PACKET_SIZE,
};
use serde::{Deserialize, Serialize};
/// Size in bytes of the message type identifier field
///
/// Each message includes a single byte identifying its type, enabling
/// efficient O(1) dispatch without trial-and-error deserialization.
pub const MESSAGE_TYPE_ID_BYTES: usize = 1;

/// Magic number for message integrity validation (0xDEAD)
///
/// This constant magic number is included at the start of every message
/// to validate message integrity and detect corruption. The value 0xDEAD
/// was chosen for easy identification in debugging and network analysis.
///
/// The magic number is transmitted in little-endian byte order for
/// consistency across different architectures.
pub const MESSAGE_MAGIC: u16 = 0xDEAD;

/// Size in bytes of the magic number field
///
/// The magic number occupies the first 2 bytes of every message header.
pub const MESSAGE_MAGIC_BYTES: usize = 2;

/// Total message header size in bytes
///
/// The header consists of:
/// - Magic number: 2 bytes
/// - Message type ID: 1 byte  
/// - Payload length: 2 bytes
/// - **Total: 5 bytes**
///
/// This constant is calculated from the sum of component sizes to ensure
/// consistency and prevent magic number errors.
pub const MESSAGE_HEADER_SIZE: usize =
    MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES + DATA_BYTES_LENGTH_IN_BYTES;

/// Size in bytes of the payload length field
///
/// The length field is a 2-byte little-endian value specifying the size
/// of the serialized payload data following the header. While the field
/// theoretically allows payloads up to 65,535 bytes, the practical limit
/// is [`DEFAULT_PACKET_SIZE`] - [`MESSAGE_HEADER_SIZE`] bytes due to the USB
/// packet size constraint of [`DEFAULT_PACKET_SIZE`].
pub const DATA_BYTES_LENGTH_IN_BYTES: usize = 2;

/// Trait for associating types with their message type identifiers
///
/// This trait must be implemented by all message types to enable automatic
/// message identification and dispatch. Each message type returns its unique
/// MessageTypeId for inclusion in the protocol header.
///
/// ## Implementation
///
/// ```rust
/// use protocol::{MessageType, MessageTypeId};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct MyCommand {
///     data: u32,
/// }
///
/// impl MessageType for MyCommand {
///     fn message_type_id() -> MessageTypeId {
///         MessageTypeId::TypeHostCommandConfigurePeripheral
///     }
/// }
/// ```
pub trait MessageType {
    /// Get the unique message type identifier for this type
    ///
    /// This method returns the MessageTypeId that will be included in the
    /// message header for efficient dispatch. Each message type must return
    /// a unique identifier.
    fn message_type_id() -> MessageTypeId;
}

/// Core I/O trait for protocol message serialization and deserialization
///
/// This trait provides a complete interface for converting between Rust types and
/// the wire protocol format. It automatically handles message headers, type
/// identification, and binary serialization using bincode.
///
/// ## Automatic Implementation
///
/// The `IO` trait contains default implementations for all serialization and deserialization
/// methods. Types automatically gain full `IO` functionality by implementing the required
/// prerequisite traits through attribute macros.
///
/// ### Prerequisite Traits
///
/// To use the `IO` trait, your type must implement:
/// - `Serialize` and `Deserialize` from serde (for data serialization)
/// - `MessageType` (for message type identification)
///
/// ### Automatic Implementation via Macros
///
/// The easiest way to implement these traits is using the `#[HostIO(...)]` or `#[PluginIO(...)]`
/// attribute macros from the `protocol_io` crate, which automatically implement all required traits:
///
/// ```rust,no_run  
/// use protocol_io::HostIO;
/// use serde::{Serialize, Deserialize};
/// use protocol::MessageTypeId;
///
/// #[derive(Serialize, Deserialize)]
/// #[HostIO(MessageTypeId::TypeHostCommandConfigurePeripheral)]
/// struct MyCommand {
///     data: u32,
/// }
/// // Now MyCommand automatically has all IO trait methods available
/// ```
///
/// ## Features
///
/// - **Header Management**: Automatically adds/validates protocol headers
/// - **Type Safety**: Compile-time message type verification
/// - **Efficient Encoding**: Binary serialization with minimal overhead
/// - **Cross-Platform**: Works in both std and no_std environments
/// - **Memory Flexible**: Supports both owned and borrowed serialization
///
/// ## Usage
///
/// ```rust,no_run
/// use protocol::{IO, DEFAULT_PACKET_SIZE};
/// # use protocol::io_types::HostCommandConfigurePeripheral;
/// # let command: HostCommandConfigurePeripheral = panic!("Documentation example");
///
/// // Serialize to owned buffer
/// let serialized: [u8; DEFAULT_PACKET_SIZE] = command.to_bytes()?;
///
/// // Serialize to provided buffer (no allocation)
/// let mut buffer = [0u8; DEFAULT_PACKET_SIZE];
/// command.to_bytes_in_slice(&mut buffer)?;
///
/// // Deserialize from received data
/// let deserialized = HostCommandConfigurePeripheral::from_bytes(&serialized)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait IO<'a>: IOBase<'a> {
    /// Serialize the message to a Vec (std only)
    ///
    /// This method serializes the message content (without header) to a
    /// dynamically allocated Vec. Available only when the `std` feature is enabled.
    ///
    /// ## Serialization Method Selection
    ///
    /// The serialization format is determined at compile-time via feature flags.
    /// Exactly one of the following must be enabled:
    /// - **`bincode_serialization`**: Uses bincode binary format
    /// - **`protocol_buffers`**: Uses Protocol Buffers format
    ///
    /// The crate enforces that exactly one serialization method is enabled
    /// through compile-time checks in `lib.rs`.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<u8>)` - Successfully serialized message data
    /// - `Err(Error)` - Serialization failed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use protocol::io_types::HostCommandConfigurePeripheral;
    /// # let command: HostCommandConfigurePeripheral = panic!("Documentation example");
    /// let payload_bytes = command.serialize_bytes()?;
    /// println!("Payload size: {} bytes", payload_bytes.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    #[cfg(feature = "std")]
    fn serialize_bytes(&self) -> Result<std::vec::Vec<u8>> {
        #[cfg(feature = "bincode_serialization")]
        return bincode::serde::encode_to_vec(self, bincode::config::standard())
            .map_err(|_| Error::UnableToSerializeToBincode);
        #[cfg(feature = "protocol_buffers")]
        return Ok(prost::Message::encode_to_vec(self));
    }

    /// Serialize the message to a provided slice (no allocation)
    ///
    /// This method serializes the message content (without header) into a
    /// provided buffer slice. This is the primary serialization method for
    /// no_std environments and memory-constrained applications.
    ///
    /// ## Serialization Method Selection
    ///
    /// The serialization format is determined at compile-time via feature flags.
    /// Exactly one of the following must be enabled:
    /// - **`bincode_serialization`**: Uses bincode binary format
    /// - **`protocol_buffers`**: Uses Protocol Buffers format with `prost::Message::encode`
    ///
    /// The crate enforces that exactly one serialization method is enabled
    /// through compile-time checks in `lib.rs`.
    ///
    /// # Arguments
    ///
    /// * `out` - Destination buffer for serialized data
    ///
    /// # Returns
    ///
    /// - `Ok(usize)` - Number of bytes written to the buffer
    /// - `Err(Error)` - Serialization failed or buffer too small
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use protocol::io_types::HostCommandConfigurePeripheral;
    /// # let command: HostCommandConfigurePeripheral = panic!("Documentation example");
    /// let mut buffer = [0u8; 128];
    /// let bytes_written = command.serialize_bytes_in_slice(&mut buffer)?;
    /// println!("Wrote {} bytes to buffer", bytes_written);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    #[allow(unused_mut)]
    fn serialize_bytes_in_slice(&self, mut out: &'a mut [u8]) -> Result<usize> {
        #[cfg(feature = "bincode_serialization")]
        return bincode::serde::encode_into_slice(self, out, bincode::config::standard())
            .map_err(|_| Error::UnableToSerializeToBincode);
        #[cfg(feature = "protocol_buffers")]
        {
            prost::Message::encode(self, &mut out)
                .map_err(|_| Error::UnableToSerializeToProtobuf)?;
            return Ok(self.encoded_len());
        }
    }

    /// Deserialize the message from a byte slice (no allocation)
    ///
    /// This method deserializes message content from a byte slice without
    /// allocating additional memory.
    ///
    /// ## Deserialization Method Selection
    ///
    /// The deserialization format is determined at compile-time via feature flags.
    /// Exactly one of the following must be enabled:
    /// - **`bincode_serialization`**: Uses bincode with `borrow_decode_from_slice`
    ///   for zero-copy deserialization where possible
    /// - **`protocol_buffers`**: Uses Protocol Buffers with `prost::Message::decode`
    ///
    /// The crate enforces that exactly one serialization method is enabled
    /// through compile-time checks in `lib.rs`.
    ///
    /// # Arguments
    ///
    /// * `payload` - Byte slice containing the serialized message data
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` - Successfully deserialized message
    /// - `Err(Error)` - Deserialization failed or invalid data
    #[inline(always)]
    #[allow(unused_variables)]
    fn deserialize_bytes(payload: &'a [u8]) -> Result<Self> {
        #[cfg(feature = "bincode_serialization")]
        {
            let res: Self =
                bincode::serde::borrow_decode_from_slice(&payload, bincode::config::standard())
                    .map_err(|_| Error::UnableToDeserializeFromBincode(""))?
                    .0;
            return Ok(res);
        }
        #[cfg(feature = "protocol_buffers")]
        {
            let res = prost::Message::decode(payload)
                .map_err(|_| Error::UnableToDeserializeFromProtobuf)?;
            return Ok(res);
        }
    }

    /// Convert from bytes
    #[inline(always)]
    fn from_bytes(input: &'a [u8]) -> Result<Self> {
        // Extract length from bytes 2-3 (after magic and type_id)
        let length: usize = (0..DATA_BYTES_LENGTH_IN_BYTES)
            .map(|i| ((input[MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES + i] as usize) << (i * 8)))
            .sum();
        if length > DEFAULT_PACKET_SIZE {
            return Err(Error::UnableToDeserializeFromBincode(
                "Packet size exceeds the allowable limit",
            ));
        }

        Self::deserialize_bytes(&input[MESSAGE_HEADER_SIZE..(MESSAGE_HEADER_SIZE + length)])
    }

    /// Serialize to bytes
    #[inline(always)]
    #[cfg(feature = "std")]
    fn to_bytes<const N: usize>(&self) -> Result<[u8; N]> {
        use lib_utils::MatchSliceLengths;

        let mut serialized_bytes = self.serialize_bytes()?;
        let length = serialized_bytes.len() as usize;

        // Create full message header: magic + type_id + length
        let mut header_data: std::vec::Vec<u8> = std::vec::Vec::new();

        // Add magic bytes (0xDEAD)
        header_data.extend_from_slice(&MESSAGE_MAGIC.to_le_bytes());

        // Add message type ID
        header_data.push(Self::message_type_id() as u8);

        // Add length bytes
        header_data
            .extend((0..DATA_BYTES_LENGTH_IN_BYTES).map(|x| ((length >> (x * 8)) & 0xFF) as u8));

        header_data.append(&mut serialized_bytes);

        if header_data.len() > N {
            return Err(Error::SerializationBufferOverflow);
        }
        Ok(header_data.match_size(0x00))
    }

    /// Serialize to bytes
    #[inline(always)]
    fn to_bytes_in_slice<const N: usize>(&self, buffer: &'a mut [u8; N]) -> Result<()> {
        let (header, payload) = buffer.split_at_mut(MESSAGE_HEADER_SIZE);
        let length = self.serialize_bytes_in_slice(payload)? as usize;

        // Create header: magic + type_id + length
        let mut header_bytes: heapless::Vec<u8, MESSAGE_HEADER_SIZE> = heapless::Vec::new();

        // Add magic bytes (0xDEAD)
        let _ = header_bytes.extend_from_slice(&MESSAGE_MAGIC.to_le_bytes());

        // Add message type ID
        let _ = header_bytes.push(Self::message_type_id() as u8);

        // Add length bytes
        for x in 0..DATA_BYTES_LENGTH_IN_BYTES {
            let _ = header_bytes.push(((length >> (x * 8)) & 0xFF) as u8);
        }

        header.copy_from_slice(&header_bytes);
        if MESSAGE_HEADER_SIZE + length > N {
            return Err(Error::SerializationBufferOverflow);
        }
        Ok(())
    }
}

/// Base trait for all protocol I/O types
///
/// This trait defines the requirements for types that can be serialized and
/// deserialized through the protocol. The exact requirements depend on the
/// enabled serialization features:
///
/// ## With `protocol_buffers` feature:
/// - `Serialize` + `Deserialize<'a>`: Required for compatibility
/// - `MessageType`: For protocol message type identification
/// - `prost::Message`: For Protocol Buffers serialization
/// - `Default`: Required by prost for protobuf deserialization
/// - `Sized`: Required for type safety
///
/// ## Without `protocol_buffers` feature (bincode only):
/// - `Serialize` + `Deserialize<'a>`: For bincode serialization
/// - `MessageType`: For protocol message type identification
/// - `Sized`: Required for type safety
#[cfg(feature = "protocol_buffers")]
pub trait IOBase<'a>:
    Serialize + Deserialize<'a> + Sized + MessageType + prost::Message + Default
{
}

/// Base trait for all protocol I/O types
///
/// This trait defines the requirements for types that can be serialized and
/// deserialized through the protocol using bincode serialization.
///
/// Required traits:
/// - `Serialize` + `Deserialize<'a>`: For bincode serialization
/// - `MessageType`: For protocol message type identification
/// - `Sized`: Required for type safety
#[cfg(not(feature = "protocol_buffers"))]
pub trait IOBase<'a>: Serialize + Deserialize<'a> + Sized + MessageType {}

/// Plugin specific input and output types
pub trait PluginIO<'a>: IO<'a> {}

/// Host specific input and output types
pub trait HostIO<'a>: IO<'a> {}
