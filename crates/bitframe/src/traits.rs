//! Core traits for bitframe layout types.
//!
//! These traits are implemented automatically by the `#[bitframe]` proc-macro.
//! They enable generic code over any bitframe layout.
//!
//! # Examples
//!
//! ```
//! use bitframe::traits::BitLayout;
//!
//! fn print_size<T: BitLayout>() {
//!     // Use T::SIZE_BITS and T::SIZE_BYTES in generic code
//! }
//! ```

use crate::error::Error;

/// Compile-time size information for a bit-level layout.
///
/// Every `#[bitframe]` struct implements this trait, providing the total
/// size in bits and bytes. `SIZE_BYTES` is always `ceil(SIZE_BITS / 8)`.
///
/// # Examples
///
/// ```
/// use bitframe::traits::BitLayout;
///
/// // Implemented by generated types:
/// // assert_eq!(CcsdsPrimaryHeaderRef::SIZE_BITS, 48);
/// // assert_eq!(CcsdsPrimaryHeaderRef::SIZE_BYTES, 6);
/// ```
pub trait BitLayout {
    /// Total size of the layout in bits.
    const SIZE_BITS: usize;

    /// Total size of the layout in bytes (`ceil(SIZE_BITS / 8)`).
    const SIZE_BYTES: usize;
}

/// Zero-copy parsing from a byte slice.
///
/// Implemented by `#[bitframe]` structs. The `View` associated type is the
/// generated `FooRef<'a>` that borrows the input bytes.
///
/// # Examples
///
/// ```
/// use bitframe::prelude::*;
///
/// #[bitframe]
/// pub struct Msg {
///     pub id:   u4,
///     pub data: u12,
/// }
///
/// fn parse_any<'a, T: Parseable<'a>>(bytes: &'a [u8]) -> Result<T::View, bitframe::Error>
/// where T::View: core::fmt::Debug {
///     let (view, _rest) = T::parse(bytes)?;
///     Ok(view)
/// }
///
/// let view = parse_any::<Msg>(&[0x1F, 0xFF])?;
/// # Ok::<(), bitframe::Error>(())
/// ```
pub trait Parseable<'a>: BitLayout {
    /// The view type that borrows the input bytes.
    type View: Copy;

    /// Parses a view from the front of `bytes`, returning the view and remaining bytes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooShort`] if `bytes.len() < Self::SIZE_BYTES`.
    fn parse(bytes: &'a [u8]) -> Result<(Self::View, &'a [u8]), Error>;

    /// Parses a view from `bytes`, requiring the slice to be exactly `SIZE_BYTES` long.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooShort`] if `bytes.len() != Self::SIZE_BYTES`.
    fn parse_exact(bytes: &'a [u8]) -> Result<Self::View, Error>;
}
