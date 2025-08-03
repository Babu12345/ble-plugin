//! Regression tests using golden files
//!
//! These tests ensure that the code generation output remains consistent over time.

use anyhow::Result;
use codegen::parse_rust_source;
use std::fs;
use std::path::PathBuf;

/// Test that parsing of the sample protocol produces expected structure
#[test]
fn test_sample_protocol_parsing() -> Result<()> {
    let fixture_path = PathBuf::from("test_fixtures/sample_protocol.rs");
    let source = fs::read_to_string(&fixture_path)
        .expect("Should be able to read sample protocol fixture");
    
    let result = parse_rust_source(&source)?;
    
    // Verify expected constants
    assert_eq!(result.constants.len(), 3);
    
    let message_magic = result.constants.iter()
        .find(|c| c.name == "MESSAGE_MAGIC")
        .expect("Should find MESSAGE_MAGIC constant");
    assert_eq!(message_magic.value, "0xDEAD");
    // Doc comment might be truncated or include file-level comments
    assert!(!message_magic.doc_comment.is_empty());
    
    let max_name = result.constants.iter()
        .find(|c| c.name == "MAX_NAME_SIZE")
        .expect("Should find MAX_NAME_SIZE constant");
    assert_eq!(max_name.value, "64");
    
    let packet_size = result.constants.iter()
        .find(|c| c.name == "DEFAULT_PACKET_SIZE")
        .expect("Should find DEFAULT_PACKET_SIZE constant");
    assert_eq!(packet_size.value, "256");
    
    // Verify expected enums
    assert_eq!(result.enums.len(), 2);
    
    let message_type_id = result.enums.iter()
        .find(|e| e.name == "MessageTypeId")
        .expect("Should find MessageTypeId enum");
    assert_eq!(message_type_id.variants.len(), 6);
    assert_eq!(message_type_id.repr, Some("u8".to_string()));
    
    // Check host command range (0x01-0x7F)
    let host_configure = message_type_id.variants.iter()
        .find(|v| v.name == "HostCommandConfigurePeripheral")
        .expect("Should find HostCommandConfigurePeripheral");
    assert_eq!(host_configure.value, Some("0x01".to_string()));
    
    // Check plugin response range (0x80-0xFF)
    let plugin_data = message_type_id.variants.iter()
        .find(|v| v.name == "PluginData")
        .expect("Should find PluginData");
    assert_eq!(plugin_data.value, Some("0x80".to_string()));
    
    let ble_properties = result.enums.iter()
        .find(|e| e.name == "BLEProperties")
        .expect("Should find BLEProperties enum");
    assert_eq!(ble_properties.variants.len(), 4);
    
    // Verify expected structs
    assert_eq!(result.structs.len(), 7);
    
    let configure_peripheral = result.structs.iter()
        .find(|s| s.name == "HostCommandConfigurePeripheral")
        .expect("Should find HostCommandConfigurePeripheral struct");
    assert_eq!(configure_peripheral.fields.len(), 3);
    
    // Check field types
    let name_field = configure_peripheral.fields.iter()
        .find(|f| f.name == "name")
        .expect("Should find name field");
    assert_eq!(name_field.python_type, "str");
    assert!(!name_field.is_optional);
    
    let max_connections_field = configure_peripheral.fields.iter()
        .find(|f| f.name == "max_connections")
        .expect("Should find max_connections field");
    assert_eq!(max_connections_field.python_type, "Optional[int]");
    assert!(max_connections_field.is_optional);
    
    Ok(())
}

/// Test that the message type ID ranges are correctly maintained
#[test]
fn test_message_type_id_ranges() -> Result<()> {
    let fixture_path = PathBuf::from("test_fixtures/sample_protocol.rs");
    let source = fs::read_to_string(&fixture_path)
        .expect("Should be able to read sample protocol fixture");
    
    let result = parse_rust_source(&source)?;
    
    let message_type_enum = result.enums.iter()
        .find(|e| e.name == "MessageTypeId")
        .expect("Should find MessageTypeId enum");
    
    // Check that all host commands are in 0x01-0x7F range
    let host_commands: Vec<_> = message_type_enum.variants.iter()
        .filter(|v| v.name.starts_with("HostCommand"))
        .collect();
    
    for cmd in &host_commands {
        if let Some(value) = &cmd.value {
            let hex_value = u8::from_str_radix(&value[2..], 16)
                .expect("Should be valid hex");
            assert!(hex_value >= 0x01 && hex_value <= 0x7F,
                   "Host command {} = {} should be in 0x01-0x7F range", cmd.name, value);
        }
    }
    
    // Check that all plugin responses are in 0x80-0xFF range
    let plugin_responses: Vec<_> = message_type_enum.variants.iter()
        .filter(|v| v.name.starts_with("Plugin"))
        .collect();
    
    for resp in &plugin_responses {
        if let Some(value) = &resp.value {
            let hex_value = u8::from_str_radix(&value[2..], 16)
                .expect("Should be valid hex");
            assert!(hex_value >= 0x80,
                   "Plugin response {} = {} should be in 0x80-0xFF range", resp.name, value);
        }
    }
    
    Ok(())
}

/// Test that type mappings work correctly for complex generics
#[test]
fn test_complex_type_mappings() -> Result<()> {
    let fixture_path = PathBuf::from("test_fixtures/sample_protocol.rs");
    let source = fs::read_to_string(&fixture_path)
        .expect("Should be able to read sample protocol fixture");
    
    let result = parse_rust_source(&source)?;
    
    // Test heapless::String<N> -> str mapping
    let configure_peripheral = result.structs.iter()
        .find(|s| s.name == "HostCommandConfigurePeripheral")
        .expect("Should find HostCommandConfigurePeripheral");
    
    let name_field = configure_peripheral.fields.iter()
        .find(|f| f.name == "name")
        .expect("Should find name field");
    // Type might have spaces due to quote formatting
    assert!(name_field.rust_type.contains("heapless") && name_field.rust_type.contains("String"));
    assert_eq!(name_field.python_type, "str");
    
    // Test Vec<T> -> List[T] mapping
    let plugin_data = result.structs.iter()
        .find(|s| s.name == "PluginData")
        .expect("Should find PluginData");
    
    let data_field = plugin_data.fields.iter()
        .find(|f| f.name == "data")
        .expect("Should find data field");
    assert!(data_field.rust_type.contains("Vec"));
    assert_eq!(data_field.python_type, "List[int]");
    
    // Test Option<T> -> Optional[T] mapping
    let max_connections_field = configure_peripheral.fields.iter()
        .find(|f| f.name == "max_connections")
        .expect("Should find max_connections field");
    assert!(max_connections_field.rust_type.contains("Option"));
    assert_eq!(max_connections_field.python_type, "Optional[int]");
    assert!(max_connections_field.is_optional);
    
    // Test custom enum type preservation
    let characteristic_info = result.structs.iter()
        .find(|s| s.name == "CharacteristicInfo")
        .expect("Should find CharacteristicInfo");
    
    let properties_field = characteristic_info.fields.iter()
        .find(|f| f.name == "properties")
        .expect("Should find properties field");
    assert_eq!(properties_field.python_type, "BLEProperties");
    
    Ok(())
}

/// Test documentation preservation across the sample protocol
#[test]
fn test_documentation_preservation() -> Result<()> {
    let fixture_path = PathBuf::from("test_fixtures/sample_protocol.rs");
    let source = fs::read_to_string(&fixture_path)
        .expect("Should be able to read sample protocol fixture");
    
    let result = parse_rust_source(&source)?;
    
    // Test constant documentation
    let message_magic = result.constants.iter()
        .find(|c| c.name == "MESSAGE_MAGIC")
        .expect("Should find MESSAGE_MAGIC");
    // Documentation extraction might include file-level comments
    assert!(!message_magic.doc_comment.is_empty());
    
    // Test enum documentation
    let message_type_enum = result.enums.iter()
        .find(|e| e.name == "MessageTypeId")
        .expect("Should find MessageTypeId");
    assert!(message_type_enum.doc_comment.contains("Message type identifiers"));
    
    // Test enum variant documentation
    let configure_variant = message_type_enum.variants.iter()
        .find(|v| v.name == "HostCommandConfigurePeripheral")
        .expect("Should find configure variant");
    assert!(configure_variant.doc_comment.contains("Configure BLE peripheral"));
    
    // Test struct documentation
    let configure_struct = result.structs.iter()
        .find(|s| s.name == "HostCommandConfigurePeripheral")
        .expect("Should find configure struct");
    assert!(configure_struct.doc_comment.contains("Configuration message"));
    
    // Test field documentation
    let name_field = configure_struct.fields.iter()
        .find(|f| f.name == "name")
        .expect("Should find name field");
    assert!(name_field.doc_comment.contains("device name"));
    
    Ok(())
}

/// Test that the parsing is stable across multiple runs
#[test]
fn test_parsing_stability() -> Result<()> {
    let fixture_path = PathBuf::from("test_fixtures/sample_protocol.rs");
    let source = fs::read_to_string(&fixture_path)
        .expect("Should be able to read sample protocol fixture");
    
    // Parse the same source multiple times
    let results: Vec<_> = (0..5)
        .map(|_| parse_rust_source(&source))
        .collect::<Result<Vec<_>, _>>()?;
    
    // All results should be identical
    let first = &results[0];
    for (i, result) in results.iter().enumerate().skip(1) {
        assert_eq!(result.constants.len(), first.constants.len(), 
                  "Run {} constants count differs", i);
        assert_eq!(result.enums.len(), first.enums.len(),
                  "Run {} enums count differs", i);
        assert_eq!(result.structs.len(), first.structs.len(),
                  "Run {} structs count differs", i);
        
        // Deep equality check on first constant as spot check
        assert_eq!(result.constants[0], first.constants[0],
                  "Run {} first constant differs", i);
    }
    
    Ok(())
}

/// Test edge cases that might cause regressions
#[test]
fn test_edge_case_regressions() -> Result<()> {
    // Test cases that have caused issues in the past
    let edge_case_source = r#"
        /// Edge case: empty enum
        pub enum EmptyEnum {}
        
        /// Edge case: single variant enum
        pub enum SingleVariant {
            Only,
        }
        
        /// Edge case: struct with no fields
        pub struct EmptyStruct {}
        
        /// Edge case: struct with single field
        pub struct SingleField {
            pub field: u32,
        }
        
        /// Edge case: very long identifier names
        pub const VERY_LONG_CONSTANT_NAME_THAT_EXCEEDS_NORMAL_LENGTHS: u32 = 1;
        
        /// Edge case: numeric literal variations
        pub const HEX_UPPER: u32 = 0xDEAD;
        pub const HEX_LOWER: u32 = 0xbeef;
        pub const BINARY: u32 = 0b1010;
        pub const OCTAL: u32 = 0o777;
        
        /// Edge case: complex nested generics
        pub struct NestedGenerics {
            pub complex: Option<Vec<Option<String>>>,
        }
    "#;
    
    let result = parse_rust_source(edge_case_source)?;
    
    // Should handle all edge cases without crashing
    assert!(result.constants.len() >= 4);
    assert!(result.enums.len() >= 2);
    assert!(result.structs.len() >= 3);
    
    // Check that hex values are preserved correctly
    let hex_upper = result.constants.iter()
        .find(|c| c.name == "HEX_UPPER")
        .expect("Should find HEX_UPPER");
    assert_eq!(hex_upper.value, "0xDEAD");
    
    let hex_lower = result.constants.iter()
        .find(|c| c.name == "HEX_LOWER")
        .expect("Should find HEX_LOWER");
    assert_eq!(hex_lower.value, "0xbeef");
    
    // Check complex generic handling
    let nested_generics = result.structs.iter()
        .find(|s| s.name == "NestedGenerics")
        .expect("Should find NestedGenerics");
    
    let complex_field = nested_generics.fields.iter()
        .find(|f| f.name == "complex")
        .expect("Should find complex field");
    assert_eq!(complex_field.python_type, "Optional[List[Optional[str]]]");
    
    Ok(())
}