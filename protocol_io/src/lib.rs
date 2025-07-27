#![deny(missing_docs)]
#![no_std]

//! Protocol types for easy use and deriving

use proc_macro::TokenStream;

use quote::quote;
use syn::{DeriveInput, GenericParam, parse_macro_input};

/// Derives HostIO
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

/// Derives PluginIO
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
