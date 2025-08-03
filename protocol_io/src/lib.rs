#![deny(missing_docs)]
#![no_std]

//! # Protocol IO - Procedural Macros for BLE Plugin Protocol
//!
//! This crate provides convenient derive macros for implementing protocol I/O traits
//! in the BLE plugin communication system. It automatically generates trait implementations
//! for `HostIO` and `PluginIO`, reducing boilerplate and ensuring consistent protocol handling.
//!
//! ## Overview
//!
//! The BLE plugin protocol distinguishes between two types of communication:
//! - **Host I/O**: Messages sent from host devices (PCs, mobile) to plugin devices
//! - **Plugin I/O**: Messages sent from plugin devices back to host devices
//!
//! This crate provides derive macros that automatically implement the appropriate
//! I/O traits based on the message direction, handling lifetime parameters and
//! generic constraints correctly.
//!
//! ## Key Features
//!
//! - **Automatic Trait Implementation**: Derives `IO`, `HostIO`, and `PluginIO` traits
//! - **Lifetime Handling**: Correctly manages lifetime parameters in generic types
//! - **Zero Runtime Cost**: Pure compile-time code generation
//! - **Type Safety**: Ensures protocol trait consistency at compile time
//! - **Minimal Dependencies**: Lightweight procedural macro implementation
//!
//! ## Derive Macros
//!
//! ### `#[derive(HostIO)]`
//! Implements `IO<'a>` and `HostIO<'a>` traits for message types sent from hosts to plugins.
//! Use this for command messages that configure or control the BLE plugin device.
//!
//! ### `#[derive(PluginIO)]`
//! Implements `IO<'a>` and `PluginIO<'a>` traits for message types sent from plugins to hosts.
//! Use this for response messages, data forwarding, and error notifications.
//!
//! ## Usage Examples
//!
//! ### Host Command Message
//!
//! ```rust
//! use protocol_io::HostIO;
//! use serde::{Serialize, Deserialize};
//! use protocol::{MessageType, MessageTypeId};
//!
//! #[derive(Serialize, Deserialize, HostIO)]
//! struct ConfigurePeripheralCommand {
//!     name: String,
//!     uuid: String,
//! }
//!
//! impl MessageType for ConfigurePeripheralCommand {
//!     fn message_type_id() -> MessageTypeId {
//!         MessageTypeId::HostCommandConfigurePeripheral
//!     }
//! }
//!
//! // Now automatically implements IO<'a> and HostIO<'a>
//! ```
//!
//! ### Plugin Response Message
//!
//! ```rust
//! use protocol_io::PluginIO;
//! use serde::{Serialize, Deserialize};
//! use protocol::{MessageType, MessageTypeId};
//!
//! #[derive(Serialize, Deserialize, PluginIO)]
//! struct ServiceInfoResponse {
//!     service_uuid: String,
//!     characteristics: Vec<String>,
//!     exists: bool,
//! }
//!
//! impl MessageType for ServiceInfoResponse {
//!     fn message_type_id() -> MessageTypeId {
//!         MessageTypeId::PluginServiceInfoResponse
//!     }
//! }
//!
//! // Now automatically implements IO<'a> and PluginIO<'a>
//! ```
//!
//! ### Generic Types with Lifetimes
//!
//! The macros correctly handle generic types with lifetime parameters:
//!
//! ```rust
//! use protocol_io::HostIO;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, HostIO)]
//! struct GenericCommand<'a> {
//!     data: &'a [u8],
//!     name: &'a str,
//! }
//!
//! // Correctly implements IO<'a> and HostIO<'a> with proper lifetime bounds
//! ```
//!
//! ## Code Generation
//!
//! The derive macros generate implementations that look like this:
//!
//! ```rust,ignore
//! // For #[derive(HostIO)]
//! impl<'a> IO<'a> for YourType {}
//! impl<'a> HostIO<'a> for YourType {}
//!
//! // For #[derive(PluginIO)]  
//! impl<'a> IO<'a> for YourType {}
//! impl<'a> PluginIO<'a> for YourType {}
//! ```
//!
//! ## Integration
//!
//! This crate is designed to work seamlessly with the main `protocol` crate:
//!
//! ```toml
//! [dependencies]
//! protocol = { path = "../protocol" }
//! protocol_io = { path = "../protocol_io" }
//! serde = { version = "1.0", features = ["derive"] }
//! ```
//!
//! ## Requirements
//!
//! To use these derive macros, your types must:
//!
//! 1. Implement `Serialize` and `Deserialize` from serde
//! 2. Implement the `MessageType` trait from the protocol crate
//! 3. Be used with the appropriate derive macro for their direction
//!
//! ## Error Handling
//!
//! The macros perform compile-time validation and will produce clear error
//! messages if used incorrectly. Common issues include:
//!
//! - Missing serde derives
//! - Missing MessageType implementation
//! - Incorrect lifetime parameter usage

use proc_macro::TokenStream;

use quote::quote;
use syn::{DeriveInput, GenericParam, parse_macro_input};

/// Derive macro for implementing HostIO traits
///
/// This macro automatically implements both `IO<'a>` and `HostIO<'a>` traits for types
/// that represent messages sent from host devices to plugin devices. It handles both
/// simple types and generic types with lifetime parameters.
///
/// ## Usage
///
/// ```rust
/// use protocol_io::HostIO;
/// use serde::{Serialize, Deserialize};
/// use protocol::{MessageType, MessageTypeId};
///
/// #[derive(Serialize, Deserialize, HostIO)]
/// struct MyHostCommand {
///     data: u32,
///     flag: bool,
/// }
///
/// impl MessageType for MyHostCommand {
///     fn message_type_id() -> MessageTypeId {
///         MessageTypeId::HostCommandConfigurePeripheral
///     }
/// }
/// ```
///
/// ## Generated Code
///
/// For a type `MyHostCommand`, this macro generates:
///
/// ```rust,ignore
/// impl<'a> IO<'a> for MyHostCommand {}
/// impl<'a> HostIO<'a> for MyHostCommand {}
/// ```
///
/// ## Lifetime Handling
///
/// For types with existing lifetime parameters, the macro correctly uses the first
/// lifetime parameter:
///
/// ```rust
/// use protocol_io::HostIO;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, HostIO)]
/// struct CommandWithLifetime<'a> {
///     data: &'a [u8],
/// }
/// ```
///
/// This generates:
///
/// ```rust,ignore
/// impl<'a> IO<'a> for CommandWithLifetime<'a> {}
/// impl<'a> HostIO<'a> for CommandWithLifetime<'a> {}
/// ```
///
/// ## Requirements
///
/// The target type must:
/// - Implement `Serialize` and `Deserialize`
/// - Implement `MessageType`
/// - Be used for host-to-plugin communication
#[proc_macro_derive(HostIO)]
pub fn derive_host_io(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    let mut lifetimes = generics.params.iter().filter_map(|param| {
        if let GenericParam::Lifetime(lifetime_def) = param {
            Some(&lifetime_def.lifetime)
        } else {
            None
        }
    });

    let expanded = match lifetimes.next() {
        None => {
            // Generate the trait implementation
            quote! {
                impl<'a> IO<'a> for #name {}
                impl<'a> HostIO<'a> for #name {}
            }
        }
        Some(first_lt) => {
            // Extract existing generics and add the lifetime parameter
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

            // Generate the trait implementation
            quote! {
                impl #impl_generics IO<#first_lt> for #name #ty_generics #where_clause {}
                impl #impl_generics HostIO<#first_lt> for #name #ty_generics #where_clause {}
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for implementing PluginIO traits
///
/// This macro automatically implements both `IO<'a>` and `PluginIO<'a>` traits for types
/// that represent messages sent from plugin devices to host devices. It handles both
/// simple types and generic types with lifetime parameters.
///
/// ## Usage
///
/// ```rust
/// use protocol_io::PluginIO;
/// use serde::{Serialize, Deserialize};
/// use protocol::{MessageType, MessageTypeId};
///
/// #[derive(Serialize, Deserialize, PluginIO)]
/// struct MyPluginResponse {
///     status: String,
///     data: Vec<u8>,
/// }
///
/// impl MessageType for MyPluginResponse {
///     fn message_type_id() -> MessageTypeId {
///         MessageTypeId::PluginServiceInfoResponse
///     }
/// }
/// ```
///
/// ## Generated Code
///
/// For a type `MyPluginResponse`, this macro generates:
///
/// ```rust,ignore
/// impl<'a> IO<'a> for MyPluginResponse {}
/// impl<'a> PluginIO<'a> for MyPluginResponse {}
/// ```
///
/// ## Lifetime Handling
///
/// For types with existing lifetime parameters, the macro correctly uses the first
/// lifetime parameter:
///
/// ```rust
/// use protocol_io::PluginIO;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, PluginIO)]
/// struct ResponseWithLifetime<'a> {
///     message: &'a str,
/// }
/// ```
///
/// This generates:
///
/// ```rust,ignore
/// impl<'a> IO<'a> for ResponseWithLifetime<'a> {}
/// impl<'a> PluginIO<'a> for ResponseWithLifetime<'a> {}
/// ```
///
/// ## Use Cases
///
/// This derive is appropriate for:
/// - Configuration responses (success/error status)
/// - Data forwarding from BLE clients to host
/// - Service and characteristic information responses
/// - Error notifications and status updates
///
/// ## Requirements
///
/// The target type must:
/// - Implement `Serialize` and `Deserialize`
/// - Implement `MessageType`
/// - Be used for plugin-to-host communication
#[proc_macro_derive(PluginIO)]
pub fn derive_plugin_io(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    let mut lifetimes = generics.params.iter().filter_map(|param| {
        if let GenericParam::Lifetime(lifetime_def) = param {
            Some(&lifetime_def.lifetime)
        } else {
            None
        }
    });

    let expanded = match lifetimes.next() {
        None => {
            // Generate the trait implementation
            quote! {
                impl<'a> IO<'a> for #name {}
                impl<'a> PluginIO<'a> for #name {}
            }
        }
        Some(first_lt) => {
            // Extract existing generics and add the lifetime parameter
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

            // Generate the trait implementation
            quote! {
                impl #impl_generics IO<#first_lt> for #name #ty_generics #where_clause {}
                impl #impl_generics PluginIO<#first_lt> for #name #ty_generics #where_clause {}
            }
        }
    };

    TokenStream::from(expanded)
}
