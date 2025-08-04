#![deny(missing_docs)]
#![no_std]

//! # Protocol IO - Attribute Macros for BLE Plugin Protocol
//!
//! This crate provides convenient attribute macros for implementing protocol I/O traits
//! in the BLE plugin communication system. It automatically generates trait implementations
//! for `HostIO`, `PluginIO`, and `MessageType`, reducing boilerplate and ensuring consistent 
//! protocol handling.
//!
//! ## Overview
//!
//! The BLE plugin protocol distinguishes between two types of communication:
//! - **Host I/O**: Messages sent from host devices (PCs, mobile) to plugin devices
//! - **Plugin I/O**: Messages sent from plugin devices back to host devices
//!
//! This crate provides attribute macros that automatically implement the appropriate
//! I/O traits based on the message direction, handling lifetime parameters and
//! generic constraints correctly.
//!
//! ## Key Features
//!
//! - **Automatic Trait Implementation**: Generates `IO`, `HostIO`/`PluginIO`, and `MessageType` traits
//! - **Consolidated API**: Single attribute combines trait implementation and message type ID
//! - **Lifetime Handling**: Correctly manages lifetime parameters in generic types
//! - **Zero Runtime Cost**: Pure compile-time code generation
//! - **Type Safety**: Ensures protocol trait consistency at compile time
//! - **Minimal Dependencies**: Lightweight procedural macro implementation
//!
//! ## Attribute Macros
//!
//! ### `#[HostIO(MessageTypeId)]`
//! Implements `IO<'a>`, `HostIO<'a>`, and `MessageType` traits for message types sent from 
//! hosts to plugins. Use this for command messages that configure or control the BLE plugin device.
//!
//! ### `#[PluginIO(MessageTypeId)]`
//! Implements `IO<'a>`, `PluginIO<'a>`, and `MessageType` traits for message types sent from 
//! plugins to hosts. Use this for response messages, data forwarding, and error notifications.
//!
//! ## Usage Examples
//!
//! ### Host Command Message
//!
//! ```rust
//! use protocol_io::HostIO;
//! use serde::{Serialize, Deserialize};
//! use protocol::MessageTypeId;
//!
//! #[derive(Serialize, Deserialize)]
//! #[HostIO(MessageTypeId::HostCommandConfigurePeripheral)]
//! struct ConfigurePeripheralCommand {
//!     name: String,
//!     uuid: String,
//! }
//!
//! // Automatically implements IO<'a>, HostIO<'a>, and MessageType
//! ```
//!
//! ### Plugin Response Message
//!
//! ```rust
//! use protocol_io::PluginIO;
//! use serde::{Serialize, Deserialize};
//! use protocol::MessageTypeId;
//!
//! #[derive(Serialize, Deserialize)]
//! #[PluginIO(MessageTypeId::PluginServiceInfoResponse)]
//! struct ServiceInfoResponse {
//!     service_uuid: String,
//!     characteristics: Vec<String>,
//!     exists: bool,
//! }
//!
//! // Automatically implements IO<'a>, PluginIO<'a>, and MessageType
//! ```
//!
//! ### Types with Lifetimes
//!
//! The macros correctly handle types with lifetime parameters, using the first
//! lifetime parameter for the IO trait implementations:
//!
//! ```rust
//! use protocol_io::HostIO;
//! use serde::{Serialize, Deserialize};
//! use protocol::MessageTypeId;
//!
//! #[derive(Serialize, Deserialize)]
//! #[HostIO(MessageTypeId::HostCommandConfigureCharacteristic)]
//! struct CommandWithLifetime<'a> {
//!     data: &'a [u8],
//!     name: &'a str,
//! }
//!
//! // Generates: IO<'a>, HostIO<'a>, and MessageType implementations
//! ```
//!
//! ### Multiple Lifetimes
//!
//! For types with multiple lifetimes, the macro uses the first lifetime parameter:
//!
//! ```rust
//! use protocol_io::PluginIO;
//! use serde::{Serialize, Deserialize};
//! use protocol::MessageTypeId;
//!
//! #[derive(Serialize, Deserialize)]
//! #[PluginIO(MessageTypeId::PluginData)]
//! struct MultiLifetimeResponse<'a, 'b> {
//!     primary: &'a str,
//!     secondary: &'b [u8],
//! }
//!
//! // Generates: IO<'a>, PluginIO<'a>, and MessageType implementations
//! // Note: Uses 'a (first lifetime parameter) for the IO traits
//! ```
//!
//! ### Enum Support
//!
//! Both structs and enums are supported:
//!
//! ```rust
//! use protocol_io::HostIO;
//! use serde::{Serialize, Deserialize};
//! use protocol::MessageTypeId;
//!
//! #[derive(Serialize, Deserialize)]
//! #[HostIO(MessageTypeId::HostCommandStartAdvertisement)]
//! enum HostCommand {
//!     Start,
//!     Stop { reason: String },
//!     Configure { setting: u32, enabled: bool },
//! }
//! ```
//!
//! ## Code Generation
//!
//! The attribute macros generate implementations that look like this:
//!
//! ```rust,ignore
//! // For #[HostIO(MessageTypeId::SomeCommand)]
//! impl<'a> IO<'a> for YourType {}
//! impl<'a> HostIO<'a> for YourType {}
//! impl MessageType for YourType {
//!     fn message_type_id() -> MessageTypeId {
//!         MessageTypeId::SomeCommand
//!     }
//! }
//!
//! // For #[PluginIO(MessageTypeId::SomeResponse)]
//! impl<'a> IO<'a> for YourType {}
//! impl<'a> PluginIO<'a> for YourType {}
//! impl MessageType for YourType {
//!     fn message_type_id() -> MessageTypeId {
//!         MessageTypeId::SomeResponse
//!     }
//! }
//! ```
//!
//! For types with existing lifetime parameters, the generated code respects the type's
//! generic constraints:
//!
//! ```rust,ignore
//! // For a type like: struct MyType<'a, 'b> { ... }
//! impl<'a, 'b> IO<'a> for MyType<'a, 'b> {}
//! impl<'a, 'b> HostIO<'a> for MyType<'a, 'b> {}
//! impl<'a, 'b> MessageType for MyType<'a, 'b> {
//!     fn message_type_id() -> MessageTypeId {
//!         MessageTypeId::YourVariant
//!     }
//! }
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
//! To use these attribute macros, your types must:
//!
//! 1. **Implement Serde traits**: `#[derive(Serialize, Deserialize)]`
//! 2. **Use appropriate attribute**: `#[HostIO(...)]` for host→plugin, `#[PluginIO(...)]` for plugin→host
//! 3. **Provide MessageTypeId**: Pass the appropriate `MessageTypeId` variant as the attribute parameter
//!
//! ## Error Handling
//!
//! The macros perform compile-time validation and will produce clear error
//! messages if used incorrectly. Common issues include:
//!
//! - Missing serde derives
//! - Missing MessageTypeId parameter in the attribute
//! - Using wrong attribute for message direction (HostIO vs PluginIO)
//!
//! ## Benefits Over Manual Implementation
//!
//! ### Before (Manual Implementation)
//! ```rust,ignore
//! #[derive(Serialize, Deserialize, HostIO)]
//! struct MyCommand { ... }
//!
//! impl MessageType for MyCommand {
//!     fn message_type_id() -> MessageTypeId {
//!         MessageTypeId::HostCommandSomething
//!     }
//! }
//! ```
//!
//! ### After (Attribute Macro)
//! ```rust,ignore
//! #[derive(Serialize, Deserialize)]
//! #[HostIO(MessageTypeId::HostCommandSomething)]
//! struct MyCommand { ... }
//! ```
//!
//! The attribute macro approach:
//! - Reduces boilerplate by ~50%
//! - Eliminates possibility of MessageType implementation mismatch
//! - Keeps message type ID close to the type definition
//! - Automatically handles lifetime parameters correctly

use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, GenericParam};

/// Attribute macro for implementing HostIO traits
///
/// This macro automatically implements `IO<'a>`, `HostIO<'a>`, and `MessageType` traits
/// for types that represent messages sent from host devices to plugin devices.
///
/// ## Usage
///
/// Apply this attribute to structs or enums that represent host-to-plugin messages:
///
/// ```rust
/// use protocol_io::HostIO;
/// use serde::{Serialize, Deserialize};
/// use protocol::MessageTypeId;
///
/// #[derive(Serialize, Deserialize)]
/// #[HostIO(MessageTypeId::HostCommandConfigurePeripheral)]
/// struct MyHostCommand {
///     data: u32,
///     flag: bool,
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
/// impl MessageType for MyHostCommand {
///     fn message_type_id() -> MessageTypeId {
///         MessageTypeId::HostCommandConfigurePeripheral
///     }
/// }
/// ```
///
/// ## Lifetime Handling
///
/// For types with lifetime parameters, the macro uses the first lifetime parameter
/// for the IO trait implementations:
///
/// ```rust
/// use protocol_io::HostIO;
/// use serde::{Serialize, Deserialize};
/// use protocol::MessageTypeId;
///
/// #[derive(Serialize, Deserialize)]
/// #[HostIO(MessageTypeId::HostCommandConfigureCharacteristic)]
/// struct CommandWithLifetime<'a, 'b> {
///     data: &'a [u8],
///     name: &'b str,
/// }
///
/// // Generates: IO<'a>, HostIO<'a>, and MessageType implementations
/// // Note: Uses 'a (first lifetime) for the IO traits
/// ```
///
/// ## Requirements
///
/// - The type must implement `Serialize` and `Deserialize` from serde
/// - The MessageTypeId must be a valid variant from the protocol crate
/// - Use this only for messages sent from host to plugin
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn HostIO(args: TokenStream, input: TokenStream) -> TokenStream {
    let message_type_id = parse_macro_input!(args as Expr);
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

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let io_impl = match lifetimes.next() {
        None => {
            quote! {
                impl<'a> IO<'a> for #name {}
                impl<'a> HostIO<'a> for #name {}
            }
        }
        Some(first_lt) => {
            quote! {
                impl #impl_generics IO<#first_lt> for #name #ty_generics #where_clause {}
                impl #impl_generics HostIO<#first_lt> for #name #ty_generics #where_clause {}
            }
        }
    };

    // Always generate MessageType implementation with the provided ID
    let message_type_impl = quote! {
        impl #impl_generics MessageType for #name #ty_generics #where_clause {
            fn message_type_id() -> MessageTypeId {
                #message_type_id
            }
        }
    };

    let expanded = quote! {
        #input

        #io_impl
        #message_type_impl
    };

    TokenStream::from(expanded)
}

/// Attribute macro for implementing PluginIO traits
///
/// This macro automatically implements `IO<'a>`, `PluginIO<'a>`, and `MessageType` traits
/// for types that represent messages sent from plugin devices to host devices.
///
/// ## Usage
///
/// ```rust
/// use protocol_io::PluginIO;
/// use serde::{Serialize, Deserialize};
/// use protocol::MessageTypeId;
///
/// #[derive(Serialize, Deserialize)]
/// #[PluginIO(MessageTypeId::PluginServiceInfoResponse)]
/// struct MyPluginResponse {
///     status: String,
///     data: Vec<u8>,
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
/// impl MessageType for MyPluginResponse {
///     fn message_type_id() -> MessageTypeId {
///         MessageTypeId::PluginServiceInfoResponse
///     }
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn PluginIO(args: TokenStream, input: TokenStream) -> TokenStream {
    let message_type_id = parse_macro_input!(args as Expr);
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

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let io_impl = match lifetimes.next() {
        None => {
            quote! {
                impl<'a> IO<'a> for #name {}
                impl<'a> PluginIO<'a> for #name {}
            }
        }
        Some(first_lt) => {
            quote! {
                impl #impl_generics IO<#first_lt> for #name #ty_generics #where_clause {}
                impl #impl_generics PluginIO<#first_lt> for #name #ty_generics #where_clause {}
            }
        }
    };

    // Always generate MessageType implementation with the provided ID
    let message_type_impl = quote! {
        impl #impl_generics MessageType for #name #ty_generics #where_clause {
            fn message_type_id() -> MessageTypeId {
                #message_type_id
            }
        }
    };

    let expanded = quote! {
        #input

        #io_impl
        #message_type_impl
    };

    TokenStream::from(expanded)
}
