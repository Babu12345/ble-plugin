//! Python code generator for protocol library
//!
//! This binary parses the Rust protocol library and generates equivalent Python code
//! to ensure consistency between the two implementations.

use anyhow::{Context, Result};
use askama::Template;
use clap::Parser;
use codegen::{ProtocolDef, ConstantDef, EnumDef, StructDef};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "generate-python")]
#[command(about = "Generate Python code from Rust protocol library")]
struct Args {
    /// Path to the protocol library source
    #[arg(short, long, default_value = "../protocol/src")]
    protocol_path: PathBuf,
    
    /// Output directory for generated Python code
    #[arg(short, long, default_value = "../pc/python/plugin_host")]
    output_dir: PathBuf,
    
    /// Validate existing Python code against Rust definitions
    #[arg(short, long)]
    validate: bool,
}

/// Template-friendly struct for generation
#[derive(Template)]
#[template(path = "python_types.py.j2")]
struct PythonTypesTemplate {
    constants: Vec<TemplateConstant>,
    enums: Vec<TemplateEnum>,
    structs: Vec<TemplateStruct>,
}

/// Template-friendly constant definition  
#[derive(Clone)]
struct TemplateConstant {
    name: String,
    value: String,
    doc_comment: String,
    rust_type: String,
}

/// Template-friendly enum variant
#[derive(Clone)]
struct TemplateEnumVariant {
    name: String,
    value: String, // Always present, empty if None
    doc_comment: String,
}

/// Template-friendly enum definition
#[derive(Clone)]
struct TemplateEnum {
    name: String,
    variants: Vec<TemplateEnumVariant>,
    doc_comment: String,
    repr: String, // Always present, empty if None
    is_int_enum: bool, // True if enum has integer values (not string values)
}

/// Template-friendly struct field
#[derive(Clone)]
struct TemplateStructField {
    name: String,
    rust_type: String,
    python_type: String,
    doc_comment: String,
    is_optional: bool,
}

/// Template-friendly struct definition
#[derive(Clone)]
struct TemplateStruct {
    name: String,
    fields: Vec<TemplateStructField>,
    doc_comment: String,
    message_type_id: String, // Always present, empty if None
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🔍 Parsing Rust protocol library...");
    let protocol_def = parse_protocol_library(&args.protocol_path)?;

    if args.validate {
        println!("✅ Validating existing Python code...");
        validate_python_code(&args.output_dir, &protocol_def)?;
    } else {
        println!("🐍 Generating Python code...");
        generate_python_code(&args.output_dir, &protocol_def)?;
        println!("✅ Python code generated successfully!");
    }

    Ok(())
}

/// Parse the Rust protocol library and extract definitions
fn parse_protocol_library(protocol_path: &PathBuf) -> Result<ProtocolDef> {
    let mut all_constants = Vec::new();
    let mut all_enums = Vec::new();
    let mut all_structs = Vec::new();

    // Parse io.rs for constants and MessageTypeId enum
    let io_path = protocol_path.join("io.rs");
    let io_source = fs::read_to_string(&io_path)
        .with_context(|| format!("Failed to read {}", io_path.display()))?;
    
    let io_def = codegen::parse_rust_source(&io_source)?;
    all_constants.extend(io_def.constants);
    all_enums.extend(io_def.enums);

    // Parse lib.rs for additional constants
    let lib_path = protocol_path.join("lib.rs");  
    let lib_source = fs::read_to_string(&lib_path)
        .with_context(|| format!("Failed to read {}", lib_path.display()))?;
    
    let lib_def = codegen::parse_rust_source(&lib_source)?;
    all_constants.extend(lib_def.constants);

    // Parse io_types.rs for struct definitions
    let io_types_path = protocol_path.join("io_types.rs");
    let io_types_source = fs::read_to_string(&io_types_path)
        .with_context(|| format!("Failed to read {}", io_types_path.display()))?;
    
    let io_types_def = codegen::parse_rust_source(&io_types_source)?;
    all_enums.extend(io_types_def.enums);
    all_structs.extend(io_types_def.structs);
    all_constants.extend(io_types_def.constants);

    // Add some derived constants that are important for Python
    all_constants.push(ConstantDef {
        name: "MESSAGE_MAGIC_BYTES".to_string(),
        value: "2".to_string(),
        doc_comment: "Size in bytes of the magic number field".to_string(),
        rust_type: "usize".to_string(),
    });

    all_constants.push(ConstantDef {
        name: "MESSAGE_TYPE_ID_BYTES".to_string(),
        value: "1".to_string(),
        doc_comment: "Size in bytes of the message type identifier field".to_string(),
        rust_type: "usize".to_string(),
    });

    all_constants.push(ConstantDef {
        name: "DATA_BYTES_LENGTH_IN_BYTES".to_string(),
        value: "2".to_string(),
        doc_comment: "Size in bytes of the payload length field".to_string(),
        rust_type: "usize".to_string(),
    });

    // Filter to only include relevant constants and remove duplicates
    let mut seen_names = std::collections::HashSet::new();
    let filtered_constants: Vec<ConstantDef> = all_constants.into_iter()
        .filter(|c| {
            let is_relevant = matches!(c.name.as_str(), 
                "MESSAGE_MAGIC" | "MAX_NAME_SIZE" | "DEFAULT_PACKET_SIZE" | 
                "MAX_PROPERTIES" | "MAX_CHARACTERISTICS_PER_SERVICE");
            
            let is_new = seen_names.insert(c.name.clone());
            is_relevant && is_new
        })
        .collect();

    // Map MessageTypeId enum values to struct names for Python generation
    let mut enhanced_structs = all_structs;
    if let Some(message_type_enum) = all_enums.iter().find(|e| e.name == "MessageTypeId") {
        let type_id_map: HashMap<String, String> = message_type_enum.variants.iter()
            .map(|v| (v.name.clone(), v.name.clone()))
            .collect();

        for struct_def in &mut enhanced_structs {
            if let Some(type_id) = type_id_map.get(&struct_def.name) {
                struct_def.message_type_id = Some(type_id.clone());
            }
        }
    }

    Ok(ProtocolDef {
        constants: filtered_constants,
        enums: all_enums,
        structs: enhanced_structs,
    })
}

/// Generate Python code from protocol definitions
fn generate_python_code(output_dir: &PathBuf, protocol_def: &ProtocolDef) -> Result<()> {
    // Ensure output directory exists
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory {}", output_dir.display()))?;

    // Convert to template-friendly structures
    let template_constants: Vec<TemplateConstant> = protocol_def.constants.iter()
        .map(|c| TemplateConstant {
            name: c.name.clone(),
            value: c.value.clone(),
            doc_comment: c.doc_comment.clone(),
            rust_type: c.rust_type.clone(),
        }).collect();

    let template_enums: Vec<TemplateEnum> = protocol_def.enums.iter()
        .map(|e| {
            let variants: Vec<TemplateEnumVariant> = e.variants.iter().map(|v| TemplateEnumVariant {
                name: v.name.clone(),
                value: v.value.clone().unwrap_or_default(),
                doc_comment: v.doc_comment.clone(),
            }).collect();
            
            // Determine if this is an integer enum by checking if any variant has a numeric value
            let is_int_enum = variants.iter().any(|v| {
                !v.value.is_empty() && 
                !v.value.starts_with('"') && 
                !v.value.starts_with('\'') &&
                (v.value.parse::<i64>().is_ok() || v.value.starts_with("0x"))
            });
            
            TemplateEnum {
                name: e.name.clone(),
                variants,
                doc_comment: e.doc_comment.clone(),
                repr: e.repr.clone().unwrap_or_default(),
                is_int_enum,
            }
        }).collect();

    let template_structs: Vec<TemplateStruct> = protocol_def.structs.iter()
        .map(|s| TemplateStruct {
            name: s.name.clone(),
            fields: s.fields.iter().map(|f| TemplateStructField {
                name: f.name.clone(),
                rust_type: f.rust_type.clone(),
                python_type: f.python_type.clone(),
                doc_comment: f.doc_comment.clone(),
                is_optional: f.is_optional,
            }).collect(),
            doc_comment: s.doc_comment.clone(),
            message_type_id: s.message_type_id.clone().unwrap_or_default(),
        }).collect();

    // Generate types.py
    let template = PythonTypesTemplate {
        constants: template_constants,
        enums: template_enums,
        structs: template_structs,
    };

    let generated_code = template.render()
        .context("Failed to render Python template")?;

    let output_file = output_dir.join("generated_types.py");
    fs::write(&output_file, generated_code)
        .with_context(|| format!("Failed to write {}", output_file.display()))?;

    println!("📝 Generated: {}", output_file.display());

    // Generate a summary report
    generate_summary_report(output_dir, protocol_def)?;

    Ok(())
}

/// Generate a summary report of what was generated
fn generate_summary_report(output_dir: &PathBuf, protocol_def: &ProtocolDef) -> Result<()> {
    let report_content = format!(
        r#"# Protocol Code Generation Report

This report summarizes the Python code generated from the Rust protocol library.

## Constants Generated
{}

## Enums Generated
{}

## Structs Generated
{}

## Usage

Replace the existing types.py file with generated_types.py, or carefully merge
the generated definitions into your existing code.

Generated at: {}
"#,
        protocol_def.constants.iter()
            .map(|c| format!("- {} = {} ({})", c.name, c.value, c.rust_type))
            .collect::<Vec<_>>()
            .join("\n"),
        protocol_def.enums.iter()
            .map(|e| format!("- {} ({} variants)", e.name, e.variants.len()))
            .collect::<Vec<_>>()
            .join("\n"),
        protocol_def.structs.iter()
            .map(|s| format!("- {} ({} fields)", s.name, s.fields.len()))
            .collect::<Vec<_>>()
            .join("\n"),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    let report_file = output_dir.join("generation_report.md");
    fs::write(&report_file, report_content)
        .with_context(|| format!("Failed to write report {}", report_file.display()))?;

    Ok(())
}

/// Validate existing Python code against Rust definitions
fn validate_python_code(python_dir: &PathBuf, protocol_def: &ProtocolDef) -> Result<()> {
    let types_file = python_dir.join("types.py");
    
    if !types_file.exists() {
        println!("⚠️  Python types.py file not found at {}", types_file.display());
        return Ok(());
    }

    let python_content = fs::read_to_string(&types_file)
        .with_context(|| format!("Failed to read {}", types_file.display()))?;

    let mut issues = Vec::new();

    // Check MessageTypeId enum values
    if let Some(message_type_enum) = protocol_def.enums.iter().find(|e| e.name == "MessageTypeId") {
        for variant in &message_type_enum.variants {
            if let Some(value) = &variant.value {
                let expected_line = format!("{} = {}", variant.name, value);
                if !python_content.contains(&expected_line) {
                    issues.push(format!("Missing or incorrect MessageTypeId.{} = {}", variant.name, value));
                }
            }
        }
    }

    // Check constants
    for constant in &protocol_def.constants {
        let expected_line = format!("{} = {}", constant.name, constant.value);
        if !python_content.contains(&expected_line) {
            issues.push(format!("Missing or incorrect constant {} = {}", constant.name, constant.value));
        }
    }

    if issues.is_empty() {
        println!("✅ Python code validation passed!");
    } else {
        println!("❌ Python code validation found {} issues:", issues.len());
        for issue in issues {
            println!("  - {}", issue);
        }
        println!("\n💡 Run without --validate to generate corrected Python code.");
    }

    Ok(())
}