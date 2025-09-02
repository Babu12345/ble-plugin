fn main() {
    #[cfg(feature = "quick_protocol_buffer")]
    {
        use std::collections::HashMap;
        use std::fs;
        use std::io::Write;
        use std::path::{Path, PathBuf};

        // Parse the proto file to find @derive and @rust_macro annotations
        let proto_content =
            fs::read_to_string("protocol.proto").expect("Failed to read protocol.proto");

        let mut type_attributes: HashMap<String, Vec<String>> = HashMap::new();
        let mut pending_attributes: Vec<String> = Vec::new();

        for line in proto_content.lines() {
            // Check for @derive annotation
            if line.contains("@derive(") {
                if let Some(start) = line.find("@derive(") {
                    let derive_start = start + "@derive(".len();
                    if let Some(end) = line[derive_start..].find(')') {
                        let derives = &line[derive_start..derive_start + end];
                        pending_attributes.push(format!("#[derive({})]", derives));
                    }
                }
            }

            // Check for @rust_macro annotation - handle nested parentheses
            if line.contains("@rust_macro(") {
                if let Some(start) = line.find("@rust_macro(") {
                    let macro_start = start + "@rust_macro(".len();
                    // Find matching closing parenthesis, accounting for nested ones
                    let rest = &line[macro_start..];
                    let mut paren_count = 1;
                    let mut end_pos = 0;

                    for (i, ch) in rest.chars().enumerate() {
                        if ch == '(' {
                            paren_count += 1;
                        } else if ch == ')' {
                            paren_count -= 1;
                            if paren_count == 0 {
                                end_pos = i;
                                break;
                            }
                        }
                    }

                    if paren_count == 0 {
                        let macro_content = &rest[..end_pos];
                        pending_attributes.push(format!("#[{}]", macro_content));
                    }
                }
            }

            // Check for enum or message declarations
            if line.trim().starts_with("enum ") || line.trim().starts_with("message ") {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    let type_name = parts[1].trim_end_matches('{');
                    if !pending_attributes.is_empty() {
                        type_attributes.insert(
                            format!("protocol.{}", type_name),
                            pending_attributes.clone(),
                        );
                        pending_attributes.clear();
                    }
                }
            }
        }

        let in_file = Path::new("protocol.proto");
        let out_dir = PathBuf::from("src");

        // Generate with pb-rs
        use pb_rs::{types::FileDescriptor, ConfigBuilder};

        let config = ConfigBuilder::new(
            &[in_file],
            None,
            Some(&out_dir.as_path()),
            &[Path::new(".")],
        )
        .expect("Failed to create pb-rs config")
        .dont_use_cow(true)
        .nostd(cfg!(not(feature = "std")))
        .build();

        FileDescriptor::run(&config).expect("Failed to generate protocol code with pb-rs");

        // Post-process the generated file to add attributes and imports
        let protocol_file = out_dir.join("protocol.rs");
        let content =
            fs::read_to_string(&protocol_file).expect("Failed to read generated protocol.rs");

        let mut new_content = String::new();
        let lines: Vec<&str> = content.lines().collect();

        // Find where inner attributes end
        let mut inner_attr_end = 0;
        for (i, line) in lines.iter().enumerate() {
            if !line.trim().starts_with("#!")
                && !line.trim().starts_with("//")
                && !line.trim().is_empty()
            {
                inner_attr_end = i;
                break;
            }
        }

        // Add inner attributes and comments first
        for i in 0..inner_attr_end {
            new_content.push_str(lines[i]);
            new_content.push('\n');
        }

        // Add our imports
        new_content.push_str("#![allow(missing_docs)]\n");
        #[cfg(not(feature = "std"))]
        new_content.push_str("\nextern crate alloc;\n");
        new_content.push_str("use crate::{IO, IOBase, HostIO, PluginIO, MessageType};\n");

        // Process the rest of the file
        for i in inner_attr_end..lines.len() {
            let line = lines[i];

            // Check if this line declares a type we have attributes for
            let mut attributes_to_add = Vec::new();

            for (type_path, attributes) in &type_attributes {
                // Extract just the type name from the full path "protocol.TypeName"
                if let Some(type_name) = type_path.strip_prefix("protocol.") {
                    if line.contains(&format!("pub struct {}", type_name))
                        || line.contains(&format!("pub enum {}", type_name))
                    {
                        attributes_to_add = attributes.clone();
                        break;
                    }
                }
            }

            // Add attributes before the type declaration
            // Filter attributes based on the type: enums should not get @rust_macro
            for attr in attributes_to_add {
                let is_enum = line.contains("pub enum ");
                let is_rust_macro = attr.starts_with("#[protocol_io::");

                // Skip @rust_macro attributes for enums
                if is_enum && is_rust_macro {
                    continue;
                }

                new_content.push_str(&attr);
                new_content.push('\n');
            }

            new_content.push_str(line);
            new_content.push('\n');
        }

        let mut file =
            fs::File::create(&protocol_file).expect("Failed to open protocol.rs for writing");
        file.write_all(new_content.as_bytes())
            .expect("Failed to write modified protocol.rs");
    }

    #[cfg(feature = "protocol_buffer")]
    {
        use std::collections::HashMap;
        use std::fs;
        use std::io::Write;
        use std::path::PathBuf;
        // Parse the proto file to find @derive and @rust_macro annotations
        let proto_content =
            fs::read_to_string("protocol.proto").expect("Failed to read protocol.proto");

        let mut type_attributes: HashMap<String, Vec<String>> = HashMap::new();
        let mut pending_attributes: Vec<String> = Vec::new();

        for line in proto_content.lines() {
            // Check for @derive annotation
            if line.contains("@derive(") {
                if let Some(start) = line.find("@derive(") {
                    let derive_start = start + "@derive(".len();
                    if let Some(end) = line[derive_start..].find(')') {
                        let derives = &line[derive_start..derive_start + end];
                        pending_attributes.push(format!("#[derive({})]", derives));
                    }
                }
            }

            // Check for @rust_macro annotation - handle nested parentheses
            if line.contains("@rust_macro(") {
                if let Some(start) = line.find("@rust_macro(") {
                    let macro_start = start + "@rust_macro(".len();
                    // Find matching closing parenthesis, accounting for nested ones
                    let rest = &line[macro_start..];
                    let mut paren_count = 1;
                    let mut end_pos = 0;

                    for (i, ch) in rest.chars().enumerate() {
                        if ch == '(' {
                            paren_count += 1;
                        } else if ch == ')' {
                            paren_count -= 1;
                            if paren_count == 0 {
                                end_pos = i;
                                break;
                            }
                        }
                    }

                    if paren_count == 0 {
                        let macro_content = &rest[..end_pos];
                        pending_attributes.push(format!("#[{}]", macro_content));
                    }
                }
            }

            // Check for enum or message declarations
            if line.trim().starts_with("enum ") || line.trim().starts_with("message ") {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    let type_name = parts[1].trim_end_matches('{');
                    if !pending_attributes.is_empty() {
                        type_attributes.insert(
                            format!("protocol.{}", type_name),
                            pending_attributes.clone(),
                        );
                        pending_attributes.clear();
                    }
                }
            }
        }

        let out_dir = PathBuf::from("src");
        let mut config = prost_build::Config::new();
        config.out_dir(&out_dir);

        // Add all attributes (derives and macros) for each type
        for (type_path, attributes) in type_attributes {
            for attribute in attributes {
                config.type_attribute(&type_path, &attribute);
            }
        }

        config
            .compile_protos(&["protocol.proto"], &["."])
            .expect("Failed to compile protos");

        // Post-process the generated file to add necessary imports
        let protocol_file = out_dir.join("protocol.rs");
        let content =
            fs::read_to_string(&protocol_file).expect("Failed to read generated protocol.rs");

        // Add imports after the @generated comment
        let imports = "use crate::{IO, IOBase, HostIO, PluginIO, MessageType};";
        let generated_comment = "// This file is @generated by prost-build.";

        let new_content = if let Some(pos) = content.find(generated_comment) {
            // Find the end of the comment line
            let comment_end = pos + generated_comment.len();
            let mut result = String::new();
            result.push_str(&content[..comment_end]);
            result.push_str("\n");
            result.push_str(imports);
            result.push_str("\n\n");
            result.push_str(&content[comment_end..]);
            result
        } else {
            // Fallback: add at the beginning if comment not found
            format!("{}\n{}", imports, content)
        };

        let mut file =
            fs::File::create(&protocol_file).expect("Failed to open protocol.rs for writing");
        file.write_all(new_content.as_bytes())
            .expect("Failed to write modified protocol.rs");
    }
}
