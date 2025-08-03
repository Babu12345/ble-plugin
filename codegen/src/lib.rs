//! Code generation utilities for maintaining consistency between Rust and Python protocol libraries.
//!
//! This crate provides tools to parse the Rust protocol library and generate equivalent Python code,
//! ensuring that constants, enums, structs, and message type IDs remain synchronized.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use syn::{Item, ItemConst, ItemEnum, ItemStruct};

/// Represents a constant definition extracted from Rust code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstantDef {
    pub name: String,
    pub value: String,
    pub doc_comment: String,
    pub rust_type: String,
}

/// Represents an enum variant extracted from Rust code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<String>,
    pub doc_comment: String,
}

/// Represents an enum definition extracted from Rust code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub doc_comment: String,
    pub repr: Option<String>,
}

/// Represents a struct field extracted from Rust code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructField {
    pub name: String,
    pub rust_type: String,
    pub python_type: String,
    pub doc_comment: String,
    pub is_optional: bool,
}

/// Represents a struct definition extracted from Rust code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
    pub doc_comment: String,
    pub message_type_id: Option<String>,
}

/// Complete protocol definition extracted from Rust code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtocolDef {
    pub constants: Vec<ConstantDef>,
    pub enums: Vec<EnumDef>,
    pub structs: Vec<StructDef>,
}

/// Maps Rust types to Python equivalents
pub fn rust_type_to_python(rust_type: &str) -> String {
    // Normalize whitespace for consistent matching
    let normalized = rust_type.replace(" ", "");

    match normalized.as_str() {
        "u8" => "attrs2bin.U8".to_string(),
        "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => "int".to_string(),
        "f32" | "f64" => "float".to_string(),
        "bool" => "bool".to_string(),
        "String" => "str".to_string(),
        "Uuid" => "str".to_string(),
        t if t.starts_with("Vec<") || t.starts_with("heapless::Vec<") => {
            // For heapless::Vec<T, N>, we only care about the first type parameter T
            let inner_type = extract_generic_type(t)
                .map(|inner| {
                    // If there are multiple type parameters (e.g., "u16, 10"), take only the first
                    inner.split(',').next().unwrap_or(&inner).trim().to_string()
                })
                .unwrap_or_else(|| "Any".to_string());
            format!("List[{}]", rust_type_to_python(&inner_type))
        }
        t if t.starts_with("String<") || t.starts_with("heapless::String<") => "str".to_string(),
        t if t.starts_with("Option<") => {
            format!(
                "Optional[{}]",
                extract_generic_type(t)
                    .map(|inner| rust_type_to_python(&inner))
                    .unwrap_or_else(|| "Any".to_string())
            )
        }
        t if t.contains("&") && t.contains("[u8]") => "bytes".to_string(), // Slice references &[u8]
        t if t.contains("[u8") => "List[attrs2bin.U8]".to_string(),                 // Arrays like [u8; 6]
        _ => rust_type.to_string(), // For custom types, keep as-is
    }
}

/// Extracts the inner type from a generic type like Vec<T> or Option<T>
fn extract_generic_type(type_str: &str) -> Option<String> {
    let start = type_str.find('<')?;
    let end = type_str.rfind('>')?;
    Some(type_str[start + 1..end].to_string())
}

/// Check if an item has public visibility
fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

/// Extracts documentation comments from attributes and formats them for Python
pub fn extract_doc_comment(attrs: &[syn::Attribute]) -> String {
    let doc_lines: Vec<String> = attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                match &attr.meta {
                    syn::Meta::NameValue(meta) if meta.path.is_ident("doc") => {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }) = &meta.value
                        {
                            Some(lit_str.value().trim().to_string())
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect();

    if doc_lines.is_empty() {
        return String::new();
    }

    // Format as a single line summary for Python comments
    let combined = doc_lines.join(" ");

    // Remove markdown-style formatting and references
    let cleaned = combined
        .replace("[`", "")
        .replace("`]", "")
        .replace("```rust", "")
        .replace("```", "")
        .replace("**", "") // Remove bold
        .replace("*", "") // Remove italic
        .replace("# ", "")
        .replace("[", "") // Remove bare brackets
        .replace("]", "")
        .replace("`", "") // Remove bare backticks
        .trim()
        .to_string();

    // Truncate if too long for a single comment line
    if cleaned.len() > 80 {
        format!("{}...", &cleaned[..77])
    } else {
        cleaned
    }
}

/// Parse Rust source code and extract protocol definitions
pub fn parse_rust_source(source: &str) -> Result<ProtocolDef> {
    let syntax_tree = syn::parse_file(source).context("Failed to parse Rust source code")?;

    let mut constants = Vec::new();
    let mut enums = Vec::new();
    let mut structs = Vec::new();

    parse_items(&syntax_tree.items, &mut constants, &mut enums, &mut structs)?;

    Ok(ProtocolDef {
        constants,
        enums,
        structs,
    })
}

/// Recursively parse items from a list, handling nested modules
fn parse_items(
    items: &[Item],
    constants: &mut Vec<ConstantDef>,
    enums: &mut Vec<EnumDef>,
    structs: &mut Vec<StructDef>,
) -> Result<()> {
    for item in items {
        match item {
            Item::Const(const_item) => {
                if let Some(constant) = extract_constant(&const_item)? {
                    constants.push(constant);
                }
            }
            Item::Enum(enum_item) => {
                if let Some(enum_def) = extract_enum(&enum_item)? {
                    enums.push(enum_def);
                }
            }
            Item::Struct(struct_item) => {
                if let Some(struct_def) = extract_struct(&struct_item)? {
                    structs.push(struct_def);
                }
            }
            Item::Mod(mod_item) => {
                // Recursively parse items inside modules
                if let Some((_, mod_items)) = &mod_item.content {
                    parse_items(mod_items, constants, enums, structs)?;
                }
            }
            _ => {} // Ignore other items
        }
    }
    Ok(())
}

/// Extract constant definition from syn::ItemConst
fn extract_constant(const_item: &ItemConst) -> Result<Option<ConstantDef>> {
    // Only extract public constants
    if !is_public(&const_item.vis) {
        return Ok(None);
    }

    let name = const_item.ident.to_string();
    let doc_comment = extract_doc_comment(&const_item.attrs);

    // Extract the type
    let rust_type = {
        let ty = &const_item.ty;
        quote::quote!(#ty).to_string()
    };

    // Extract the value more carefully, preserving hex notation when possible
    let value = match &*const_item.expr {
        syn::Expr::Lit(expr_lit) => {
            match &expr_lit.lit {
                syn::Lit::Int(lit_int) => {
                    let token = lit_int.token();
                    let token_str = token.to_string();
                    // Preserve hex notation if present
                    if token_str.starts_with("0x") || token_str.starts_with("0X") {
                        token_str
                    } else {
                        lit_int.base10_digits().to_string()
                    }
                }
                syn::Lit::Str(lit_str) => format!("\"{}\"", lit_str.value()),
                syn::Lit::Bool(lit_bool) => lit_bool.value.to_string(),
                _ => quote::quote!(#const_item.expr).to_string(),
            }
        }
        syn::Expr::Path(expr_path) => {
            // Handle constants like references to other constants
            if let Some(segment) = expr_path.path.segments.last() {
                segment.ident.to_string()
            } else {
                quote::quote!(#const_item.expr).to_string()
            }
        }
        _ => {
            // For other expressions, just serialize them as-is
            quote::quote!(#const_item.expr).to_string()
        }
    };

    Ok(Some(ConstantDef {
        name,
        value,
        doc_comment,
        rust_type,
    }))
}

/// Extract enum definition from syn::ItemEnum
fn extract_enum(enum_item: &ItemEnum) -> Result<Option<EnumDef>> {
    // Only extract public enums
    if !is_public(&enum_item.vis) {
        return Ok(None);
    }

    let name = enum_item.ident.to_string();
    let doc_comment = extract_doc_comment(&enum_item.attrs);

    // Extract repr attribute if present
    let repr = enum_item.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("repr") {
            match &attr.meta {
                syn::Meta::List(meta_list) => {
                    // Extract just the inner token content for repr
                    Some(meta_list.tokens.to_string())
                }
                _ => None,
            }
        } else {
            None
        }
    });

    let mut variants = Vec::new();
    for variant in &enum_item.variants {
        let variant_name = variant.ident.to_string();
        let variant_doc = extract_doc_comment(&variant.attrs);

        // Extract discriminant value if present
        let value = variant
            .discriminant
            .as_ref()
            .map(|(_, expr)| quote::quote!(#expr).to_string());

        variants.push(EnumVariant {
            name: variant_name,
            value,
            doc_comment: variant_doc,
        });
    }

    Ok(Some(EnumDef {
        name,
        variants,
        doc_comment,
        repr,
    }))
}

/// Extract struct definition from syn::ItemStruct
fn extract_struct(struct_item: &ItemStruct) -> Result<Option<StructDef>> {
    // Only extract public structs
    if !is_public(&struct_item.vis) {
        return Ok(None);
    }

    let name = struct_item.ident.to_string();
    let doc_comment = extract_doc_comment(&struct_item.attrs);

    let mut fields = Vec::new();

    if let syn::Fields::Named(named_fields) = &struct_item.fields {
        for field in &named_fields.named {
            if let Some(field_name) = &field.ident {
                let field_name = field_name.to_string();
                let field_doc = extract_doc_comment(&field.attrs);
                let rust_type = {
                    let ty = &field.ty;
                    quote::quote!(#ty).to_string()
                };
                let python_type = rust_type_to_python(&rust_type);
                let is_optional = rust_type.replace(" ", "").starts_with("Option<");

                fields.push(StructField {
                    name: field_name,
                    rust_type,
                    python_type,
                    doc_comment: field_doc,
                    is_optional,
                });
            }
        }
    }

    // TODO: Extract message_type_id from MessageType impl if present
    let message_type_id = None;

    Ok(Some(StructDef {
        name,
        fields,
        doc_comment,
        message_type_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_type_to_python_primitives() {
        assert_eq!(rust_type_to_python("u8"), "attrs2bin.U8");
        assert_eq!(rust_type_to_python("u16"), "int");
        assert_eq!(rust_type_to_python("u32"), "int");
        assert_eq!(rust_type_to_python("u64"), "int");
        assert_eq!(rust_type_to_python("i8"), "int");
        assert_eq!(rust_type_to_python("i16"), "int");
        assert_eq!(rust_type_to_python("i32"), "int");
        assert_eq!(rust_type_to_python("i64"), "int");
        assert_eq!(rust_type_to_python("f32"), "float");
        assert_eq!(rust_type_to_python("f64"), "float");
        assert_eq!(rust_type_to_python("bool"), "bool");
        assert_eq!(rust_type_to_python("String"), "str");
        assert_eq!(rust_type_to_python("Uuid"), "str");
    }

    #[test]
    fn test_rust_type_to_python_generics() {
        assert_eq!(rust_type_to_python("Vec<u8>"), "List[attrs2bin.U8]");
        assert_eq!(rust_type_to_python("Vec<String>"), "List[str]");
        assert_eq!(rust_type_to_python("heapless::Vec<u16>"), "List[int]");
        assert_eq!(rust_type_to_python("Option<u32>"), "Optional[int]");
        assert_eq!(rust_type_to_python("Option<String>"), "Optional[str]");
        assert_eq!(rust_type_to_python("heapless::String<32>"), "str");
        assert_eq!(rust_type_to_python("String<64>"), "str");
    }

    #[test]
    fn test_rust_type_to_python_custom_types() {
        assert_eq!(rust_type_to_python("CustomType"), "CustomType");
        assert_eq!(rust_type_to_python("MessageTypeId"), "MessageTypeId");
    }

    #[test]
    fn test_extract_generic_type() {
        assert_eq!(extract_generic_type("Vec<u8>"), Some("u8".to_string()));
        assert_eq!(
            extract_generic_type("Option<String>"),
            Some("String".to_string())
        );
        assert_eq!(
            extract_generic_type("heapless::Vec<CustomType>"),
            Some("CustomType".to_string())
        );
        assert_eq!(extract_generic_type("String<32>"), Some("32".to_string()));
        assert_eq!(extract_generic_type("SimpleType"), None);
    }

    #[test]
    fn test_parse_constant_basic() {
        let source = r#"
            /// This is a test constant
            pub const TEST_VALUE: u32 = 42;
        "#;

        let result = parse_rust_source(source).unwrap();
        assert_eq!(result.constants.len(), 1);

        let constant = &result.constants[0];
        assert_eq!(constant.name, "TEST_VALUE");
        assert_eq!(constant.value, "42");
        assert_eq!(constant.rust_type, "u32");
        assert_eq!(constant.doc_comment, "This is a test constant");
    }

    #[test]
    fn test_parse_constant_hex_value() {
        let source = r#"
            /// Magic number constant
            pub const MAGIC: u16 = 0xDEAD;
        "#;

        let result = parse_rust_source(source).unwrap();
        assert_eq!(result.constants.len(), 1);

        let constant = &result.constants[0];
        assert_eq!(constant.name, "MAGIC");
        assert_eq!(constant.value, "0xDEAD");
        assert_eq!(constant.rust_type, "u16");
        assert_eq!(constant.doc_comment, "Magic number constant");
    }

    #[test]
    fn test_parse_constant_string_value() {
        let source = r#"
            /// String constant
            pub const DEFAULT_NAME: &str = "test";
        "#;

        let result = parse_rust_source(source).unwrap();
        assert_eq!(result.constants.len(), 1);

        let constant = &result.constants[0];
        assert_eq!(constant.name, "DEFAULT_NAME");
        assert_eq!(constant.value, "\"test\"");
        assert_eq!(constant.rust_type, "& str");
    }

    #[test]
    fn test_parse_enum_basic() {
        let source = r#"
            /// Test enum for message types
            #[repr(u8)]
            pub enum MessageType {
                /// First variant
                First = 0x01,
                /// Second variant  
                Second = 0x02,
                /// Third variant without explicit value
                Third,
            }
        "#;

        let result = parse_rust_source(source).unwrap();
        assert_eq!(result.enums.len(), 1);

        let enum_def = &result.enums[0];
        assert_eq!(enum_def.name, "MessageType");
        assert_eq!(enum_def.doc_comment, "Test enum for message types");
        assert_eq!(enum_def.repr, Some("u8".to_string()));
        assert_eq!(enum_def.variants.len(), 3);

        assert_eq!(enum_def.variants[0].name, "First");
        assert_eq!(enum_def.variants[0].value, Some("0x01".to_string()));
        assert_eq!(enum_def.variants[0].doc_comment, "First variant");

        assert_eq!(enum_def.variants[1].name, "Second");
        assert_eq!(enum_def.variants[1].value, Some("0x02".to_string()));
        assert_eq!(enum_def.variants[1].doc_comment, "Second variant");

        assert_eq!(enum_def.variants[2].name, "Third");
        assert_eq!(enum_def.variants[2].value, None);
        assert_eq!(
            enum_def.variants[2].doc_comment,
            "Third variant without explicit value"
        );
    }

    #[test]
    fn test_parse_struct_basic() {
        let source = r#"
            /// Test struct for configuration
            pub struct Config {
                /// The name field
                pub name: String,
                /// The port number
                pub port: u16,
                /// Optional description
                pub description: Option<String>,
                /// List of tags
                pub tags: Vec<String>,
            }
        "#;

        let result = parse_rust_source(source).unwrap();
        assert_eq!(result.structs.len(), 1);

        let struct_def = &result.structs[0];
        assert_eq!(struct_def.name, "Config");
        assert_eq!(struct_def.doc_comment, "Test struct for configuration");
        assert_eq!(struct_def.fields.len(), 4);

        assert_eq!(struct_def.fields[0].name, "name");
        assert_eq!(struct_def.fields[0].rust_type, "String");
        assert_eq!(struct_def.fields[0].python_type, "str");
        assert_eq!(struct_def.fields[0].doc_comment, "The name field");
        assert!(!struct_def.fields[0].is_optional);

        assert_eq!(struct_def.fields[1].name, "port");
        assert_eq!(struct_def.fields[1].rust_type, "u16");
        assert_eq!(struct_def.fields[1].python_type, "int");
        assert_eq!(struct_def.fields[1].doc_comment, "The port number");
        assert!(!struct_def.fields[1].is_optional);

        assert_eq!(struct_def.fields[2].name, "description");
        assert!(struct_def.fields[2].rust_type.contains("Option"));
        assert!(struct_def.fields[2].rust_type.contains("String"));
        assert_eq!(struct_def.fields[2].python_type, "Optional[str]");
        assert_eq!(struct_def.fields[2].doc_comment, "Optional description");
        assert!(struct_def.fields[2].is_optional);

        assert_eq!(struct_def.fields[3].name, "tags");
        assert!(struct_def.fields[3].rust_type.contains("Vec"));
        assert!(struct_def.fields[3].rust_type.contains("String"));
        assert_eq!(struct_def.fields[3].python_type, "List[str]");
        assert_eq!(struct_def.fields[3].doc_comment, "List of tags");
        assert!(!struct_def.fields[3].is_optional);
    }

    #[test]
    fn test_extract_doc_comment_markdown_cleanup() {
        // Create a mock attribute with markdown content
        let source = r#"
            /// This has [`code`] and ```rust blocks``` and # headers
            /// Multiple lines with references
            pub const TEST: u32 = 1;
        "#;

        let result = parse_rust_source(source).unwrap();
        let constant = &result.constants[0];

        // Should clean up markdown formatting
        assert!(constant.doc_comment.contains("code"));
        assert!(!constant.doc_comment.contains("[`"));
        assert!(!constant.doc_comment.contains("`]"));
        assert!(!constant.doc_comment.contains("```"));
        assert!(!constant.doc_comment.contains("# "));
    }

    #[test]
    fn test_extract_doc_comment_truncation() {
        let source = r#"
            /// This is a very long documentation comment that should be truncated because it exceeds the maximum length limit for single line comments in the generated Python code
            pub const LONG_DOC: u32 = 1;
        "#;

        let result = parse_rust_source(source).unwrap();
        let constant = &result.constants[0];

        // Should be truncated and end with "..."
        assert!(constant.doc_comment.len() <= 80);
        assert!(constant.doc_comment.ends_with("..."));
    }

    #[test]
    fn test_parse_empty_source() {
        let source = "";
        let result = parse_rust_source(source).unwrap();

        assert_eq!(result.constants.len(), 0);
        assert_eq!(result.enums.len(), 0);
        assert_eq!(result.structs.len(), 0);
    }

    #[test]
    fn test_parse_invalid_rust_syntax() {
        let source = "this is not valid rust code {{{";
        let result = parse_rust_source(source);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_mixed_items() {
        let source = r#"
            use std::collections::HashMap;
            
            /// Test constant
            pub const MAX_SIZE: usize = 100;
            
            /// Test enum
            pub enum Status {
                Active = 1,
                Inactive = 0,
            }
            
            /// Test struct
            pub struct Item {
                pub id: u32,
                pub status: Status,
            }
            
            // This function should be ignored
            pub fn some_function() {}
            
            // This impl should be ignored
            impl Item {
                pub fn new() -> Self {
                    Self { id: 0, status: Status::Active }
                }
            }
        "#;

        let result = parse_rust_source(source).unwrap();

        // Should parse exactly what we expect and ignore other items
        assert_eq!(result.constants.len(), 1);
        assert_eq!(result.enums.len(), 1);
        assert_eq!(result.structs.len(), 1);

        assert_eq!(result.constants[0].name, "MAX_SIZE");
        assert_eq!(result.enums[0].name, "Status");
        assert_eq!(result.structs[0].name, "Item");
    }

    /// Test parsing of nested modules like those in io_types.rs
    #[test]
    fn test_parse_nested_modules() -> Result<()> {
        let source = r#"
            /// Top level constant
            pub const TOP_LEVEL: u32 = 1;
            
            /// Host module
            pub mod host {
                /// Host constant
                pub const HOST_CONST: u32 = 2;
                
                /// Host enum
                pub enum HostEnum {
                    HostVariant = 10,
                }
                
                /// Host struct
                pub struct HostStruct {
                    pub field: u32,
                }
            }
            
            /// Plugin module  
            pub mod plugin {
                /// Plugin constant
                pub const PLUGIN_CONST: u32 = 3;
                
                /// Plugin struct
                pub struct PluginStruct {
                    pub name: String,
                    pub data: Vec<u8>,
                }
            }
        "#;

        let result = parse_rust_source(source)?;

        // Should find constants from both top-level and modules
        assert_eq!(result.constants.len(), 3);
        let constant_names: Vec<_> = result.constants.iter().map(|c| &c.name).collect();
        assert!(constant_names.contains(&&"TOP_LEVEL".to_string()));
        assert!(constant_names.contains(&&"HOST_CONST".to_string()));
        assert!(constant_names.contains(&&"PLUGIN_CONST".to_string()));

        // Should find enums from modules
        assert_eq!(result.enums.len(), 1);
        assert_eq!(result.enums[0].name, "HostEnum");

        // Should find structs from modules
        assert_eq!(result.structs.len(), 2);
        let struct_names: Vec<_> = result.structs.iter().map(|s| &s.name).collect();
        assert!(struct_names.contains(&&"HostStruct".to_string()));
        assert!(struct_names.contains(&&"PluginStruct".to_string()));

        // Check field types are correctly mapped
        let plugin_struct = result
            .structs
            .iter()
            .find(|s| s.name == "PluginStruct")
            .expect("Should find PluginStruct");

        let data_field = plugin_struct
            .fields
            .iter()
            .find(|f| f.name == "data")
            .expect("Should find data field");
        assert_eq!(data_field.python_type, "List[attrs2bin.U8]");

        Ok(())
    }

    /// Test that module parsing handles array and slice types correctly
    #[test]
    fn test_module_array_and_slice_types() -> Result<()> {
        let source = r#"
            pub mod types {
                pub struct ArrayStruct {
                    pub fixed_array: [u8; 6],
                    pub slice_ref: &'static [u8],
                    pub vec_data: Vec<u16>,
                    pub heapless_vec: heapless::Vec<u32, 10>,
                }
            }
        "#;

        let result = parse_rust_source(source)?;

        assert_eq!(result.structs.len(), 1);
        let struct_def = &result.structs[0];
        assert_eq!(struct_def.name, "ArrayStruct");

        // Check that array types are converted correctly
        let fixed_array_field = struct_def
            .fields
            .iter()
            .find(|f| f.name == "fixed_array")
            .expect("Should find fixed_array field");
        assert_eq!(fixed_array_field.python_type, "List[attrs2bin.U8]");

        let slice_ref_field = struct_def
            .fields
            .iter()
            .find(|f| f.name == "slice_ref")
            .expect("Should find slice_ref field");
        assert_eq!(slice_ref_field.python_type, "bytes");

        let vec_data_field = struct_def
            .fields
            .iter()
            .find(|f| f.name == "vec_data")
            .expect("Should find vec_data field");
        assert_eq!(vec_data_field.python_type, "List[int]");

        let heapless_vec_field = struct_def
            .fields
            .iter()
            .find(|f| f.name == "heapless_vec")
            .expect("Should find heapless_vec field");
        assert_eq!(heapless_vec_field.python_type, "List[int]");

        Ok(())
    }

    /// Test module parsing with const expressions and complex values
    #[test]
    fn test_module_const_expressions() -> Result<()> {
        let source = r#"
            pub mod constants {
                pub const SIMPLE_VALUE: u32 = 42;
                pub const HEX_VALUE: u16 = 0xDEAD;
                pub const STRING_VALUE: &str = "test_string";
                pub const BOOL_VALUE: bool = true;
                pub const REFERENCE_VALUE: usize = SIMPLE_VALUE;
            }
        "#;

        let result = parse_rust_source(source)?;

        assert_eq!(result.constants.len(), 5);

        let simple_const = result
            .constants
            .iter()
            .find(|c| c.name == "SIMPLE_VALUE")
            .expect("Should find SIMPLE_VALUE");
        assert_eq!(simple_const.value, "42");

        let hex_const = result
            .constants
            .iter()
            .find(|c| c.name == "HEX_VALUE")
            .expect("Should find HEX_VALUE");
        assert_eq!(hex_const.value, "0xDEAD");

        let string_const = result
            .constants
            .iter()
            .find(|c| c.name == "STRING_VALUE")
            .expect("Should find STRING_VALUE");
        assert_eq!(string_const.value, "\"test_string\"");

        let bool_const = result
            .constants
            .iter()
            .find(|c| c.name == "BOOL_VALUE")
            .expect("Should find BOOL_VALUE");
        assert_eq!(bool_const.value, "true");

        let ref_const = result
            .constants
            .iter()
            .find(|c| c.name == "REFERENCE_VALUE")
            .expect("Should find REFERENCE_VALUE");
        assert_eq!(ref_const.value, "SIMPLE_VALUE");

        Ok(())
    }
}
