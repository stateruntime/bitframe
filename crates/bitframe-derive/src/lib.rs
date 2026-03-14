//! Proc-macro internals for `bitframe`.
//!
//! This crate is not meant to be used directly. Use the `bitframe` crate instead,
//! which re-exports the `#[bitframe]` and `#[bitframe_enum]` attribute macros.

#![forbid(unsafe_code)]
#![allow(unreachable_pub)]

use proc_macro::TokenStream;

mod bitframe_attr;
mod bitframe_enum_attr;
mod codegen;
mod extract;
mod field;

/// Generates a zero-copy view type for a bit-level packet layout.
///
/// Apply to a struct with named fields. Each field's type determines its bit width:
///
/// | Type | Width |
/// |------|-------|
/// | `bool` | 1 bit |
/// | `u8`, `u16`, `u32`, `u64` | 8/16/32/64 bits |
/// | `u1`..`u63` | 1..63 bits |
///
/// See the [`bitframe`](https://docs.rs/bitframe) crate documentation for full usage.
#[proc_macro_attribute]
pub fn bitframe(attr: TokenStream, item: TokenStream) -> TokenStream {
    match bitframe_attr::expand_bitframe(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Generates bit-width-aware conversion methods for an enum.
///
/// See the [`bitframe`](https://docs.rs/bitframe) crate documentation for usage.
#[proc_macro_attribute]
pub fn bitframe_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    match bitframe_enum_attr::expand_bitframe_enum(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
