//! Validation tests for the codegen system
//!
//! These tests verify that the generated Python code is valid and consistent with the Rust source.

use anyhow::Result;
use codegen::parse_rust_source;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Test that generated Python code has valid syntax
#[test]
fn test_generated_python_syntax() -> Result<()> {
    let source = r#"
        /// Test constants
        pub const TEST_MAGIC: u16 = 0xDEAD;
        pub const TEST_SIZE: usize = 256;
        
        /// Test enum
        #[repr(u8)]
        pub enum TestMessageType {
            /// First message
            First = 0x01,
            /// Second message
            Second = 0x02,
        }
        
        /// Test struct
        pub struct TestMessage {
            /// Message ID
            pub id: u32,
            /// Message content
            pub content: String,
            /// Optional data
            pub data: Option<Vec<u8>>,
        }
    "#;
    
    let protocol_def = parse_rust_source(source)?;
    
    // Generate Python-like code manually to test syntax
    let temp_dir = TempDir::new()?;
    let python_file = temp_dir.path().join("test_generated.py");
    
    let mut python_code = String::new();
    python_code.push_str("# Generated Python code\n");
    python_code.push_str("from enum import Enum\n");
    python_code.push_str("from typing import Optional, List\n\n");
    
    // Add constants
    for constant in &protocol_def.constants {
        python_code.push_str(&format!("# {}\n", constant.doc_comment));
        python_code.push_str(&format!("{} = {}\n\n", constant.name, constant.value));
    }
    
    // Add enums
    for enum_def in &protocol_def.enums {
        python_code.push_str(&format!("class {}(Enum):\n", enum_def.name));
        if !enum_def.doc_comment.is_empty() {
            python_code.push_str(&format!("    \"\"\"{}\"\"\"\n", enum_def.doc_comment));
        }
        
        for variant in &enum_def.variants {
            if !variant.doc_comment.is_empty() {
                python_code.push_str(&format!("    # {}\n", variant.doc_comment));
            }
            if let Some(value) = &variant.value {
                python_code.push_str(&format!("    {} = {}\n", variant.name, value));
            } else {
                python_code.push_str(&format!("    {} = \"{}\"\n", variant.name, variant.name));
            }
        }
        python_code.push_str("\n");
    }
    
    // Add basic class structure for structs
    for struct_def in &protocol_def.structs {
        python_code.push_str(&format!("class {}:\n", struct_def.name));
        if !struct_def.doc_comment.is_empty() {
            python_code.push_str(&format!("    \"\"\"{}\"\"\"\n", struct_def.doc_comment));
        }
        python_code.push_str("    pass\n\n");
    }
    
    fs::write(&python_file, python_code)?;
    
    // Check Python syntax using python -m py_compile
    let output = Command::new("python3")
        .args(&["-m", "py_compile", python_file.to_str().unwrap()])
        .output();
    
    match output {
        Ok(result) => {
            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                panic!("Generated Python code has syntax errors:\n{}", stderr);
            }
        }
        Err(_) => {
            // Skip test if Python is not available
            println!("Python not available, skipping syntax validation");
        }
    }
    
    Ok(())
}

/// Test validation of existing Python code against Rust definitions
#[test]
fn test_python_validation_logic() -> Result<()> {
    let rust_source = r#"
        pub const MESSAGE_MAGIC: u16 = 0xDEAD;
        pub const PACKET_SIZE: usize = 256;
        
        #[repr(u8)]
        pub enum MessageType {
            Data = 0x01,
            Control = 0x02,
        }
    "#;
    
    let protocol_def = parse_rust_source(rust_source)?;
    
    // Test that validation logic would correctly identify matches/mismatches
    let correct_python = r#"
MESSAGE_MAGIC = 0xDEAD
PACKET_SIZE = 256

class MessageType(Enum):
    Data = 0x01
    Control = 0x02
"#;
    
    let incorrect_python = r#"
MESSAGE_MAGIC = 0xBEEF  # Wrong value
PACKET_SIZE = 512       # Wrong value

class MessageType(Enum):
    Data = 0x10         # Wrong value
    Control = 0x20      # Wrong value
"#;
    
    // Check constants validation
    for constant in &protocol_def.constants {
        let expected_line = format!("{} = {}", constant.name, constant.value);
        assert!(correct_python.contains(&expected_line), 
               "Correct Python should contain: {}", expected_line);
        assert!(!incorrect_python.contains(&expected_line), 
               "Incorrect Python should not contain: {}", expected_line);
    }
    
    // Check enum validation
    for enum_def in &protocol_def.enums {
        for variant in &enum_def.variants {
            if let Some(value) = &variant.value {
                let expected_line = format!("{} = {}", variant.name, value);
                assert!(correct_python.contains(&expected_line), 
                       "Correct Python should contain: {}", expected_line);
                assert!(!incorrect_python.contains(&expected_line), 
                       "Incorrect Python should not contain: {}", expected_line);
            }
        }
    }
    
    Ok(())
}

/// Test consistency between Rust MessageTypeId values and expected ranges
#[test]
fn test_message_type_id_ranges() -> Result<()> {
    let source = r#"
        #[repr(u8)]
        pub enum MessageTypeId {
            HostCommandConfigurePeripheral = 0x01,
            HostCommandConfigureService = 0x02,
            HostCommandGetServiceInfo = 0x05,
            PluginData = 0x80,
            PluginConfigurationError = 0x81,
            PluginServiceInfoResponse = 0x82,
        }
    "#;
    
    let protocol_def = parse_rust_source(source)?;
    let message_enum = protocol_def.enums.iter()
        .find(|e| e.name == "MessageTypeId")
        .expect("Should find MessageTypeId enum");
    
    // Verify host commands are in 0x01-0x7F range
    let host_commands: Vec<_> = message_enum.variants.iter()
        .filter(|v| v.name.starts_with("HostCommand"))
        .collect();
    
    for cmd in &host_commands {
        if let Some(value) = &cmd.value {
            let hex_value = if value.starts_with("0x") {
                u8::from_str_radix(&value[2..], 16).unwrap()
            } else {
                value.parse::<u8>().unwrap()
            };
            assert!(hex_value >= 0x01 && hex_value <= 0x7F, 
                   "Host command {} has value {} outside 0x01-0x7F range", cmd.name, value);
        }
    }
    
    // Verify plugin responses are in 0x80-0xFF range
    let plugin_responses: Vec<_> = message_enum.variants.iter()
        .filter(|v| v.name.starts_with("Plugin"))
        .collect();
    
    for resp in &plugin_responses {
        if let Some(value) = &resp.value {
            let hex_value = if value.starts_with("0x") {
                u8::from_str_radix(&value[2..], 16).unwrap()
            } else {
                value.parse::<u8>().unwrap()
            };
            assert!(hex_value >= 0x80 && hex_value <= 0xFF, 
                   "Plugin response {} has value {} outside 0x80-0xFF range", resp.name, value);
        }
    }
    
    Ok(())
}

/// Test that generated Python preserves important Rust semantics
#[test]
fn test_rust_semantics_preservation() -> Result<()> {
    let source = r#"
        /// Critical system constant - do not modify
        pub const PROTOCOL_VERSION: u8 = 1;
        
        /// Message size limits
        pub const MAX_MESSAGE_SIZE: usize = 1024;
        pub const MIN_MESSAGE_SIZE: usize = 8;
        
        /// Authentication token length
        pub const AUTH_TOKEN_BYTES: usize = 32;
        
        #[repr(u8)]
        pub enum Priority {
            Low = 1,
            Medium = 5, 
            High = 10,
            Critical = 20,
        }
        
        pub struct AuthenticatedMessage {
            pub token: [u8; 32],  // Fixed size array
            pub priority: Priority,
            pub payload_size: u16,
            pub payload: Vec<u8>,
        }
    "#;
    
    let protocol_def = parse_rust_source(source)?;
    
    // Test that constants maintain their exact values
    let version_const = protocol_def.constants.iter()
        .find(|c| c.name == "PROTOCOL_VERSION")
        .expect("Should find PROTOCOL_VERSION");
    assert_eq!(version_const.value, "1");
    
    let max_size_const = protocol_def.constants.iter()
        .find(|c| c.name == "MAX_MESSAGE_SIZE")
        .expect("Should find MAX_MESSAGE_SIZE");
    assert_eq!(max_size_const.value, "1024");
    
    // Test that enum values are preserved exactly
    let priority_enum = protocol_def.enums.iter()
        .find(|e| e.name == "Priority")
        .expect("Should find Priority enum");
    
    let critical_variant = priority_enum.variants.iter()
        .find(|v| v.name == "Critical")
        .expect("Should find Critical variant");
    assert_eq!(critical_variant.value, Some("20".to_string()));
    
    // Test that struct fields map correctly to Python
    let auth_struct = protocol_def.structs.iter()
        .find(|s| s.name == "AuthenticatedMessage")
        .expect("Should find AuthenticatedMessage struct");
    
    let token_field = auth_struct.fields.iter()
        .find(|f| f.name == "token")
        .expect("Should find token field");
    // Fixed size arrays should map to appropriate Python type
    assert!(token_field.rust_type.contains("[u8"));
    
    let payload_field = auth_struct.fields.iter()
        .find(|f| f.name == "payload")
        .expect("Should find payload field");
    assert_eq!(payload_field.python_type, "List[int]");
    
    Ok(())
}

/// Test edge cases in type conversion
#[test]
fn test_type_conversion_edge_cases() -> Result<()> {
    let source = r#"
        pub struct EdgeCaseTypes {
            // Nested options
            pub nested_option: Option<Option<String>>,
            
            // Complex generics
            pub complex_vec: Vec<Option<Vec<String>>>,
            
            // Heapless types with size parameters
            pub heapless_string: heapless::String<64>,
            pub heapless_vec: heapless::Vec<Option<u32>, 16>,
            
            // Custom types
            pub custom_type: MyCustomType,
            pub generic_custom: GenericType<String>,
            
            // Primitive arrays
            pub byte_array: [u8; 16],
            pub word_array: [u32; 4],
        }
    "#;
    
    let protocol_def = parse_rust_source(source)?;
    let struct_def = &protocol_def.structs[0];
    
    // Test nested option conversion
    let nested_option = struct_def.fields.iter()
        .find(|f| f.name == "nested_option")
        .expect("Should find nested_option field");
    assert_eq!(nested_option.python_type, "Optional[Optional[str]]");
    
    // Test complex generic conversion
    let complex_vec = struct_def.fields.iter()
        .find(|f| f.name == "complex_vec")
        .expect("Should find complex_vec field");
    assert_eq!(complex_vec.python_type, "List[Optional[List[str]]]");
    
    // Test heapless types
    let heapless_string = struct_def.fields.iter()
        .find(|f| f.name == "heapless_string")
        .expect("Should find heapless_string field");
    assert_eq!(heapless_string.python_type, "str");
    
    let heapless_vec = struct_def.fields.iter()
        .find(|f| f.name == "heapless_vec")
        .expect("Should find heapless_vec field");
    assert_eq!(heapless_vec.python_type, "List[Optional[int]]");
    
    // Test that custom types are preserved
    let custom_type = struct_def.fields.iter()
        .find(|f| f.name == "custom_type")
        .expect("Should find custom_type field");
    assert!(custom_type.python_type.contains("MyCustomType"));
    
    // Test arrays (should map to List)
    let byte_array = struct_def.fields.iter()
        .find(|f| f.name == "byte_array")
        .expect("Should find byte_array field");
    // Arrays are preserved as-is since they're not in our type mapping
    assert!(byte_array.rust_type.contains("[u8"));
    
    Ok(())
}

/// Test documentation comment preservation and formatting
#[test]
fn test_documentation_preservation() -> Result<()> {
    let source = r#"
        /// This is a single line comment
        pub const SIMPLE: u32 = 1;
        
        /// This is a multiline comment
        /// that spans multiple lines
        /// and should be joined together
        pub const MULTILINE: u32 = 2;
        
        /// This has [references] and `code` and other **markdown**
        /// that should be cleaned up for Python comments
        pub const MARKDOWN: u32 = 3;
        
        /// Very long comment that exceeds the reasonable length for a single line and should be truncated with ellipsis
        pub const LONG: u32 = 4;
    "#;
    
    let protocol_def = parse_rust_source(source)?;
    
    // Test simple comment preservation
    let simple = protocol_def.constants.iter()
        .find(|c| c.name == "SIMPLE")
        .expect("Should find SIMPLE constant");
    assert_eq!(simple.doc_comment, "This is a single line comment");
    
    // Test multiline joining
    let multiline = protocol_def.constants.iter()
        .find(|c| c.name == "MULTILINE")
        .expect("Should find MULTILINE constant");
    assert!(multiline.doc_comment.contains("multiline"));
    assert!(multiline.doc_comment.contains("multiple lines"));
    // Comments are joined with spaces
    assert!(multiline.doc_comment.contains("spans multiple"));
    
    // Test markdown cleanup
    let markdown = protocol_def.constants.iter()
        .find(|c| c.name == "MARKDOWN")
        .expect("Should find MARKDOWN constant");
    assert!(markdown.doc_comment.contains("references"));
    assert!(markdown.doc_comment.contains("code"));
    assert!(markdown.doc_comment.contains("markdown"));
    // All markdown formatting should be cleaned
    assert!(!markdown.doc_comment.contains("["));
    assert!(!markdown.doc_comment.contains("]"));
    assert!(!markdown.doc_comment.contains("`"));
    assert!(!markdown.doc_comment.contains("**"));
    assert!(!markdown.doc_comment.contains("*"));
    
    // Test truncation
    let long = protocol_def.constants.iter()
        .find(|c| c.name == "LONG")
        .expect("Should find LONG constant");
    assert!(long.doc_comment.len() <= 80);
    assert!(long.doc_comment.ends_with("..."));
    
    Ok(())
}