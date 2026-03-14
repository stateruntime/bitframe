//! Code generation for `#[bitframe]` structs.
//!
//! Takes parsed fields and generates the `FooRef<'a>` view type with all
//! accessors, trait implementations, and parsing functions.
//!
//! # What gets generated
//!
//! For a struct `Foo` with fields, this module generates:
//!
//! - **`FooRef<'a>`** — a newtype wrapping `&'a [u8]` (the zero-copy view)
//! - **`SIZE_BITS` / `SIZE_BYTES`** — compile-time constants for the layout size
//! - **Accessor methods** — one `pub fn field_name(&self) -> T` per field, reading
//!   bits on demand from the underlying byte slice
//! - **`as_bytes()`** — returns the underlying `&[u8]`
//! - **`parse()` / `parse_exact()`** — associated functions for constructing views
//! - **Trait impls**: `Debug` (shows all field values), `Clone`, `Copy`,
//!   `PartialEq`, `Eq`, `BitLayout`, `Parseable`, `TryFrom<&[u8]>`, `AsRef<[u8]>`

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::extract::gen_accessor_body;
use crate::field::BitField;

/// Generate the complete output for a `#[bitframe]` struct.
pub fn gen_bitframe_output(
    vis: &syn::Visibility,
    name: &syn::Ident,
    fields: &[BitField],
) -> TokenStream {
    let total_bits: usize = fields.iter().map(|f| f.bit_width).sum();
    let total_bytes = total_bits.div_ceil(8);

    let ref_name = format_ident!("{}Ref", name);

    let accessors = gen_accessors(fields);
    let debug_impl = gen_debug_impl(&ref_name, fields);
    let parse_impl = gen_parse_impl(total_bytes);
    let bitlayout_impl = gen_bitlayout_impl(&ref_name, total_bits, total_bytes);
    let parseable_impl = gen_parseable_impl(name, &ref_name);
    let try_from_impl = gen_try_from_impl(&ref_name);
    let as_ref_impl = gen_as_ref_impl(&ref_name);

    let doc_str = format!(
        "Zero-copy view over a `{name}` layout ({total_bits} bits / {total_bytes} bytes).\n\n\
         Borrows `&[u8]` and reads fields on demand. Created via \
         [`{ref_name}::parse`] or [`{ref_name}::parse_exact`]."
    );

    quote! {
        #[doc = #doc_str]
        #[derive(Clone, Copy, PartialEq, Eq)]
        #vis struct #ref_name<'a>(&'a [u8]);

        impl<'a> #ref_name<'a> {
            /// Total size of the layout in bits.
            pub const SIZE_BITS: usize = #total_bits;

            /// Total size of the layout in bytes.
            pub const SIZE_BYTES: usize = #total_bytes;

            #accessors

            /// Returns the underlying byte slice this view covers.
            #[must_use]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.0
            }

            #parse_impl
        }

        #debug_impl
        #bitlayout_impl
        #parseable_impl
        #try_from_impl
        #as_ref_impl
    }
}

/// Generate accessor methods for all fields.
fn gen_accessors(fields: &[BitField]) -> TokenStream {
    let mut bit_offset = 0usize;
    let mut accessors = Vec::new();

    for field in fields {
        let name = &field.name;
        let ret_ty = field.field_kind.return_type_tokens();
        let body = gen_accessor_body(field, bit_offset);
        let doc = format!(
            "Reads the `{}` field ({} bit{}, offset {}).",
            name,
            field.bit_width,
            if field.bit_width == 1 { "" } else { "s" },
            bit_offset
        );

        accessors.push(quote! {
            #[doc = #doc]
            #[must_use]
            pub fn #name(&self) -> #ret_ty {
                #body
            }
        });

        bit_offset += field.bit_width;
    }

    quote! { #(#accessors)* }
}

/// Generate `Debug` impl that shows all field values.
fn gen_debug_impl(ref_name: &syn::Ident, fields: &[BitField]) -> TokenStream {
    let ref_name_str = ref_name.to_string();
    let field_debugs: Vec<TokenStream> = fields
        .iter()
        .map(|f| {
            let name = &f.name;
            let name_str = name.to_string();
            quote! { .field(#name_str, &self.#name()) }
        })
        .collect();

    quote! {
        impl<'a> core::fmt::Debug for #ref_name<'a> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(#ref_name_str)
                    #(#field_debugs)*
                    .finish()
            }
        }
    }
}

/// Generate `parse` and `parse_exact` associated functions.
fn gen_parse_impl(total_bytes: usize) -> TokenStream {
    quote! {
        /// Parses a view from the front of `bytes`, returning the view and remaining bytes.
        ///
        /// # Errors
        ///
        /// Returns [`bitframe::Error::TooShort`] if `bytes` is shorter than
        /// [`SIZE_BYTES`](Self::SIZE_BYTES).
        pub fn parse(bytes: &'a [u8]) -> Result<(Self, &'a [u8]), bitframe::Error> {
            if bytes.len() < #total_bytes {
                return Err(bitframe::Error::TooShort {
                    needed_bytes: #total_bytes,
                    have_bytes: bytes.len(),
                });
            }
            let (mine, rest) = bytes.split_at(#total_bytes);
            Ok((Self(mine), rest))
        }

        /// Parses a view from `bytes`, requiring exactly [`SIZE_BYTES`](Self::SIZE_BYTES) bytes.
        ///
        /// # Errors
        ///
        /// Returns [`bitframe::Error::TooShort`] if `bytes.len() != SIZE_BYTES`.
        pub fn parse_exact(bytes: &'a [u8]) -> Result<Self, bitframe::Error> {
            if bytes.len() != #total_bytes {
                return Err(bitframe::Error::TooShort {
                    needed_bytes: #total_bytes,
                    have_bytes: bytes.len(),
                });
            }
            Ok(Self(bytes))
        }
    }
}

/// Generate `BitLayout` trait impl.
fn gen_bitlayout_impl(ref_name: &syn::Ident, total_bits: usize, total_bytes: usize) -> TokenStream {
    quote! {
        impl<'a> bitframe::traits::BitLayout for #ref_name<'a> {
            const SIZE_BITS: usize = #total_bits;
            const SIZE_BYTES: usize = #total_bytes;
        }
    }
}

/// Generate `Parseable` trait impl on the original struct name.
fn gen_parseable_impl(name: &syn::Ident, ref_name: &syn::Ident) -> TokenStream {
    quote! {
        impl bitframe::traits::BitLayout for #name {
            const SIZE_BITS: usize = #ref_name::<'static>::SIZE_BITS;
            const SIZE_BYTES: usize = #ref_name::<'static>::SIZE_BYTES;
        }

        impl<'a> bitframe::traits::Parseable<'a> for #name {
            type View = #ref_name<'a>;

            fn parse(bytes: &'a [u8]) -> Result<(Self::View, &'a [u8]), bitframe::Error> {
                #ref_name::parse(bytes)
            }

            fn parse_exact(bytes: &'a [u8]) -> Result<Self::View, bitframe::Error> {
                #ref_name::parse_exact(bytes)
            }
        }
    }
}

/// Generate `TryFrom<&[u8]>` impl.
fn gen_try_from_impl(ref_name: &syn::Ident) -> TokenStream {
    quote! {
        impl<'a> core::convert::TryFrom<&'a [u8]> for #ref_name<'a> {
            type Error = bitframe::Error;

            fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
                Self::parse(bytes).map(|(view, _)| view)
            }
        }
    }
}

/// Generate `AsRef<[u8]>` impl.
fn gen_as_ref_impl(ref_name: &syn::Ident) -> TokenStream {
    quote! {
        impl<'a> AsRef<[u8]> for #ref_name<'a> {
            fn as_ref(&self) -> &[u8] {
                self.0
            }
        }
    }
}
