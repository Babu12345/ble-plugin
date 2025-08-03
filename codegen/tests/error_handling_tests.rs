//! Error handling tests for the codegen system
//!
//! These tests verify that the system handles various error conditions gracefully.

use anyhow::Result;
use codegen::parse_rust_source;

/// Test handling of malformed Rust syntax
#[test]
fn test_malformed_rust_syntax() {
    let malformed_sources = vec![
        // Unclosed braces
        "pub const TEST: u32 = {",
        "pub enum Test { Variant",
        "pub struct Test { field: u32",
        
        // Invalid syntax
        "pub const = 42;",
        "pub enum { }",
        "pub struct { }",
        
        // Mixed valid/invalid
        "pub const VALID: u32 = 1; pub const INVALID = ;",
        
        // Completely invalid
        "this is not rust code at all!",
        "}{][{[}",
        "",  // Empty is actually valid
    ];
    
    for (i, source) in malformed_sources.iter().enumerate() {
        let result = parse_rust_source(source);
        
        if i == malformed_sources.len() - 1 {
            // Empty source should succeed
            assert!(result.is_ok(), "Empty source should be valid");
        } else {
            // All others should fail
            assert!(result.is_err(), "Malformed source {} should fail: {}", i, source);
            
            // Error should contain useful information
            let error = result.unwrap_err();
            let error_msg = error.to_string();
            assert!(error_msg.contains("Failed to parse") || error_msg.contains("parse"), 
                   "Error should mention parsing: {}", error_msg);
        }
    }
}

/// Test handling of unsupported Rust constructs
#[test]
fn test_unsupported_constructs() -> Result<()> {
    let source_with_unsupported = r#"
        // These should be ignored, not cause errors
        use std::collections::HashMap;
        
        pub const VALID_CONST: u32 = 42;
        
        pub fn some_function() {
            println!("This should be ignored");
        }
        
        impl SomeStruct {
            pub fn method(&self) {}
        }
        
        pub enum ValidEnum {
            Variant1,
            Variant2,
        }
        
        pub trait SomeTrait {
            fn trait_method(&self);
        }
        
        pub struct ValidStruct {
            pub field: u32,
        }
        
        pub mod some_module {
            pub const INNER_CONST: u32 = 1;
        }
        
        macro_rules! some_macro {
            () => {};
        }
    "#;
    
    let result = parse_rust_source(source_with_unsupported)?;
    
    // Should extract supported items (including those in modules)
    assert_eq!(result.constants.len(), 2);
    
    let constant_names: Vec<_> = result.constants.iter().map(|c| &c.name).collect();
    assert!(constant_names.contains(&&"VALID_CONST".to_string()));
    assert!(constant_names.contains(&&"INNER_CONST".to_string()));
    
    assert_eq!(result.enums.len(), 1);
    assert_eq!(result.enums[0].name, "ValidEnum");
    
    assert_eq!(result.structs.len(), 1);
    assert_eq!(result.structs[0].name, "ValidStruct");
    
    Ok(())
}

/// Test handling of complex type expressions that might cause issues
#[test]
fn test_complex_type_expressions() -> Result<()> {
    let source = r#"
        // Complex const expressions
        pub const CALCULATED: usize = 2 + 3 * 4;
        pub const REFERENCE_CONST: usize = CALCULATED;
        pub const FUNCTION_CALL: usize = std::mem::size_of::<u32>();
        
        // Complex type expressions
        pub struct ComplexTypes {
            // These should not crash the parser
            pub function_ptr: fn() -> u32,
            pub complex_generic: std::collections::HashMap<String, Vec<Option<u32>>>,
            pub lifetime_param: &'static str,
            pub trait_object: Box<dyn std::fmt::Display>,
            pub closure_type: Box<dyn Fn() -> u32>,
        }
        
        // Enum with complex discriminants
        pub enum ComplexEnum {
            Calculated = 2 + 3,
            Reference = ComplexEnum::Calculated as isize + 1,
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    // Should extract what it can without crashing
    assert!(result.constants.len() >= 1);
    assert!(result.structs.len() >= 1); 
    assert!(result.enums.len() >= 1);
    
    // Complex expressions should be preserved as strings
    let calculated = result.constants.iter()
        .find(|c| c.name == "CALCULATED");
    if let Some(calc) = calculated {
        assert!(!calc.value.is_empty());
    }
    
    Ok(())
}

/// Test that the system handles very large files gracefully
#[test]
fn test_large_file_handling() -> Result<()> {
    let mut large_source = String::new();
    
    // Generate a large number of constants
    for i in 0..1000 {
        large_source.push_str(&format!("/// Constant number {}\n", i));
        large_source.push_str(&format!("pub const CONST_{}: u32 = {};\n\n", i, i));
    }
    
    // Generate many enum variants
    large_source.push_str("/// Large enum\npub enum LargeEnum {\n");
    for i in 0..500 {
        large_source.push_str(&format!("    /// Variant {}\n", i));
        large_source.push_str(&format!("    Variant{} = {},\n", i, i));
    }
    large_source.push_str("}\n\n");
    
    // Generate many struct fields
    large_source.push_str("/// Large struct\npub struct LargeStruct {\n");
    for i in 0..200 {
        large_source.push_str(&format!("    /// Field {}\n", i));
        large_source.push_str(&format!("    pub field_{}: u32,\n", i));
    }
    large_source.push_str("}\n");
    
    let result = parse_rust_source(&large_source)?;
    
    // Should handle large files without issues
    assert_eq!(result.constants.len(), 1000);
    assert_eq!(result.enums.len(), 1);
    assert_eq!(result.enums[0].variants.len(), 500);
    assert_eq!(result.structs.len(), 1);
    assert_eq!(result.structs[0].fields.len(), 200);
    
    Ok(())
}

/// Test error handling in type conversion edge cases
#[test]
fn test_type_conversion_error_cases() -> Result<()> {
    let source = r#"
        pub struct EdgeCaseStruct {
            // These shouldn't crash the type converter
            pub weird_generic: SomeType<A, B, C, D>,
            pub empty_angle: SomeType<>,
            pub nested_weird: Vec<HashMap<String, SomeCustomType<T>>>,
            pub reference: &'static SomeType,
            pub mutable_ref: &'static mut SomeType,
            pub raw_pointer: *const u8,
            pub mut_raw_pointer: *mut u8,
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    assert_eq!(result.structs.len(), 1);
    let struct_def = &result.structs[0];
    
    // All fields should be parsed without crashing
    assert_eq!(struct_def.fields.len(), 7);
    
    // Python types should be generated (even if they're not perfect)
    for field in &struct_def.fields {
        assert!(!field.python_type.is_empty(), 
               "Field {} should have a Python type", field.name);
    }
    
    Ok(())
}

/// Test handling of files with encoding issues (simulated)
#[test]
fn test_unicode_and_encoding() -> Result<()> {
    let source = r#"
        /// Unicode in comments: 测试, Москва, العربية
        pub const UNICODE_DOC: u32 = 42;
        
        /// Emoji test: 🚀 🎉 ✨
        pub const EMOJI_DOC: u32 = 43;
        
        pub struct UnicodeStruct {
            /// Field with unicode: café
            pub unicode_field: String,
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    // Should handle unicode in documentation
    assert!(!result.constants.is_empty());
    let unicode_const = result.constants.iter()
        .find(|c| c.name == "UNICODE_DOC")
        .expect("Should find unicode const");
    
    // Documentation should be preserved (potentially cleaned up)
    assert!(!unicode_const.doc_comment.is_empty());
    
    Ok(())
}

/// Test graceful degradation when optional information is missing
#[test]
fn test_missing_optional_information() -> Result<()> {
    let source = r#"
        // No doc comments
        pub const NO_DOC: u32 = 1;
        
        // Enum without repr
        pub enum NoRepr {
            Variant1,
            Variant2,
        }
        
        // Enum without explicit values
        pub enum NoValues {
            First,
            Second,
            Third,
        }
        
        // Struct without field docs
        pub struct NoFieldDocs {
            pub field1: u32,
            pub field2: String,
        }
    "#;
    
    let result = parse_rust_source(source)?;
    
    // Should handle missing optional information gracefully
    let no_doc_const = result.constants.iter()
        .find(|c| c.name == "NO_DOC")
        .expect("Should find NO_DOC");
    assert_eq!(no_doc_const.doc_comment, "");
    
    let no_repr_enum = result.enums.iter()
        .find(|e| e.name == "NoRepr")
        .expect("Should find NoRepr");
    assert_eq!(no_repr_enum.repr, None);
    
    let no_values_enum = result.enums.iter()
        .find(|e| e.name == "NoValues")
        .expect("Should find NoValues");
    for variant in &no_values_enum.variants {
        assert_eq!(variant.value, None);
    }
    
    let no_docs_struct = result.structs.iter()
        .find(|s| s.name == "NoFieldDocs")
        .expect("Should find NoFieldDocs");
    for field in &no_docs_struct.fields {
        assert_eq!(field.doc_comment, "");
    }
    
    Ok(())
}