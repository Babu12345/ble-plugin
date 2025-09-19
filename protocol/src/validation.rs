#![cfg(test)]

//! MESSAGE_TYPE_ID registration and uniqueness validation using inventory.
//! Should only be used for testing
//!
//! This module uses the inventory crate to collect MESSAGE_TYPE_ID registrations
//! from all structs that use the protocol_io macros, ensuring uniqueness. Only to be used in tests
use crate::protocol::MessageTypeId;

// Collect all MESSAGE_TYPE_ID registrations (test builds only)
inventory::collect!(MessageTypeId);

#[test]
#[cfg(feature = "std")]
/// Validate that all registered MESSAGE_TYPE_IDs are unique
pub fn validate_message_type_id_uniqueness() {
    use std::collections::HashMap;

    let mut id_counts: HashMap<u16, usize> = HashMap::new();

    // Count occurrences of each MESSAGE_TYPE_ID
    for registration in inventory::iter::<MessageTypeId> {
        let id_value = *registration as u16;
        *id_counts.entry(id_value).or_insert(0) += 1;
    }

    // Check for conflicts
    let mut conflicts_found = false;
    for (id_value, count) in &id_counts {
        if *count > 1 {
            conflicts_found = true;
            eprintln!(
                "❌ MESSAGE_TYPE_ID conflict: 0x{:04X} used by {} IO structs",
                id_value, count
            );
        }
    }

    if conflicts_found {
        panic!("MESSAGE_TYPE_ID validation failed: conflicts detected");
    }

    println!(
        "✅ MESSAGE_TYPE_ID validation passed: {} unique registrations",
        id_counts.len()
    );

    // Debug output if requested
    if std::env::var("PROTOCOL_DEBUG").is_ok() {
        let mut sorted: Vec<_> = id_counts.iter().collect();
        sorted.sort_by_key(|(id, _)| *id);

        println!("📋 Registered MESSAGE_TYPE_IDs:");
        for (id_value, count) in sorted {
            println!(
                "   0x{:04X} (used {} time{})",
                id_value,
                count,
                if *count == 1 { "" } else { "s" }
            );
        }
    }
}
