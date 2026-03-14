//! `#[bitframe]` attribute expansion.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Result};

use crate::codegen::gen_bitframe_output;
use crate::field::{parse_field, validate_fields};

/// Expand a `#[bitframe]` attribute on a struct.
pub fn expand_bitframe(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = syn::parse2(item)?;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(named) => named,
            _ => {
                return Err(Error::new_spanned(
                    &input,
                    "#[bitframe] requires a struct with named fields",
                ))
            }
        },
        _ => {
            return Err(Error::new_spanned(
                &input,
                "#[bitframe] can only be applied to structs",
            ))
        }
    };

    let parsed_fields: Vec<_> = fields
        .named
        .iter()
        .map(parse_field)
        .collect::<Result<_>>()?;

    validate_fields(&parsed_fields, input.ident.span())?;

    let generated = gen_bitframe_output(&input.vis, &input.ident, &parsed_fields);

    // Re-emit the original struct (stripped of fields — it becomes a marker)
    // plus the generated view type
    let vis = &input.vis;
    let name = &input.ident;
    let doc = format!(
        "Layout definition for [`{name}Ref`]. Used as a type-level marker for \
         [`Parseable`](bitframe::traits::Parseable)."
    );

    Ok(quote! {
        #[doc = #doc]
        #vis struct #name;

        #generated
    })
}
