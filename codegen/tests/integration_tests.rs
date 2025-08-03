//! Integration tests for the codegen system
//! 
//! These tests verify the complete pipeline from Rust source parsing to Python code generation.

use anyhow::Result;
use codegen::{parse_rust_source, ProtocolDef};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test the complete parsing of the actual protocol library
#[test]
fn test_parse_actual_protocol_library() -> Result<()> {
    let protocol_path = PathBuf::from("../protocol/src");
    
    // Parse io.rs
    let io_path = protocol_path.join("io.rs");
    if io_path.exists() {
        let io_source = fs::read_to_string(&io_path)?;
        let io_def = parse_rust_source(&io_source)?;
        
        // Should find MessageTypeId enum
        assert!(io_def.enums.iter().any(|e| e.name == "MessageTypeId"));
        
        // Should find constants like MESSAGE_MAGIC
        assert!(io_def.constants.iter().any(|c| c.name == "MESSAGE_MAGIC"));
    }
    
    Ok(())
}

/// Test parsing a complete protocol definition
#[test]  
fn test_parse_complete_protocol() -> Result<()> {
    let source = r#"
        /// Magic number for protocol validation
        pub const MESSAGE_MAGIC: u16 = 0xDEAD;
        
        /// Default packet size for communication
        pub const DEFAULT_PACKET_SIZE: usize = 256;
        
        /// Message type identifiers
        #[repr(u8)]
        pub enum MessageTypeId {
            /// Host command to configure peripheral
            HostCommandConfigurePeripheral = 0x01,
            /// Plugin data response
            PluginData = 0x80,
        }
        
        /// Configuration message for BLE peripheral
        pub struct HostCommandConfigurePeripheral {
            /// The device name
            pub name: String,
            /// The service UUID
            pub uuid: String,
            /// Maximum connections allowed
            pub max_connections: Option<u8>,
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    // Verify constants
    assert_eq!(result.constants.len(), 2);
    assert!(result.constants.iter().any(|c| c.name == "MESSAGE_MAGIC" && c.value == "0xDEAD"));
    assert!(result.constants.iter().any(|c| c.name == "DEFAULT_PACKET_SIZE" && c.value == "256"));
    
    // Verify enum
    assert_eq!(result.enums.len(), 1);
    let enum_def = &result.enums[0];
    assert_eq!(enum_def.name, "MessageTypeId");
    assert_eq!(enum_def.variants.len(), 2);
    assert_eq!(enum_def.variants[0].name, "HostCommandConfigurePeripheral");
    assert_eq!(enum_def.variants[0].value, Some("0x01".to_string()));
    assert_eq!(enum_def.variants[1].name, "PluginData");
    assert_eq!(enum_def.variants[1].value, Some("0x80".to_string()));
    
    // Verify struct
    assert_eq!(result.structs.len(), 1);
    let struct_def = &result.structs[0];
    assert_eq!(struct_def.name, "HostCommandConfigurePeripheral");
    assert_eq!(struct_def.fields.len(), 3);
    
    // Check field types and Python mappings
    assert_eq!(struct_def.fields[0].name, "name");
    assert_eq!(struct_def.fields[0].python_type, "str");
    assert!(!struct_def.fields[0].is_optional);
    
    assert_eq!(struct_def.fields[1].name, "uuid"); 
    assert_eq!(struct_def.fields[1].python_type, "str");
    assert!(!struct_def.fields[1].is_optional);
    
    assert_eq!(struct_def.fields[2].name, "max_connections");
    assert_eq!(struct_def.fields[2].python_type, "Optional[int]");
    assert!(struct_def.fields[2].is_optional);
    
    Ok(())
}

/// Test error handling for malformed Rust code
#[test]
fn test_error_handling_invalid_syntax() {
    let invalid_sources = vec![
        "pub const BROKEN = ;",  // Missing value
        "pub enum { Broken }",   // Missing enum name
        "pub struct { broken: }", // Missing struct name and field type
        "fn unclosed_brace() {", // Unclosed brace
        "this is not rust code at all",
    ];
    
    for source in invalid_sources {
        let result = parse_rust_source(source);
        assert!(result.is_err(), "Should fail to parse: {}", source);
    }
}

/// Test parsing with mixed visibility and attributes
#[test]
fn test_parse_mixed_visibility_and_attributes() -> Result<()> {
    let source = r#"
        // Private constant should be ignored in most contexts but still parsed
        const PRIVATE_CONST: u32 = 1;
        
        /// Public constant with docs
        pub const PUBLIC_CONST: u32 = 2;
        
        // Private enum
        enum PrivateEnum {
            Variant1,
        }
        
        /// Public enum with repr and docs
        #[repr(u8)]
        #[derive(Debug, Clone)]
        pub enum PublicEnum {
            /// First variant
            #[deprecated]
            Variant1 = 10,
            /// Second variant
            Variant2 = 20,
        }
        
        // Private struct
        struct PrivateStruct {
            field: u32,
        }
        
        /// Public struct with attributes
        #[derive(Debug, Clone, Serialize)]
        pub struct PublicStruct {
            /// A public field
            pub public_field: String,
            /// A private field
            private_field: u32,
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    // Should parse only public items (private items are filtered out)
    assert_eq!(result.constants.len(), 1); // Only PUBLIC_CONST
    assert_eq!(result.enums.len(), 1);     // Only PublicEnum 
    assert_eq!(result.structs.len(), 1);   // Only PublicStruct
    
    // Check that attributes are preserved where relevant
    let public_enum = result.enums.iter().find(|e| e.name == "PublicEnum").unwrap();
    assert_eq!(public_enum.repr, Some("u8".to_string()));
    
    Ok(())
}

/// Test parsing complex generic types
#[test]
fn test_parse_complex_generic_types() -> Result<()> {
    let source = r#"
        /// Complex struct with various generic types
        pub struct ComplexMessage {
            /// Simple vector
            pub simple_vec: Vec<u8>,
            /// Nested generics
            pub nested: Vec<Option<String>>,
            /// Heapless types
            pub heapless_string: heapless::String<32>,
            pub heapless_vec: heapless::Vec<u16, 10>,
            /// Multiple option levels
            pub complex_option: Option<Option<Vec<String>>>,
            /// Custom generic type
            pub custom: HashMap<String, Vec<u32>>,
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    assert_eq!(result.structs.len(), 1);
    let struct_def = &result.structs[0];
    assert_eq!(struct_def.fields.len(), 6);
    
    // Check Python type mappings for complex generics
    assert_eq!(struct_def.fields[0].python_type, "List[int]");
    assert_eq!(struct_def.fields[1].python_type, "List[Optional[str]]");
    assert_eq!(struct_def.fields[2].python_type, "str");
    assert_eq!(struct_def.fields[3].python_type, "List[int]");
    assert_eq!(struct_def.fields[4].python_type, "Optional[Optional[List[str]]]");
    // Custom types are preserved as-is (with spaces from quote formatting)
    assert!(struct_def.fields[5].python_type.contains("HashMap"));
    assert!(struct_def.fields[5].python_type.contains("String"));
    assert!(struct_def.fields[5].python_type.contains("u32"));
    
    Ok(())
}

/// Test documentation extraction and formatting
#[test]
fn test_documentation_extraction_and_formatting() -> Result<()> {
    let source = r#"
        /// This is a simple doc comment
        pub const SIMPLE_DOC: u32 = 1;
        
        /// This is a multiline
        /// documentation comment
        /// with multiple lines
        pub const MULTILINE_DOC: u32 = 2;
        
        /// This has [`code references`] and ```rust code blocks``` 
        /// and # headers that should be cleaned up
        pub const MARKDOWN_DOC: u32 = 3;
        
        /// This is an extremely long documentation comment that should be truncated because it exceeds the reasonable length limit for single-line Python comments and needs to be shortened
        pub const LONG_DOC: u32 = 4;
    "#;
    
    let result = parse_rust_source(source)?;
    
    assert_eq!(result.constants.len(), 4);
    
    // Simple doc should be preserved
    let simple = result.constants.iter().find(|c| c.name == "SIMPLE_DOC").unwrap();
    assert_eq!(simple.doc_comment, "This is a simple doc comment");
    
    // Multiline should be joined
    let multiline = result.constants.iter().find(|c| c.name == "MULTILINE_DOC").unwrap();
    assert!(multiline.doc_comment.contains("multiline"));
    assert!(multiline.doc_comment.contains("multiple lines"));
    
    // Markdown should be cleaned
    let markdown = result.constants.iter().find(|c| c.name == "MARKDOWN_DOC").unwrap();
    assert!(markdown.doc_comment.contains("code references"));
    assert!(!markdown.doc_comment.contains("[`"));
    assert!(!markdown.doc_comment.contains("`]"));
    assert!(!markdown.doc_comment.contains("```"));
    assert!(!markdown.doc_comment.contains("# "));
    
    // Long doc should be truncated
    let long = result.constants.iter().find(|c| c.name == "LONG_DOC").unwrap();
    assert!(long.doc_comment.len() <= 80);
    assert!(long.doc_comment.ends_with("..."));
    
    Ok(())
}

/// Test integration with the actual protocol files if they exist
#[test]
fn test_integration_with_real_protocol_files() -> Result<()> {
    let protocol_src = PathBuf::from("../protocol/src");
    
    if !protocol_src.exists() {
        println!("Skipping integration test - protocol source not found");
        return Ok(());
    }
    
    // Test parsing io.rs
    let io_path = protocol_src.join("io.rs");
    if io_path.exists() {
        let io_source = fs::read_to_string(&io_path)?;
        let io_result = parse_rust_source(&io_source)?;
        
        // Should find the MessageTypeId enum with correct variant ranges
        if let Some(message_enum) = io_result.enums.iter().find(|e| e.name == "MessageTypeId") {
            // Check that plugin responses start at 0x80
            let plugin_data = message_enum.variants.iter()
                .find(|v| v.name == "PluginData");
            if let Some(plugin_data) = plugin_data {
                assert_eq!(plugin_data.value, Some("0x80".to_string()));
            }
        }
        
        // Should find MESSAGE_MAGIC constant
        assert!(io_result.constants.iter().any(|c| c.name == "MESSAGE_MAGIC"));
    }
    
    // Test parsing lib.rs
    let lib_path = protocol_src.join("lib.rs");
    if lib_path.exists() {
        let lib_source = fs::read_to_string(&lib_path)?;
        let lib_result = parse_rust_source(&lib_source)?;
        
        // Should find DEFAULT_PACKET_SIZE
        assert!(lib_result.constants.iter().any(|c| c.name == "DEFAULT_PACKET_SIZE"));
    }
    
    Ok(())
}

/// Test that generated Python code would be syntactically valid
#[test]
fn test_generated_python_syntax_validity() -> Result<()> {
    // This test would ideally generate Python code and validate its syntax
    // For now, we'll test that the parsing produces valid data structures
    
    let source = r#"
        /// Test constant
        pub const TEST_CONST: u32 = 42;
        
        /// Test enum  
        #[repr(u8)]
        pub enum TestEnum {
            /// First variant
            First = 1,
            /// Second variant
            Second = 2,
        }
        
        /// Test struct
        pub struct TestStruct {
            /// Name field
            pub name: String,
            /// Optional value
            pub value: Option<u32>,
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    // Verify that all extracted data would generate valid Python
    for constant in &result.constants {
        assert!(!constant.name.is_empty());
        assert!(!constant.value.is_empty());
        // Value should be a valid Python literal or identifier
        assert!(constant.value.chars().all(|c| c.is_alphanumeric() || "0x_.\"".contains(c)));
    }
    
    for enum_def in &result.enums {
        assert!(!enum_def.name.is_empty());
        assert!(enum_def.name.chars().all(|c| c.is_alphanumeric()));
        
        for variant in &enum_def.variants {
            assert!(!variant.name.is_empty());
            assert!(variant.name.chars().all(|c| c.is_alphanumeric()));
            
            if let Some(value) = &variant.value {
                assert!(!value.is_empty());
            }
        }
    }
    
    for struct_def in &result.structs {
        assert!(!struct_def.name.is_empty());
        assert!(struct_def.name.chars().all(|c| c.is_alphanumeric()));
        
        for field in &struct_def.fields {
            assert!(!field.name.is_empty());
            assert!(field.name.chars().all(|c| c.is_alphanumeric() || c == '_'));
            assert!(!field.python_type.is_empty());
        }
    }
    
    Ok(())
}