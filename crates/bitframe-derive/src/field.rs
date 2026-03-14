//! Field type parsing — maps Rust types to bit widths.

use proc_macro2::Span;
use syn::{Error, Result, Type};

/// A parsed field from a `#[bitframe]` struct.
#[derive(Debug)]
pub struct BitField {
    pub name: syn::Ident,
    pub bit_width: usize,
    pub field_kind: FieldKind,
    #[allow(dead_code)]
    pub vis: syn::Visibility,
}

/// What kind of value a field produces.
#[derive(Debug, Clone)]
pub enum FieldKind {
    /// `bool` — 1 bit
    Bool,
    /// Standard unsigned integer (`u8`, `u16`, `u32`, `u64`)
    StdUint { bits: usize },
    /// Bit-sized type (`u1`..`u63`)
    BitSized { bits: usize },
}

impl FieldKind {
    /// Returns the Rust type token for the accessor return type.
    pub fn return_type_tokens(&self) -> proc_macro2::TokenStream {
        use quote::quote;
        match self {
            Self::Bool => quote!(bool),
            Self::StdUint { bits: 8 } => quote!(u8),
            Self::StdUint { bits: 16 } => quote!(u16),
            Self::StdUint { bits: 32 } => quote!(u32),
            Self::StdUint { bits: 64 } => quote!(u64),
            Self::StdUint { .. } => unreachable!(),
            Self::BitSized { bits } => {
                let name = quote::format_ident!("u{bits}");
                quote!(bitframe::types::#name)
            }
        }
    }

    /// Returns the backing primitive type used for intermediate extraction.
    pub fn backing_type_tokens(&self) -> proc_macro2::TokenStream {
        use quote::quote;
        match self {
            Self::Bool | Self::StdUint { bits: 8 } => quote!(u8),
            Self::StdUint { bits: 16 } => quote!(u16),
            Self::StdUint { bits: 32 } => quote!(u32),
            Self::StdUint { bits: 64 } => quote!(u64),
            Self::StdUint { .. } => unreachable!(),
            Self::BitSized { bits } if *bits <= 7 => quote!(u8),
            Self::BitSized { bits } if *bits <= 15 => quote!(u16),
            Self::BitSized { bits } if *bits <= 31 => quote!(u32),
            Self::BitSized { .. } => quote!(u64),
        }
    }
}

/// Parse a struct field's type into a `BitField`.
pub fn parse_field(field: &syn::Field) -> Result<BitField> {
    let name = field
        .ident
        .clone()
        .ok_or_else(|| Error::new_spanned(field, "bitframe structs require named fields"))?;

    let (bit_width, field_kind) = parse_type(&field.ty)?;

    Ok(BitField {
        name,
        bit_width,
        field_kind,
        vis: field.vis.clone(),
    })
}

/// Map a type to its bit width and kind.
fn parse_type(ty: &Type) -> Result<(usize, FieldKind)> {
    let path = match ty {
        Type::Path(tp) => &tp.path,
        _ => {
            return Err(Error::new_spanned(
                ty,
                "unsupported type; supported: bool, u8, u16, u32, u64, u1..u63",
            ))
        }
    };

    let ident = path
        .get_ident()
        .ok_or_else(|| {
            Error::new_spanned(
                ty,
                "unsupported type; supported: bool, u8, u16, u32, u64, u1..u63",
            )
        })?
        .to_string();

    match ident.as_str() {
        "bool" => Ok((1, FieldKind::Bool)),
        "u8" => Ok((8, FieldKind::StdUint { bits: 8 })),
        "u16" => Ok((16, FieldKind::StdUint { bits: 16 })),
        "u32" => Ok((32, FieldKind::StdUint { bits: 32 })),
        "u64" => Ok((64, FieldKind::StdUint { bits: 64 })),
        s if s.starts_with('u') => {
            let bits_str = &s[1..];
            let bits: usize = bits_str.parse().map_err(|_| {
                Error::new_spanned(
                    ty,
                    format!(
                        "unsupported type `{ident}`; supported: bool, u8, u16, u32, u64, u1..u63"
                    ),
                )
            })?;
            if bits == 0 || bits > 63 {
                return Err(Error::new_spanned(
                    ty,
                    format!(
                        "bit width {bits} is out of range; supported: u1..u63 (and u8, u16, u32, u64)"
                    ),
                ));
            }
            // u8, u16, u32, u64 are already handled above
            Ok((bits, FieldKind::BitSized { bits }))
        }
        _ => Err(Error::new_spanned(
            ty,
            format!("unsupported type `{ident}`; supported: bool, u8, u16, u32, u64, u1..u63"),
        )),
    }
}

/// Validate that the total layout has at least one field and a computable size.
pub fn validate_fields(fields: &[BitField], span: Span) -> Result<()> {
    if fields.is_empty() {
        return Err(Error::new(
            span,
            "#[bitframe] structs must have at least one field",
        ));
    }
    Ok(())
}
