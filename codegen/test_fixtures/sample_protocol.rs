/// Sample protocol definition for testing
/// This file represents a typical Rust protocol library structure

/// Magic number for message validation
pub const MESSAGE_MAGIC: u16 = 0xDEAD;

/// Maximum size for device names
pub const MAX_NAME_SIZE: usize = 64;

/// Default packet size for communication
pub const DEFAULT_PACKET_SIZE: usize = 256;

/// Message type identifiers for protocol dispatch
#[repr(u8)]
pub enum MessageTypeId {
    /// Configure BLE peripheral device
    HostCommandConfigurePeripheral = 0x01,
    
    /// Create a new BLE service
    HostCommandConfigureService = 0x02,
    
    /// Query service information
    HostCommandGetServiceInfo = 0x05,
    
    /// Data forwarded from BLE client
    PluginData = 0x80,
    
    /// Configuration error response
    PluginConfigurationError = 0x81,
    
    /// Service information response
    PluginServiceInfoResponse = 0x82,
}

/// BLE property flags for characteristics
#[repr(u8)]
pub enum BLEProperties {
    /// Characteristic supports read operations
    Read = 0x02,
    
    /// Characteristic supports write operations
    Write = 0x08,
    
    /// Characteristic supports notifications
    Notify = 0x10,
    
    /// Characteristic supports indications
    Indicate = 0x20,
}

/// Configuration message for BLE peripheral
pub struct HostCommandConfigurePeripheral {
    /// The device name to advertise
    pub name: heapless::String<32>,
    
    /// The primary service UUID
    pub uuid: String,
    
    /// Maximum number of concurrent connections
    pub max_connections: Option<u8>,
}

/// Service configuration message
pub struct HostCommandConfigureService {
    /// Service UUID
    pub uuid: String,
    
    /// Service name for identification
    pub name: heapless::String<16>,
    
    /// Whether this is a primary service
    pub is_primary: bool,
    
    /// List of characteristic UUIDs
    pub characteristics: Vec<String>,
}

/// Service information query
pub struct HostCommandGetServiceInfo {
    /// Service UUID to query
    pub service_uuid: String,
}

/// Data message from plugin to host
pub struct PluginData {
    /// Connection handle
    pub connection_id: u16,
    
    /// Characteristic UUID that received data
    pub characteristic_uuid: String,
    
    /// The actual data payload
    pub data: Vec<u8>,
    
    /// Timestamp when data was received
    pub timestamp: u64,
}

/// Configuration error response
pub struct PluginConfigurationError {
    /// Error code
    pub error_code: u16,
    
    /// Human-readable error message
    pub message: String,
    
    /// Optional context about what was being configured
    pub context: Option<String>,
}

/// Response to service information query
pub struct PluginServiceInfoResponse {
    /// Service UUID
    pub service_uuid: String,
    
    /// Service name
    pub service_name: String,
    
    /// List of characteristic information
    pub characteristics: Vec<CharacteristicInfo>,
    
    /// Whether service is currently active
    pub is_active: bool,
}

/// Information about a BLE characteristic
pub struct CharacteristicInfo {
    /// Characteristic UUID
    pub uuid: String,
    
    /// Characteristic properties
    pub properties: BLEProperties,
    
    /// Current value if readable
    pub current_value: Option<Vec<u8>>,
    
    /// Whether notifications are enabled
    pub notifications_enabled: bool,
}