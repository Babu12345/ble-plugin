//! Core I/O types and message serialization for the BLE plugin protocol
//!
//! This module provides the fundamental building blocks for protocol communication:
//! message type definitions, serialization traits, and header format handling.
//! It implements the core protocol specification including magic number validation,
//! type-safe message identification, and efficient binary serialization.
//!
//! ## Message Protocol
//!
//! All protocol messages follow a standardized format with a 5-byte header:
//!
//! ```text
//! ┌─────────────┬─────────────┬─────────────┬─────────────────┐
//! │   Magic     │   Type ID   │   Length    │     Payload     │
//! │  (2 bytes)  │  (1 byte)   │  (2 bytes)  │   (variable)    │
//! └─────────────┴─────────────┴─────────────┴─────────────────┘
//! ```
//!
//! ## Key Features
//!
//! - **Type Safety**: Compile-time message type verification
//! - **Efficient Serialization**: Binary encoding using bincode
//! - **Header Validation**: Magic number and length checking
//! - **Cross-Platform**: Works in both std and no_std environments
//! - **Zero-Copy**: Minimizes allocations in embedded contexts
//!
//! ## Usage
//!
//! ```rust,no_run
//! use protocol::{IO, MessageType, MessageTypeId};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct MyCommand {
//!     data: u32,
//! }
//!
//! impl MessageType for MyCommand {
//!     fn message_type_id() -> MessageTypeId {
//!         MessageTypeId::HostCommandConfigurePeripheral
//!     }
//! }
//!
//! // Now MyCommand automatically implements IO trait
//! let cmd = MyCommand { data: 42 };
//! let serialized = cmd.to_bytes::<256>()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::{
    errors::{Error, Result},
    DEFAULT_PACKET_SIZE,
};
use heapless::Vec;
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

/// Message type identifiers for efficient command dispatch
///
/// This enum defines unique identifiers for each message type in the protocol,
/// enabling O(1) message dispatch without trial-and-error deserialization.
/// The type IDs are organized into logical ranges for easy identification.
///
/// ## Type ID Ranges
///
/// - **0x01-0x0F**: Host commands sent to plugin device
/// - **0x10-0x1F**: Plugin responses sent to host device
/// - **0x20+**: Reserved for future extensions
///
/// ## Usage
///
/// ```rust
/// use protocol::MessageTypeId;
///
/// // Check if message is a host command
/// let is_host_command = (type_id as u8) < 0x10;
///
/// // Check if message is a plugin response  
/// let is_plugin_response = (type_id as u8) >= 0x10;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageTypeId {
    // Host Commands (0x01-0x0F)
    
    /// Configure BLE peripheral device with name and UUID
    HostCommandConfigurePeripheral = 0x01,
    
    /// Create a new BLE service with specified UUID
    HostCommandConfigureService = 0x02,
    
    /// Create a BLE characteristic with properties
    HostCommandConfigureCharacteristic = 0x03,
    
    /// Configure characteristic for read operations with default value
    HostCommandConfigureCharacteristicRead = 0x04,
    
    /// Query information about a BLE service
    HostCommandGetServiceInfo = 0x05,
    
    /// Query information about a BLE characteristic
    HostCommandGetCharacteristicInfo = 0x06,
    
    /// Start BLE advertising with optional multi-connect support
    HostCommandStartAdvertisement = 0x07,
    
    /// Send notification/indication to connected BLE client
    HostCommandNotifyCharacteristicValue = 0x08,
    
    // Plugin Responses (0x10+)
    
    /// Data forwarded from BLE client to host
    PluginData = 0x10,
    
    /// Configuration error response from plugin
    PluginConfigurationError = 0x11,
    
    /// Service information response with characteristic list
    PluginServiceInfoResponse = 0x12,
    
    /// Characteristic information response with properties
    PluginCharacteristicInfoResponse = 0x13,
}

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
///         MessageTypeId::HostCommandConfigurePeripheral
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

/// Size in bytes of the payload length field
///
/// The length field is a 2-byte little-endian value specifying the size
/// of the serialized payload data following the header. This allows for
/// payloads up to 65,535 bytes, though practical limits are imposed by
/// USB packet size constraints.
pub const DATA_BYTES_LENGTH_IN_BYTES: usize = 2;

/// Core I/O trait for protocol message serialization and deserialization
///
/// This trait provides a complete interface for converting between Rust types and
/// the wire protocol format. It automatically handles message headers, type
/// identification, and binary serialization using bincode.
///
/// ## Automatic Implementation
///
/// Any type that implements `Serialize`, `Deserialize`, and `MessageType` automatically
/// gets the full `IO` trait implementation. No manual implementation required.
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
pub trait IO<'a>: Serialize + Deserialize<'a> + Sized + MessageType {
    /// Serialize the message to a Vec using bincode (std only)
    ///
    /// This method serializes the message content (without header) to a
    /// dynamically allocated Vec. Available only when the `std` feature is enabled.
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
        bincode::serde::encode_to_vec(self, bincode::config::standard())
            .map_err(|_| Error::UnableToSerializeToBincode)
    }

    /// Serialize the message to a provided slice (no allocation)
    ///
    /// This method serializes the message content (without header) into a
    /// provided buffer slice. This is the primary serialization method for
    /// no_std environments and memory-constrained applications.
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
    fn serialize_bytes_in_slice(&self, out: &'a mut [u8]) -> Result<usize> {
        bincode::serde::encode_into_slice(self, out, bincode::config::standard())
            .map_err(|_| Error::UnableToSerializeToBincode)
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

        let res: Self = bincode::serde::borrow_decode_from_slice(
            &input[MESSAGE_HEADER_SIZE..(MESSAGE_HEADER_SIZE + length)],
            bincode::config::standard(),
        )
        .map_err(|_| Error::UnableToDeserializeFromBincode(""))?
        .0;

        Ok(res)
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
        let mut header_bytes: Vec<u8, MESSAGE_HEADER_SIZE> = Vec::new();

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

/// Plugin specific input and output types
pub trait PluginIO<'a>: IO<'a> {}

/// Host specific input and output types
pub trait HostIO<'a>: IO<'a> {}
