//! Error types for bitframe parsing operations.
//!
//! All errors are structured, deterministic, and machine-readable. `Display`
//! provides human-friendly context; fields are accessible for programmatic use.
//!
//! # Examples
//!
//! ```
//! use bitframe::Error;
//!
//! let err = Error::TooShort { needed_bytes: 6, have_bytes: 2 };
//! assert_eq!(format!("{err}"), "buffer too short: need 6 bytes, have 2");
//! ```

/// Errors returned by bitframe parsing operations.
///
/// Each variant carries enough context to produce a useful diagnostic message.
/// Field values are intentionally public for pattern matching.
///
/// # Examples
///
/// Matching on a short-buffer error:
///
/// ```
/// use bitframe::Error;
///
/// let err = Error::TooShort { needed_bytes: 6, have_bytes: 2 };
/// match err {
///     Error::TooShort { needed_bytes, have_bytes } => {
///         assert_eq!(needed_bytes, 6);
///         assert_eq!(have_bytes, 2);
///     }
///     Error::InvalidEnum { .. } => unreachable!(),
/// }
/// ```
///
/// Matching on an invalid enum discriminant:
///
/// ```
/// use bitframe::Error;
///
/// let err = Error::InvalidEnum { field: "priority", raw: 7 };
/// match err {
///     Error::InvalidEnum { field, raw } => {
///         assert_eq!(field, "priority");
///         assert_eq!(raw, 7);
///     }
///     Error::TooShort { .. } => unreachable!(),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Buffer too short for the layout.
    ///
    /// The input slice did not contain enough bytes to cover the declared layout.
    TooShort {
        /// Minimum bytes required by the layout.
        needed_bytes: usize,
        /// Bytes actually provided.
        have_bytes: usize,
    },

    /// Enum field contains an unrecognized discriminant value.
    ///
    /// Returned when a non-exhaustive enum encounters a bit pattern that
    /// does not match any declared variant.
    InvalidEnum {
        /// The field name in the layout (e.g. `"priority"`).
        field: &'static str,
        /// The raw integer value read from the buffer.
        raw: u64,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TooShort {
                needed_bytes,
                have_bytes,
            } => {
                write!(
                    f,
                    "buffer too short: need {needed_bytes} bytes, have {have_bytes}"
                )
            }
            Self::InvalidEnum { field, raw } => {
                write!(f, "invalid enum value in field '{field}': raw value {raw}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
