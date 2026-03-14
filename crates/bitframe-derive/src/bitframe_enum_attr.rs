//! `#[bitframe_enum]` attribute expansion.
//!
//! Generates bit-width-aware conversions for enums used in `#[bitframe]` layouts.
//!
//! # Exhaustive vs non-exhaustive detection
//!
//! An enum is **exhaustive** when it has exactly `2^bits` variants covering every
//! possible bit pattern (e.g., a 2-bit enum with variants for 0, 1, 2, and 3).
//!
//! - **Exhaustive**: `from_raw` is infallible — returns the enum directly.
//! - **Non-exhaustive**: `from_raw` is fallible — returns `Result<Self, Error>`,
//!   where unrecognized discriminants produce `Error::InvalidEnum`.
//!
//! # Bit width determination
//!
//! The bit width is resolved in priority order:
//!
//! 1. Explicit: `#[bitframe_enum(bits = N)]`
//! 2. Inferred: `ceil(log2(max_discriminant + 1))` — the minimum bits needed to
//!    represent the largest discriminant value

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Error, Expr, Lit, Meta, Result};

/// Expand a `#[bitframe_enum]` attribute on an enum.
pub fn expand_bitframe_enum(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = syn::parse2(item)?;

    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(
            &input,
            "#[bitframe_enum] can only be applied to enums",
        ));
    };

    if data.variants.is_empty() {
        return Err(Error::new_spanned(
            &input,
            "#[bitframe_enum] requires at least one variant",
        ));
    }

    // Parse explicit bits from attribute: #[bitframe_enum(bits = N)]
    let explicit_bits = parse_bits_attr(attr)?;

    // Collect all discriminant values
    let discriminants = collect_discriminants(data)?;

    // Determine bit width
    let bits = determine_bits(&discriminants, explicit_bits, &input)?;

    // Validate all discriminants fit in the bit width
    validate_discriminants(&discriminants, bits)?;

    // Determine if enum is exhaustive (all 2^bits values have variants)
    let is_exhaustive = check_exhaustive(&discriminants, bits);

    let vis = &input.vis;
    let name = &input.ident;
    let attrs = &input.attrs;

    let generated = gen_enum_impls(name, &discriminants, bits, is_exhaustive);

    // Re-emit the original enum with its attributes
    let variants: Vec<TokenStream> = data
        .variants
        .iter()
        .map(|v| {
            let vname = &v.ident;
            if let Some((eq, expr)) = &v.discriminant {
                quote! { #vname #eq #expr }
            } else {
                quote! { #vname }
            }
        })
        .collect();

    let width_doc = format!("Bit width: {bits}.");

    Ok(quote! {
        #(#attrs)*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #vis enum #name {
            #(#variants,)*
        }

        impl #name {
            #[doc = #width_doc]
            pub const WIDTH: usize = #bits;

            #generated
        }
    })
}

/// Parse `bits = N` from `#[bitframe_enum(bits = N)]`.
fn parse_bits_attr(attr: TokenStream) -> Result<Option<usize>> {
    if attr.is_empty() {
        return Ok(None);
    }

    let meta: Meta = syn::parse2(attr)?;
    match meta {
        Meta::NameValue(nv) if nv.path.is_ident("bits") => match &nv.value {
            Expr::Lit(lit) => match &lit.lit {
                Lit::Int(int_lit) => Ok(Some(int_lit.base10_parse::<usize>()?)),
                _ => Err(Error::new_spanned(&nv.value, "expected integer for `bits`")),
            },
            _ => Err(Error::new_spanned(&nv.value, "expected integer for `bits`")),
        },
        _ => Err(Error::new_spanned(
            &meta,
            "expected `bits = N` (e.g., #[bitframe_enum(bits = 3)])",
        )),
    }
}

/// Collect discriminant values from enum variants.
fn collect_discriminants(data: &syn::DataEnum) -> Result<Vec<(syn::Ident, u64)>> {
    let mut discriminants = Vec::new();
    let mut next_discriminant: u64 = 0;

    for variant in &data.variants {
        if !variant.fields.is_empty() {
            return Err(Error::new_spanned(
                variant,
                "#[bitframe_enum] variants must be unit variants (no fields)",
            ));
        }

        let disc_val = if let Some((_, expr)) = &variant.discriminant {
            parse_discriminant_expr(expr)?
        } else {
            next_discriminant
        };

        discriminants.push((variant.ident.clone(), disc_val));
        next_discriminant = disc_val + 1;
    }

    Ok(discriminants)
}

/// Parse a discriminant expression to get its integer value.
fn parse_discriminant_expr(expr: &Expr) -> Result<u64> {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Int(int_lit) => int_lit.base10_parse::<u64>(),
            _ => Err(Error::new_spanned(
                expr,
                "discriminant must be an integer literal",
            )),
        },
        _ => Err(Error::new_spanned(
            expr,
            "discriminant must be an integer literal",
        )),
    }
}

/// Determine the bit width for the enum.
fn determine_bits(
    discriminants: &[(syn::Ident, u64)],
    explicit_bits: Option<usize>,
    input: &DeriveInput,
) -> Result<usize> {
    let max_disc = discriminants.iter().map(|(_, v)| *v).max().unwrap_or(0);
    let inferred_bits = if max_disc == 0 {
        1
    } else {
        (u64::BITS - max_disc.leading_zeros()) as usize
    };

    let bits = explicit_bits.unwrap_or(inferred_bits);

    if bits > 63 {
        return Err(Error::new_spanned(input, "enum bit width cannot exceed 63"));
    }

    Ok(bits)
}

/// Validate all discriminants fit within the bit width.
fn validate_discriminants(discriminants: &[(syn::Ident, u64)], bits: usize) -> Result<()> {
    let max_val = (1u64 << bits) - 1;
    for (name, val) in discriminants {
        if *val > max_val {
            return Err(Error::new_spanned(
                name,
                format!("discriminant {val} exceeds {bits}-bit maximum of {max_val}"),
            ));
        }
    }
    Ok(())
}

/// Check if all values 0..2^bits are covered by discriminants.
fn check_exhaustive(discriminants: &[(syn::Ident, u64)], bits: usize) -> bool {
    let total = 1u64 << bits;
    // For large bit widths, can't be exhaustive unless we have exactly 2^bits variants
    if discriminants.len() as u64 != total {
        return false;
    }
    all_values_covered(discriminants, bits)
}

/// Check if all values 0..2^bits are covered by discriminants.
#[allow(clippy::cast_possible_truncation)]
fn all_values_covered(discriminants: &[(syn::Ident, u64)], bits: usize) -> bool {
    let total = 1u64 << bits;
    let mut seen = vec![false; total as usize];
    for (_, val) in discriminants {
        if *val < total {
            seen[*val as usize] = true;
        }
    }
    seen.iter().all(|&v| v)
}

/// Get the bitframe type name and backing type for a given bit width.
fn bit_type_info(bits: usize) -> (TokenStream, TokenStream) {
    match bits {
        8 => (quote!(u8), quote!(u8)),
        16 => (quote!(u16), quote!(u16)),
        32 => (quote!(u32), quote!(u32)),
        64 => (quote!(u64), quote!(u64)),
        b if b <= 7 => {
            let name = format_ident!("u{b}");
            (quote!(bitframe::types::#name), quote!(u8))
        }
        b if b <= 15 => {
            let name = format_ident!("u{b}");
            (quote!(bitframe::types::#name), quote!(u16))
        }
        b if b <= 31 => {
            let name = format_ident!("u{b}");
            (quote!(bitframe::types::#name), quote!(u32))
        }
        b => {
            let name = format_ident!("u{b}");
            (quote!(bitframe::types::#name), quote!(u64))
        }
    }
}

/// Generate `from_raw` and `to_raw` methods.
fn gen_enum_impls(
    name: &syn::Ident,
    discriminants: &[(syn::Ident, u64)],
    bits: usize,
    is_exhaustive: bool,
) -> TokenStream {
    let (width_type, backing_type) = bit_type_info(bits);
    let is_std_width = matches!(bits, 8 | 16 | 32 | 64);

    let to_raw_fn = gen_to_raw(discriminants, is_std_width, &width_type, &backing_type);
    let from_raw_fn = gen_from_raw(
        name,
        discriminants,
        is_exhaustive,
        is_std_width,
        &width_type,
        &backing_type,
    );

    quote! {
        #from_raw_fn
        #to_raw_fn
    }
}

/// Generate the `to_raw` method.
fn gen_to_raw(
    discriminants: &[(syn::Ident, u64)],
    is_std_width: bool,
    width_type: &TokenStream,
    backing_type: &TokenStream,
) -> TokenStream {
    let to_raw_arms: Vec<TokenStream> = discriminants
        .iter()
        .map(|(vname, val)| {
            let lit = proc_macro2::Literal::u64_unsuffixed(*val);
            quote! { Self::#vname => #lit }
        })
        .collect();

    if is_std_width {
        quote! {
            /// Converts this variant to its raw integer value.
            #[must_use]
            pub const fn to_raw(self) -> #backing_type {
                match self {
                    #(#to_raw_arms,)*
                }
            }
        }
    } else {
        quote! {
            /// Converts this variant to its raw bit-sized value.
            #[must_use]
            pub const fn to_raw(self) -> #width_type {
                let val: #backing_type = match self {
                    #(#to_raw_arms,)*
                };
                #width_type::from_raw_unchecked(val)
            }
        }
    }
}

/// Generate the `from_raw` method.
fn gen_from_raw(
    name: &syn::Ident,
    discriminants: &[(syn::Ident, u64)],
    is_exhaustive: bool,
    is_std_width: bool,
    width_type: &TokenStream,
    backing_type: &TokenStream,
) -> TokenStream {
    if is_exhaustive {
        gen_from_raw_exhaustive(discriminants, is_std_width, width_type, backing_type)
    } else {
        gen_from_raw_fallible(name, discriminants, is_std_width, width_type, backing_type)
    }
}

/// Generate infallible `from_raw` for exhaustive enums.
fn gen_from_raw_exhaustive(
    discriminants: &[(syn::Ident, u64)],
    is_std_width: bool,
    width_type: &TokenStream,
    backing_type: &TokenStream,
) -> TokenStream {
    let from_raw_arms: Vec<TokenStream> = discriminants
        .iter()
        .map(|(vname, val)| {
            let lit = proc_macro2::Literal::u64_unsuffixed(*val);
            quote! { #lit => Self::#vname }
        })
        .collect();

    if is_std_width {
        quote! {
            /// Converts a raw integer value to this enum.
            ///
            /// This enum is exhaustive — every valid bit pattern maps to a variant.
            #[must_use]
            #[allow(clippy::panic)]
            pub const fn from_raw(val: #backing_type) -> Self {
                match val {
                    #(#from_raw_arms,)*
                    _ => unreachable!(),
                }
            }
        }
    } else {
        quote! {
            /// Converts a raw bit-sized value to this enum.
            ///
            /// This enum is exhaustive — every valid bit pattern maps to a variant.
            #[must_use]
            #[allow(clippy::panic)]
            pub const fn from_raw(val: #width_type) -> Self {
                match val.value() {
                    #(#from_raw_arms,)*
                    _ => unreachable!(),
                }
            }
        }
    }
}

/// Generate fallible `from_raw` for non-exhaustive enums.
fn gen_from_raw_fallible(
    name: &syn::Ident,
    discriminants: &[(syn::Ident, u64)],
    is_std_width: bool,
    width_type: &TokenStream,
    backing_type: &TokenStream,
) -> TokenStream {
    let from_raw_arms: Vec<TokenStream> = discriminants
        .iter()
        .map(|(vname, val)| {
            let lit = proc_macro2::Literal::u64_unsuffixed(*val);
            quote! { #lit => Ok(Self::#vname) }
        })
        .collect();

    if is_std_width {
        quote! {
            /// Tries to convert a raw integer value to this enum.
            ///
            /// # Errors
            ///
            /// Returns [`bitframe::Error::InvalidEnum`] if the value doesn't match
            /// any variant.
            pub const fn from_raw(val: #backing_type) -> Result<Self, bitframe::Error> {
                match val {
                    #(#from_raw_arms,)*
                    _ => Err(bitframe::Error::InvalidEnum {
                        field: stringify!(#name),
                        raw: val as u64,
                    }),
                }
            }
        }
    } else {
        quote! {
            /// Tries to convert a raw bit-sized value to this enum.
            ///
            /// # Errors
            ///
            /// Returns [`bitframe::Error::InvalidEnum`] if the value doesn't match
            /// any variant.
            pub const fn from_raw(val: #width_type) -> Result<Self, bitframe::Error> {
                match val.value() {
                    #(#from_raw_arms,)*
                    _ => Err(bitframe::Error::InvalidEnum {
                        field: stringify!(#name),
                        raw: val.value() as u64,
                    }),
                }
            }
        }
    }
}
