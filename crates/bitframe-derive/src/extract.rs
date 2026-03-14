//! Bit extraction code generation.
//!
//! Generates the token stream that reads bits from a `&[u8]` slice for each field.
//!
//! # Bit numbering
//!
//! bitframe uses **`MSb0`** (Most Significant bit first) numbering with big-endian
//! byte order. Bit 0 is the most significant bit of byte 0.
//!
//! ```text
//!          Byte 0              Byte 1
//!   ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐
//!   │ 0 │ 1 │ 2 │ 3 │ 4 │ 5 │ 6 │ 7 │ 8 │ 9 │10 │11 │12 │13 │14 │15 │
//!   └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘
//! ```
//!
//! # Extraction algorithm
//!
//! Given a field at `bit_offset` with `bit_width` bits:
//!
//! 1. **Byte range**: `byte_start = bit_offset / 8`, `byte_end = (bit_offset + bit_width - 1) / 8`
//! 2. **Combine bytes**: Read `byte_start..=byte_end` into a `u64`, shifting each byte
//!    into position: `(bytes[i] as u64) << ((num_bytes - 1 - i) * 8)`
//! 3. **Right-shift**: Shift right by `bits_after = (byte_end + 1) * 8 - (bit_offset + bit_width)`
//!    to align the field's LSB to bit 0
//! 4. **Mask**: AND with `(1 << bit_width) - 1` to isolate the field
//! 5. **Convert**: Cast to the field's return type (or wrap in a bit-sized newtype)
//!
//! # Byte-aligned optimization
//!
//! When `bit_offset % 8 == 0` and the field is a standard-width type (u8, u16, u32, u64),
//! the extraction skips the general shift-and-mask path and emits a direct
//! `uN::from_be_bytes(...)` call instead.

use proc_macro2::TokenStream;
use quote::quote;

use crate::field::{BitField, FieldKind};

/// Generate the accessor body that extracts a field's value from `self.0` (the byte slice).
pub fn gen_accessor_body(field: &BitField, bit_offset: usize) -> TokenStream {
    // Check for byte-aligned optimization
    if bit_offset % 8 == 0 {
        if let Some(optimized) = gen_byte_aligned_read(field, bit_offset) {
            return optimized;
        }
    }

    // General case: read spanning bytes, shift, and mask
    gen_general_read(field, bit_offset)
}

/// Try to generate an optimized direct byte read for byte-aligned fields.
fn gen_byte_aligned_read(field: &BitField, bit_offset: usize) -> Option<TokenStream> {
    let byte_start = bit_offset / 8;

    match &field.field_kind {
        FieldKind::Bool if field.bit_width == 1 => {
            let byte_idx = byte_start;
            Some(quote! {
                (self.0[#byte_idx] >> 7) != 0
            })
        }
        FieldKind::StdUint { bits: 8 } => {
            let byte_idx = byte_start;
            Some(quote! {
                self.0[#byte_idx]
            })
        }
        FieldKind::StdUint { bits: 16 } => {
            let b0 = byte_start;
            let b1 = byte_start + 1;
            Some(quote! {
                u16::from_be_bytes([self.0[#b0], self.0[#b1]])
            })
        }
        FieldKind::StdUint { bits: 32 } => {
            let b0 = byte_start;
            let b1 = byte_start + 1;
            let b2 = byte_start + 2;
            let b3 = byte_start + 3;
            Some(quote! {
                u32::from_be_bytes([self.0[#b0], self.0[#b1], self.0[#b2], self.0[#b3]])
            })
        }
        FieldKind::StdUint { bits: 64 } => {
            let b0 = byte_start;
            let b1 = byte_start + 1;
            let b2 = byte_start + 2;
            let b3 = byte_start + 3;
            let b4 = byte_start + 4;
            let b5 = byte_start + 5;
            let b6 = byte_start + 6;
            let b7 = byte_start + 7;
            Some(quote! {
                u64::from_be_bytes([
                    self.0[#b0], self.0[#b1], self.0[#b2], self.0[#b3],
                    self.0[#b4], self.0[#b5], self.0[#b6], self.0[#b7],
                ])
            })
        }
        _ => None, // Non-standard or bit-sized types at byte boundary still use general path
    }
}

/// General bit extraction: works for any bit offset and width.
fn gen_general_read(field: &BitField, bit_offset: usize) -> TokenStream {
    let bit_width = field.bit_width;
    let byte_start = bit_offset / 8;
    let byte_end = (bit_offset + bit_width - 1) / 8;
    let num_bytes = byte_end - byte_start + 1;

    // Read bytes and combine into a u64
    let byte_reads: Vec<TokenStream> = (0..num_bytes)
        .map(|i| {
            let idx = byte_start + i;
            let shift = (num_bytes - 1 - i) * 8;
            if shift > 0 {
                quote! { ((self.0[#idx] as u64) << #shift) }
            } else {
                quote! { (self.0[#idx] as u64) }
            }
        })
        .collect();

    let combined = if byte_reads.len() == 1 {
        byte_reads[0].clone()
    } else {
        quote! { #(#byte_reads)|* }
    };

    // Shift right to align the field bits to LSB
    let bits_after = (byte_end + 1) * 8 - (bit_offset + bit_width);
    let shifted = if bits_after > 0 {
        quote! { (#combined) >> #bits_after }
    } else {
        quote! { #combined }
    };

    // Mask to bit_width
    let mask = if bit_width >= 64 {
        quote! { #shifted }
    } else {
        let mask_val: u64 = (1u64 << bit_width) - 1;
        quote! { (#shifted) & #mask_val }
    };

    // Convert to the appropriate return type
    match &field.field_kind {
        FieldKind::Bool => {
            quote! { (#mask) != 0 }
        }
        FieldKind::StdUint { .. } => {
            let backing = field.field_kind.backing_type_tokens();
            quote! { (#mask) as #backing }
        }
        FieldKind::BitSized { bits } => {
            let type_name = quote::format_ident!("u{bits}");
            let backing = field.field_kind.backing_type_tokens();
            quote! { bitframe::types::#type_name::from_raw_unchecked((#mask) as #backing) }
        }
    }
}
