//! Tests for nested module parsing functionality
//!
//! These tests specifically focus on the module parsing capabilities added to handle
//! the nested structure in io_types.rs with host and plugin modules.

use anyhow::Result;
use codegen::parse_rust_source;

/// Test parsing of deeply nested modules
#[test]
fn test_deeply_nested_modules() -> Result<()> {
    let source = r#"
        pub const TOP_LEVEL: u32 = 1;
        
        pub mod level1 {
            pub const LEVEL1_CONST: u32 = 2;
            
            pub enum Level1Enum {
                Variant1 = 10,
                Variant2 = 20,
            }
            
            pub mod level2 {
                pub const LEVEL2_CONST: u32 = 3;
                
                pub struct Level2Struct {
                    pub field1: u32,
                    pub field2: String,
                }
                
                pub mod level3 {
                    pub const LEVEL3_CONST: u32 = 4;
                    
                    pub struct Level3Struct {
                        pub deep_field: bool,
                    }
                }
            }
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    // Should find only public constants from all levels
    assert_eq!(result.constants.len(), 4);
    let constant_names: Vec<_> = result.constants.iter().map(|c| &c.name).collect();
    assert!(constant_names.contains(&&"TOP_LEVEL".to_string()));
    assert!(constant_names.contains(&&"LEVEL1_CONST".to_string()));
    assert!(constant_names.contains(&&"LEVEL2_CONST".to_string()));
    assert!(constant_names.contains(&&"LEVEL3_CONST".to_string()));
    
    // Should find enum from nested module
    assert_eq!(result.enums.len(), 1);
    assert_eq!(result.enums[0].name, "Level1Enum");
    assert_eq!(result.enums[0].variants.len(), 2);
    
    // Should find structs from nested modules
    assert_eq!(result.structs.len(), 2);
    let struct_names: Vec<_> = result.structs.iter().map(|s| &s.name).collect();
    assert!(struct_names.contains(&&"Level2Struct".to_string()));
    assert!(struct_names.contains(&&"Level3Struct".to_string()));
    
    Ok(())
}

/// Test parsing modules with mixed visibility
#[test]
fn test_mixed_visibility_modules() -> Result<()> {
    let source = r#"
        pub mod public_mod {
            pub const PUB_CONST: u32 = 1;
            const PRIVATE_CONST: u32 = 2;
            
            pub struct PubStruct {
                pub field: u32,
            }
            
            struct PrivateStruct {
                field: u32,
            }
        }
        
        mod private_mod {
            pub const CONST_IN_PRIVATE_MOD: u32 = 3;
            
            pub struct StructInPrivateMod {
                pub field: bool,
            }
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    // Should find only public constants (ignoring private ones)
    assert_eq!(result.constants.len(), 2);
    let constant_names: Vec<_> = result.constants.iter().map(|c| &c.name).collect();
    assert!(constant_names.contains(&&"PUB_CONST".to_string()));
    assert!(constant_names.contains(&&"CONST_IN_PRIVATE_MOD".to_string()));
    // PRIVATE_CONST should not be found since it's not pub
    
    // Should find only public structs (ignoring private ones)
    assert_eq!(result.structs.len(), 2);
    let struct_names: Vec<_> = result.structs.iter().map(|s| &s.name).collect();
    assert!(struct_names.contains(&&"PubStruct".to_string()));
    assert!(struct_names.contains(&&"StructInPrivateMod".to_string()));
    // PrivateStruct should not be found since it's not pub
    
    Ok(())
}

/// Test parsing modules with complex type definitions
#[test]
fn test_modules_with_complex_types() -> Result<()> {
    let source = r#"
        pub mod types {
            use uuid::Uuid;
            
            pub const MAX_SIZE: usize = 100;
            
            pub enum DataType {
                Binary,
                Text,
                Json,
            }
            
            pub struct ComplexStruct<'a> {
                pub id: Uuid,
                pub data: &'a [u8],
                pub metadata: Vec<String>,
                pub optional_field: Option<u32>,
                pub size_limited: heapless::Vec<u8, 32>,
            }
        }
        
        pub mod handlers {
            use super::types::*;
            
            pub struct Handler {
                pub name: heapless::String<64>,
                pub data_type: DataType,
            }
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    // Should find constant
    assert_eq!(result.constants.len(), 1);
    assert_eq!(result.constants[0].name, "MAX_SIZE");
    
    // Should find enum
    assert_eq!(result.enums.len(), 1);
    assert_eq!(result.enums[0].name, "DataType");
    assert_eq!(result.enums[0].variants.len(), 3);
    
    // Should find structs with complex types
    assert_eq!(result.structs.len(), 2);
    
    let complex_struct = result.structs.iter()
        .find(|s| s.name == "ComplexStruct")
        .expect("Should find ComplexStruct");
    
    // Check field type conversions
    let id_field = complex_struct.fields.iter()
        .find(|f| f.name == "id")
        .expect("Should find id field");
    assert_eq!(id_field.python_type, "bytes");
    
    let data_field = complex_struct.fields.iter()
        .find(|f| f.name == "data")
        .expect("Should find data field");
    assert_eq!(data_field.python_type, "bytes");
    
    let metadata_field = complex_struct.fields.iter()
        .find(|f| f.name == "metadata")
        .expect("Should find metadata field");
    assert_eq!(metadata_field.python_type, "List[str]");
    
    let optional_field = complex_struct.fields.iter()
        .find(|f| f.name == "optional_field")
        .expect("Should find optional_field");
    assert_eq!(optional_field.python_type, "Optional[attrs2bin.U32]");
    assert!(optional_field.is_optional);
    
    let size_limited_field = complex_struct.fields.iter()
        .find(|f| f.name == "size_limited")
        .expect("Should find size_limited field");
    assert_eq!(size_limited_field.python_type, "List[attrs2bin.U8]");
    
    Ok(())
}

/// Test parsing modules with documentation
#[test]
fn test_modules_with_documentation() -> Result<()> {
    let source = r#"
        /// Main module for protocol definitions
        pub mod protocol {
            /// Maximum packet size constant
            pub const MAX_PACKET_SIZE: usize = 1024;
            
            /// Protocol message types
            pub enum MessageType {
                /// Request message
                Request = 1,
                /// Response message  
                Response = 2,
            }
            
            /// Protocol packet structure
            pub struct Packet {
                /// Message type identifier
                pub msg_type: MessageType,
                /// Packet payload data
                pub payload: Vec<u8>,
            }
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    // Check that documentation is preserved
    assert_eq!(result.constants.len(), 1);
    let constant = &result.constants[0];
    assert_eq!(constant.name, "MAX_PACKET_SIZE");
    assert!(constant.doc_comment.contains("Maximum packet size"));
    
    assert_eq!(result.enums.len(), 1);
    let enum_def = &result.enums[0];
    assert_eq!(enum_def.name, "MessageType");
    assert!(enum_def.doc_comment.contains("Protocol message types"));
    
    // Check enum variant documentation
    let request_variant = enum_def.variants.iter()
        .find(|v| v.name == "Request")
        .expect("Should find Request variant");
    assert!(request_variant.doc_comment.contains("Request message"));
    
    assert_eq!(result.structs.len(), 1);
    let struct_def = &result.structs[0];
    assert_eq!(struct_def.name, "Packet");
    assert!(struct_def.doc_comment.contains("Protocol packet structure"));
    
    // Check field documentation
    let msg_type_field = struct_def.fields.iter()
        .find(|f| f.name == "msg_type")
        .expect("Should find msg_type field");
    assert!(msg_type_field.doc_comment.contains("Message type identifier"));
    
    Ok(())
}

/// Test parsing empty modules and modules with only ignored items
#[test]  
fn test_empty_and_ignored_modules() -> Result<()> {
    let source = r#"
        pub mod empty_mod {
        }
        
        pub mod ignored_items_mod {
            use std::collections::HashMap;
            
            pub fn some_function() {
                println!("ignored");
            }
            
            impl SomeStruct {
                pub fn method(&self) {}
            }
            
            pub trait SomeTrait {
                fn trait_method(&self);
            }
        }
        
        pub mod mixed_mod {
            pub const VALID_CONST: u32 = 42;
            
            pub fn ignored_function() {}
            
            pub struct ValidStruct {
                pub field: u32,
            }
            
            impl ValidStruct {
                pub fn ignored_method(&self) {}
            }
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    // Should only find items from mixed_mod
    assert_eq!(result.constants.len(), 1);
    assert_eq!(result.constants[0].name, "VALID_CONST");
    
    assert_eq!(result.structs.len(), 1);
    assert_eq!(result.structs[0].name, "ValidStruct");
    
    assert_eq!(result.enums.len(), 0);
    
    Ok(())
}

/// Test parsing modules that mirror the actual io_types.rs structure
#[test]
fn test_io_types_like_structure() -> Result<()> {
    let source = r#"
        pub const MAX_PROPERTIES: usize = 4;
        pub const MAX_CHARACTERISTICS_PER_SERVICE: usize = 16;
        
        pub mod host {
            use uuid::Uuid;
            use heapless::String;
            
            pub struct HostCommandConfigurePeripheral {
                pub name: String<30>,
                pub uuid: Uuid,
            }
            
            pub struct HostCommandConfigureService {
                pub uuid: Uuid,
            }
            
            pub struct HostCommandGetServiceInfo {
                pub uuid: Uuid,
            }
        }
        
        pub mod plugin {
            use uuid::Uuid;
            
            pub enum PluginDataSendType {
                Notify,
                Read, 
                Write,
            }
            
            pub enum PluginConfigurationError {
                PeripheralNameTooLong,
                InvalidPeripheralUuid,
                InvalidServiceUuid,
            }
            
            pub struct PluginData<'a> {
                pub src_id: Uuid,
                pub send_type: PluginDataSendType,
                pub data: &'a [u8],
            }
            
            pub struct PluginServiceInfoResponse {
                pub service_uuid: Uuid,
                pub characteristic_uuids: heapless::Vec<Uuid, 16>,
                pub exists: bool,
            }
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    // Should find top-level constants
    assert_eq!(result.constants.len(), 2);
    let constant_names: Vec<_> = result.constants.iter().map(|c| &c.name).collect();
    assert!(constant_names.contains(&&"MAX_PROPERTIES".to_string()));
    assert!(constant_names.contains(&&"MAX_CHARACTERISTICS_PER_SERVICE".to_string()));
    
    // Should find enums from plugin module
    assert_eq!(result.enums.len(), 2);
    let enum_names: Vec<_> = result.enums.iter().map(|e| &e.name).collect();
    assert!(enum_names.contains(&&"PluginDataSendType".to_string()));
    assert!(enum_names.contains(&&"PluginConfigurationError".to_string()));
    
    // Should find structs from both modules
    assert_eq!(result.structs.len(), 5);
    let struct_names: Vec<_> = result.structs.iter().map(|s| &s.name).collect();
    assert!(struct_names.contains(&&"HostCommandConfigurePeripheral".to_string()));
    assert!(struct_names.contains(&&"HostCommandConfigureService".to_string()));
    assert!(struct_names.contains(&&"HostCommandGetServiceInfo".to_string()));
    assert!(struct_names.contains(&&"PluginData".to_string()));
    assert!(struct_names.contains(&&"PluginServiceInfoResponse".to_string()));
    
    // Check specific type conversions
    let plugin_data = result.structs.iter()
        .find(|s| s.name == "PluginData")
        .expect("Should find PluginData");
    
    let data_field = plugin_data.fields.iter()
        .find(|f| f.name == "data")
        .expect("Should find data field");
    assert_eq!(data_field.python_type, "bytes");
    
    let service_info = result.structs.iter()
        .find(|s| s.name == "PluginServiceInfoResponse")
        .expect("Should find PluginServiceInfoResponse");
    
    let uuids_field = service_info.fields.iter()
        .find(|f| f.name == "characteristic_uuids")
        .expect("Should find characteristic_uuids field");
    assert_eq!(uuids_field.python_type, "List[bytes]");
    
    Ok(())
}

/// Test that module parsing doesn't break existing non-module parsing
#[test]
fn test_backwards_compatibility() -> Result<()> {
    let source = r#"
        /// Top level constant
        pub const TOP_CONST: u32 = 1;
        
        /// Top level enum
        pub enum TopEnum {
            Variant1 = 10,
            Variant2 = 20,
        }
        
        /// Top level struct
        pub struct TopStruct {
            pub field1: u32,
            pub field2: String,
        }
        
        // This should be ignored
        pub fn some_function() {}
        
        // This should also be ignored
        impl TopStruct {
            pub fn method(&self) {}
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    // Should work exactly as before
    assert_eq!(result.constants.len(), 1);
    assert_eq!(result.constants[0].name, "TOP_CONST");
    
    assert_eq!(result.enums.len(), 1);
    assert_eq!(result.enums[0].name, "TopEnum");
    assert_eq!(result.enums[0].variants.len(), 2);
    
    assert_eq!(result.structs.len(), 1);
    assert_eq!(result.structs[0].name, "TopStruct");
    assert_eq!(result.structs[0].fields.len(), 2);
    
    Ok(())
}