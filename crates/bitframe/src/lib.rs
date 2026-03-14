//! # bitframe
//!
//! Macro-driven bit-level packet formats with zero-copy parsing.
//!
//! `bitframe` generates zero-copy **view types** over `&[u8]` from annotated struct
//! definitions. Each field is read on demand directly from the byte buffer — no
//! allocation, no copying.
//!
//! # Quick start
//!
//! ```
//! use bitframe::prelude::*;
//!
//! #[bitframe]
//! pub struct SensorReading {
//!     pub tag:    u4,
//!     pub flags:  u4,
//!     pub value:  u16,
//! }
//!
//! let bytes = [0xA5, 0x00, 0x0A];
//! let (reading, _rest) = SensorReadingRef::parse(&bytes)?;
//!
//! assert_eq!(reading.tag().value(), 0xA);
//! assert_eq!(reading.flags().value(), 0x5);
//! assert_eq!(reading.value(), 10u16);
//! # Ok::<(), bitframe::Error>(())
//! ```
//!
//! # Using `Parseable` in generic code
//!
//! ```
//! use bitframe::prelude::*;
//!
//! fn parse_and_report<'a, T: Parseable<'a>>(bytes: &'a [u8]) -> Result<T::View, Error>
//! where
//!     T::View: core::fmt::Debug,
//! {
//!     let (view, rest) = T::parse(bytes)?;
//!     // rest contains any bytes beyond the layout
//!     Ok(view)
//! }
//!
//! #[bitframe]
//! pub struct MyHeader {
//!     pub version: u4,
//!     pub length:  u12,
//! }
//!
//! let bytes = [0x10, 0x0A];
//! let header = parse_and_report::<MyHeader>(&bytes)?;
//! assert_eq!(header.version().value(), 1);
//! assert_eq!(header.length().value(), 10);
//! # Ok::<(), bitframe::Error>(())
//! ```
//!
//! # Bit-sized types
//!
//! Types `u1`..`u63` (skipping `u8`, `u16`, `u32`, `u64`) represent unsigned
//! integers narrower than their backing storage. See the [`types`] module for details.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

pub mod error;
pub mod traits;
pub mod types;

pub use bitframe_derive::{bitframe, bitframe_enum};
pub use error::Error;
pub use types::OutOfRange;

/// Re-exports everything needed for typical `bitframe` usage.
///
/// ```
/// use bitframe::prelude::*;
///
/// let v = u3::new(7);
/// assert_eq!(v, 7u8);
/// ```
pub mod prelude {
    pub use crate::error::Error;
    pub use crate::traits::{BitLayout, Parseable};
    pub use crate::types::{
        u1, u10, u11, u12, u13, u14, u15, u17, u18, u19, u2, u20, u21, u22, u23, u24, u25, u26,
        u27, u28, u29, u3, u30, u31, u33, u34, u35, u36, u37, u38, u39, u4, u40, u41, u42, u43,
        u44, u45, u46, u47, u48, u49, u5, u50, u51, u52, u53, u54, u55, u56, u57, u58, u59, u6,
        u60, u61, u62, u63, u7, u9, OutOfRange,
    };
    pub use bitframe_derive::{bitframe, bitframe_enum};
}
