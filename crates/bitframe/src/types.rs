//! Bit-sized unsigned integer types (`u1`..`u63`).
//!
//! These newtypes represent unsigned integers that are narrower than their backing
//! storage type. They are the building blocks of `#[bitframe]` layouts — each field
//! in a bit-packed struct uses one of these types (or a standard `u8`/`u16`/`u32`/`u64`).
//!
//! # Construction
//!
//! ```
//! use bitframe::prelude::*;
//!
//! // Compile-time validated (panics if out of range in const context = compile error)
//! const APID: u11 = u11::new(42);
//!
//! // Runtime fallible construction
//! let version = u3::try_new(5)?;
//! # Ok::<(), bitframe::OutOfRange>(())
//! ```
//!
//! # Backing types
//!
//! | Range | Backing |
//! |-------|---------|
//! | `u1`..`u7` | `u8` |
//! | `u9`..`u15` | `u16` |
//! | `u17`..`u31` | `u32` |
//! | `u33`..`u63` | `u64` |

/// Error returned when a value exceeds the maximum for a bit-sized type.
///
/// # Examples
///
/// ```
/// use bitframe::prelude::*;
///
/// let err = u3::try_new(8).unwrap_err();
/// assert_eq!(err.type_name(), "u3");
/// assert_eq!(err.max(), 7);
/// assert_eq!(err.actual(), 8);
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct OutOfRange {
    type_name: &'static str,
    bits: u32,
    max: u64,
    actual: u64,
}

impl OutOfRange {
    /// Creates a new `OutOfRange` error.
    #[must_use]
    pub const fn new(type_name: &'static str, bits: u32, max: u64, actual: u64) -> Self {
        Self {
            type_name,
            bits,
            max,
            actual,
        }
    }

    /// The name of the bit-sized type (e.g. `"u3"`).
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// The bit width of the type.
    #[must_use]
    pub const fn bits(&self) -> u32 {
        self.bits
    }

    /// The maximum allowed value.
    #[must_use]
    pub const fn max(&self) -> u64 {
        self.max
    }

    /// The value that was out of range.
    #[must_use]
    pub const fn actual(&self) -> u64 {
        self.actual
    }
}

impl core::fmt::Debug for OutOfRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "OutOfRange {{ type: {}, max: {}, actual: {} }}",
            self.type_name, self.max, self.actual
        )
    }
}

impl core::fmt::Display for OutOfRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "value {} exceeds {}-bit maximum of {} for type {}",
            self.actual, self.bits, self.max, self.type_name
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OutOfRange {}

macro_rules! define_uint {
    ($name:ident, $bits:expr, $backing:ty) => {
        #[doc = concat!("A ", stringify!($bits), "-bit unsigned integer.")]
        ///
        #[doc = concat!("Backed by `", stringify!($backing), "`. Valid range: `0..=", stringify!($name), "::MAX`.")]
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("use bitframe::prelude::*;")]
        ///
        #[doc = concat!("let val = ", stringify!($name), "::new(1);")]
        #[doc = concat!("assert_eq!(val.value(), 1);")]
        /// ```
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        #[allow(non_camel_case_types)]
        pub struct $name($backing);

        impl $name {
            #[doc = concat!("The bit width of `", stringify!($name), "`.")]
            pub const WIDTH: u32 = $bits;

            #[doc = concat!("The maximum value of `", stringify!($name), "`.")]
            pub const MAX: $backing = ((1 as $backing) << $bits) - 1;

            #[doc = concat!("The zero value of `", stringify!($name), "`.")]
            pub const ZERO: Self = Self(0);

            #[doc = concat!("Creates a new `", stringify!($name), "` from a raw value.")]
            ///
            /// # Panics
            ///
            /// Panics if `val` exceeds the maximum. In `const` context, this becomes
            /// a compile-time error.
            ///
            /// # Examples
            ///
            /// ```
            #[doc = concat!("use bitframe::prelude::*;")]
            ///
            #[doc = concat!("const V: ", stringify!($name), " = ", stringify!($name), "::new(1);")]
            #[doc = concat!("assert_eq!(V.value(), 1);")]
            /// ```
            #[must_use]
            #[allow(clippy::panic)]
            pub const fn new(val: $backing) -> Self {
                if val > Self::MAX {
                    panic!(concat!(
                        "value exceeds ",
                        stringify!($bits),
                        "-bit maximum for ",
                        stringify!($name)
                    ));
                }
                Self(val)
            }

            #[doc = concat!("Tries to create a new `", stringify!($name), "` from a raw value.")]
            ///
            /// # Errors
            ///
            /// Returns `Err(OutOfRange)` if `val` exceeds the maximum.
            ///
            /// # Examples
            ///
            /// ```
            #[doc = concat!("use bitframe::prelude::*;")]
            ///
            #[doc = concat!("assert!(", stringify!($name), "::try_new(0).is_ok());")]
            #[doc = concat!("assert!(", stringify!($name), "::try_new(", stringify!($name), "::MAX + 1).is_err());")]
            /// ```
            pub const fn try_new(val: $backing) -> Result<Self, OutOfRange> {
                if val > Self::MAX {
                    Err(OutOfRange::new(
                        stringify!($name),
                        $bits,
                        Self::MAX as u64,
                        val as u64,
                    ))
                } else {
                    Ok(Self(val))
                }
            }

            #[doc = concat!("Returns the raw value as `", stringify!($backing), "`.")]
            ///
            /// # Examples
            ///
            /// ```
            #[doc = concat!("use bitframe::prelude::*;")]
            ///
            #[doc = concat!("let v = ", stringify!($name), "::new(1);")]
            #[doc = concat!("assert_eq!(v.value(), 1);")]
            /// ```
            #[must_use]
            pub const fn value(self) -> $backing {
                self.0
            }

            /// Creates from a raw value without bounds checking.
            ///
            /// Creates from a raw value without bounds checking.
            ///
            /// The value must be `<= MAX`. Used internally by generated accessor code
            /// where the mask guarantees the value is in range.
            ///
            /// **Not part of the public API.** This function is an implementation detail
            /// of the `#[bitframe]` proc-macro and may change without notice.
            #[must_use]
            #[doc(hidden)]
            pub const fn from_raw_unchecked(val: $backing) -> Self {
                Self(val)
            }
        }

        // --- Debug: shows the type name ---
        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        // --- Display: shows just the value ---
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Display::fmt(&self.0, f)
            }
        }

        // --- Formatting: Hex, Binary, Octal ---
        impl core::fmt::LowerHex for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::LowerHex::fmt(&self.0, f)
            }
        }

        impl core::fmt::UpperHex for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::UpperHex::fmt(&self.0, f)
            }
        }

        impl core::fmt::Binary for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Binary::fmt(&self.0, f)
            }
        }

        impl core::fmt::Octal for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Octal::fmt(&self.0, f)
            }
        }

        // --- From: widening to backing type ---
        impl From<$name> for $backing {
            fn from(val: $name) -> Self {
                val.0
            }
        }

        // --- TryFrom: narrowing from backing type ---
        impl TryFrom<$backing> for $name {
            type Error = OutOfRange;

            fn try_from(val: $backing) -> Result<Self, Self::Error> {
                Self::try_new(val)
            }
        }

        // --- PartialEq with backing type (both directions) ---
        impl PartialEq<$backing> for $name {
            fn eq(&self, other: &$backing) -> bool {
                self.0 == *other
            }
        }

        impl PartialEq<$name> for $backing {
            fn eq(&self, other: &$name) -> bool {
                *self == other.0
            }
        }

        // --- PartialOrd with backing type (both directions) ---
        impl PartialOrd<$backing> for $name {
            fn partial_cmp(&self, other: &$backing) -> Option<core::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }

        impl PartialOrd<$name> for $backing {
            fn partial_cmp(&self, other: &$name) -> Option<core::cmp::Ordering> {
                self.partial_cmp(&other.0)
            }
        }

        // --- Bitwise operations ---
        impl core::ops::BitAnd for $name {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }

        impl core::ops::BitOr for $name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }

        impl core::ops::BitXor for $name {
            type Output = Self;

            fn bitxor(self, rhs: Self) -> Self {
                Self(self.0 ^ rhs.0)
            }
        }

        impl core::ops::Not for $name {
            type Output = Self;

            fn not(self) -> Self {
                Self(!self.0 & Self::MAX)
            }
        }
    };
}

// u1..u7 (backed by u8)
define_uint!(u1, 1, u8);
define_uint!(u2, 2, u8);
define_uint!(u3, 3, u8);
define_uint!(u4, 4, u8);
define_uint!(u5, 5, u8);
define_uint!(u6, 6, u8);
define_uint!(u7, 7, u8);

// u9..u15 (backed by u16)
define_uint!(u9, 9, u16);
define_uint!(u10, 10, u16);
define_uint!(u11, 11, u16);
define_uint!(u12, 12, u16);
define_uint!(u13, 13, u16);
define_uint!(u14, 14, u16);
define_uint!(u15, 15, u16);

// u17..u31 (backed by u32)
define_uint!(u17, 17, u32);
define_uint!(u18, 18, u32);
define_uint!(u19, 19, u32);
define_uint!(u20, 20, u32);
define_uint!(u21, 21, u32);
define_uint!(u22, 22, u32);
define_uint!(u23, 23, u32);
define_uint!(u24, 24, u32);
define_uint!(u25, 25, u32);
define_uint!(u26, 26, u32);
define_uint!(u27, 27, u32);
define_uint!(u28, 28, u32);
define_uint!(u29, 29, u32);
define_uint!(u30, 30, u32);
define_uint!(u31, 31, u32);

// u33..u63 (backed by u64)
define_uint!(u33, 33, u64);
define_uint!(u34, 34, u64);
define_uint!(u35, 35, u64);
define_uint!(u36, 36, u64);
define_uint!(u37, 37, u64);
define_uint!(u38, 38, u64);
define_uint!(u39, 39, u64);
define_uint!(u40, 40, u64);
define_uint!(u41, 41, u64);
define_uint!(u42, 42, u64);
define_uint!(u43, 43, u64);
define_uint!(u44, 44, u64);
define_uint!(u45, 45, u64);
define_uint!(u46, 46, u64);
define_uint!(u47, 47, u64);
define_uint!(u48, 48, u64);
define_uint!(u49, 49, u64);
define_uint!(u50, 50, u64);
define_uint!(u51, 51, u64);
define_uint!(u52, 52, u64);
define_uint!(u53, 53, u64);
define_uint!(u54, 54, u64);
define_uint!(u55, 55, u64);
define_uint!(u56, 56, u64);
define_uint!(u57, 57, u64);
define_uint!(u58, 58, u64);
define_uint!(u59, 59, u64);
define_uint!(u60, 60, u64);
define_uint!(u61, 61, u64);
define_uint!(u62, 62, u64);
define_uint!(u63, 63, u64);
