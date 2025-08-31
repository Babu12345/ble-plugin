use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn main() {
    #[cfg(feature = "protocol_buffers")]
    {
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
            
            // Check for @rust_macro annotation for custom macros like HostIO, PluginIO
            if line.contains("@rust_macro(") {
                if let Some(start) = line.find("@rust_macro(") {
                    let macro_start = start + "@rust_macro(".len();
                    if let Some(end) = line[macro_start..].find(')') {
                        let macro_content = &line[macro_start..macro_start + end];
                        // Handle special cases for HostIO and PluginIO with MessageTypeId
                        if macro_content.starts_with("HostIO(") || macro_content.starts_with("PluginIO(") {
                            // Convert MessageTypeId:: references to crate::protocol::MessageTypeId::
                            let processed = macro_content.replace("MessageTypeId::", "crate::protocol::MessageTypeId::");
                            pending_attributes.push(format!("#[{}]", processed));
                        } else {
                            pending_attributes.push(format!("#[{}]", macro_content));
                        }
                    }
                }
            }

            // Check for enum or message declarations
            if line.trim().starts_with("enum ") || line.trim().starts_with("message ") {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    let type_name = parts[1].trim_end_matches('{');
                    if !pending_attributes.is_empty() {
                        type_attributes
                            .insert(format!("protocol.{}", type_name), pending_attributes.clone());
                        pending_attributes.clear();
                    }
                }
            }
        }

        let out_dir = PathBuf::from("src");
        let mut config = prost_build::Config::new();
        config.out_dir(out_dir);

        // Add all attributes (derives and custom macros) for each type
        for (type_path, attributes) in type_attributes {
            for attribute in attributes {
                config.type_attribute(&type_path, &attribute);
            }
        }

        config.compile_protos(&["protocol.proto"], &["."]).unwrap();
    }
}
