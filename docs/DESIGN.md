# Design

This document captures the intended API shape for `bitframe`. It is a design contract, not an implementation.

Every interface should read like a sentence. If you have to stop and think about what a method does, the name is wrong.

## Vocabulary

| Term | Meaning |
|------|---------|
| **Layout** | A compile-time description of fields (names, widths, ordering, endianness). |
| **View** | A borrowed accessor over an input `&[u8]`. Reading fields does not copy the whole struct. |
| **Writer** | An encoder that writes fields into a caller-provided `&mut [u8]`. |
| **Bit ordering** | How bits are numbered within bytes and across byte boundaries. Must be explicit. |

## Declaring a Layout

Use `#[bitframe]` on a named-field `struct`. Each field's Rust type implies its bit width:

| Type | Width | Notes |
|------|-------|-------|
| `bool` | 1 bit | |
| `u8`, `u16`, `u32`, `u64` | 8/16/32/64 bits | Standard types — returned and accepted directly |
| `u1`..`u63` | 1..63 bits | Crate-provided unsigned newtypes (skip u8/u16/u32/u64) |
| `i2`..`i63` | 2..63 bits | Signed newtypes, two's complement (v0.2+) |
| `#[bitframe_enum]` enums | inferred from variants | See Enum Support below |

### Boolean Field Naming Convention

Name boolean fields as questions — `is_`, `has_`, or bare adjectives — so the accessor reads as English:

```rust
// Good — reads as: "header.is_telecommand()", "header.has_secondary()"
#[bitframe]
pub struct CcsdsPrimaryHeader {
    pub version:        u3,
    pub is_telecommand: bool,
    pub has_secondary:  bool,
    pub apid:           u11,
    pub seq_flags:      u2,
    pub seq_count:      u14,
    pub pkt_len:        u16,
}

// The accessor reads like a question:
if header.is_telecommand() { /* ... */ }
if header.has_secondary()  { /* ... */ }
```

This is a convention, not a rule. The macro does not enforce it. But following it means your code reads like prose.

## Parsing

### `parse` — returns the view and the remaining bytes

```rust
let (header, payload) = CcsdsPrimaryHeaderRef::parse(bytes)?;
```

**Signature:** `fn parse(bytes: &[u8]) -> Result<(FooRef<'_>, &[u8]), Error>`

### `parse_exact` — rejects extra bytes

```rust
let header = CcsdsPrimaryHeaderRef::parse_exact(bytes)?;
```

**Signature:** `fn parse_exact(bytes: &[u8]) -> Result<FooRef<'_>, Error>`

### `TryFrom<&[u8]>` — parse, discard remainder

```rust
let header = CcsdsPrimaryHeaderRef::try_from(bytes)?;
```

Equivalent to `parse(bytes).map(|(h, _)| h)`.

## View Type: `FooRef<'a>`

Stores `&'a [u8]`. One accessor per field.

### Accessors return the natural type

```rust
header.version()        // -> u3     (bit-sized)
header.is_telecommand() // -> bool
header.pkt_len()        // -> u16    (standard — no wrapper)
```

Standard Rust types (`bool`, `u8`, `u16`, `u32`, `u64`) are returned directly. No newtype for types that already have the right width.

### `as_bytes` — access the underlying slice

```rust
let raw: &[u8] = header.as_bytes();
```

Returns the exact bytes this view covers. Implements `AsRef<[u8]>`.

### Trait impls

**`FooRef<'a>`**: `Debug`, `Copy`, `Clone`, `PartialEq`, `Eq`, `AsRef<[u8]>`

`Debug` reads all fields from the buffer and formats them:
```
CcsdsPrimaryHeaderRef { version: 5, is_telecommand: false, has_secondary: true, apid: 31, ... }
```

## Owned Type: `Foo`

Holds fields by value. For when parsed data must outlive the input buffer.

```rust
let owned = CcsdsPrimaryHeader::from_bytes(bytes)?;
let owned = header_ref.to_owned();  // implements ToOwned
```

**Trait impls**: `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`

**Fields are public** — access them directly without accessors:
```rust
let apid = owned.apid;       // u11
let len = owned.pkt_len;     // u16
```

## Bit-Sized Types

Unsigned newtypes (`u1`..`u63`) that feel like native integers.

### Backing types

| Range | Backing |
|-------|---------|
| `u1`..`u7` | `u8` |
| `u9`..`u15` | `u16` |
| `u17`..`u31` | `u32` |
| `u33`..`u63` | `u64` |

### Construction

```rust
u11::new(42)                    // panics if > 2047 (const fn — checked at compile time for literals)
u11::try_new(42)                // -> Result<u11, OutOfRange>
u11::ZERO                       // -> u11(0)
```

`new()` is `const fn`. For literal values, the overflow check happens at compile time:

```rust
const APID: u11 = u11::new(42);    // compile-time validated, zero cost
const BAD: u11 = u11::new(9999);   // compile error: overflow in const
```

### Getting the value out

```rust
apid.value()          // -> u16 (the stored integer)
u16::from(apid)       // -> u16 (via From)
```

### Comparison with backing type

Bit-sized types compare directly with their backing integer — no `.value()` needed for assertions:

```rust
assert_eq!(header.apid(), 31_u16);            // u11 == u16
assert!(header.seq_count() > 0_u16);          // u14 > u16
assert_eq!(header.version(), u3::new(5));      // u3 == u3
```

Implemented via `PartialEq<u16> for u11` and `PartialOrd<u16> for u11`.

### Formatting

```rust
format!("{apid}")         // "42"           (Display)
format!("{apid:?}")       // "u11(42)"      (Debug — shows the type)
format!("{apid:#x}")      // "0x2a"         (LowerHex)
format!("{apid:#b}")      // "0b101010"     (Binary)
format!("{apid:#o}")      // "0o52"         (Octal)
```

### Bitwise operations

Bitwise ops on same-width types always produce the same type (no overflow possible):

```rust
let masked  = flags & u3::new(0b101);     // BitAnd
let merged  = flags | u3::new(0b010);     // BitOr
let flipped = flags ^ u3::new(0b111);     // BitXor
let inverted = !flags;                     // Not (masks to 3 bits)
```

### Full trait list

`Debug`, `Display`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`,
`From<u11> for u16` (widening), `TryFrom<u16> for u11` (narrowing),
`PartialEq<u16>`, `PartialOrd<u16>` (cross-type comparison),
`BitAnd`, `BitOr`, `BitXor`, `Not`,
`LowerHex`, `UpperHex`, `Binary`, `Octal`

## Enum Support

### Declaration — bit width is inferred

```rust
#[bitframe_enum]
#[repr(u8)]
pub enum SeqFlags {
    Continuation = 0,
    First        = 1,
    Last         = 2,
    Standalone   = 3,
}
// Bit width: automatically inferred as 2 (ceil(log2(max_discriminant + 1)))
```

Convention over configuration. The macro computes the minimum bit width from the discriminants. No `bits = N` required.

**Explicit width** only when you want a wider wire width than the minimum (e.g., leaving room for future variants):

```rust
#[bitframe_enum(bits = 3)]   // 3-bit field, even though values only need 2 bits
#[repr(u8)]
pub enum Priority {
    Low    = 0,
    Medium = 1,
    High   = 2,
}
```

### Exhaustive enums — direct return, no `Result`

When every possible bit pattern has a variant:

```rust
let flags: SeqFlags = header.seq_flags();  // infallible
```

### Non-exhaustive enums — `Result`

When gaps exist in the bit patterns:

```rust
match header.priority() {
    Ok(Priority::High) => { /* ... */ }
    Err(e) => log::warn!("unknown priority: {e}"),
}
```

### Raw accessor — always available

```rust
let raw: u2 = header.seq_flags_raw();     // always succeeds
```

## Padding Fields

`_`-prefixed fields are padding — readable but not writable:

```rust
pub _reserved: u5,   // accessor generated, no writer setter, zeroed on encode
```

Every bit in the layout is accounted for. No implicit gaps.

## Encoding (v0.2+)

### Writer — accepts plain values, validates for you

```rust
let mut buf = [0u8; CcsdsPrimaryHeader::SIZE_BYTES];

CcsdsPrimaryHeaderWriter::new(&mut buf)
    .version(0)             // just a number — validated against u3 range
    .is_telecommand(false)  // just a bool
    .has_secondary(true)    // just a bool
    .apid(42)               // just a number — validated against u11 range
    .seq_flags(3)           // just a number — validated against u2 range
    .seq_count(1)           // just a number — validated against u14 range
    .pkt_len(100)           // u16 — no validation needed
    .finalize()?;
```

**You write values. The library checks the range.** Each setter accepts the backing integer type (`u8` for u1-u7 fields, `u16` for u9-u15, etc.) and panics if the value exceeds the field's max. This matches `u3::new()` behavior — the user never wraps values in newtypes just to write them.

For standard types (`bool`, `u8`, `u16`, `u32`, `u64`), the setter accepts the type directly. No range check needed — the types already have the right width.

If you already have a bit-sized type, pass its value:

```rust
let existing: u11 = some_source();
writer.apid(existing.value());
```

`finalize()` writes padding fields as zero and returns `Ok(())`.

### `encode_to` on Owned Type

```rust
header.encode_to(&mut buf)?;
```

Roundtrip guarantee: `Foo::from_bytes(bytes)?.encode_to(&mut buf)` produces `buf == bytes[..SIZE_BYTES]`.

## Mutable Views (v0.3+)

Setters accept plain values, same as the Writer:

```rust
let (mut header, _) = CcsdsPrimaryHeaderRefMut::parse(&mut bytes)?;
header.set_seq_count(999);   // just a number — validated against u14 range
header.set_apid(42);         // just a number — validated against u11 range
```

In-place modification. No copy. No rebuild. Just the bits that changed.

## Nested Struct Composition (v0.3+)

```rust
#[bitframe]
pub struct TmPacket {
    pub primary:   CcsdsPrimaryHeader,
    pub secondary: TmSecondaryHeader,
}
```

Accessor returns a sub-view: `fn primary(&self) -> CcsdsPrimaryHeaderRef<'_>`.

## `BitLayout` Trait

Every `#[bitframe]` type implements the `BitLayout` trait for generic code:

```rust
pub trait BitLayout {
    const SIZE_BITS: usize;
    const SIZE_BYTES: usize;
}

pub trait Parseable<'a>: BitLayout {
    type View: Copy;
    fn parse(bytes: &'a [u8]) -> Result<(Self::View, &'a [u8]), Error>;
    fn parse_exact(bytes: &'a [u8]) -> Result<Self::View, Error>;
}
```

This enables:

```rust
fn parse_and_log<'a, T: Parseable<'a>>(bytes: &'a [u8]) -> Result<T::View, Error>
where T::View: core::fmt::Debug {
    let (view, _) = T::parse(bytes)?;
    log::debug!("parsed {}-byte header: {view:?}", T::SIZE_BYTES);
    Ok(view)
}
```

## Errors

Structured, deterministic, machine-readable:

```rust
pub enum Error {
    /// Buffer too short for the layout.
    TooShort { needed_bytes: usize, have_bytes: usize },

    /// Enum field contains an unrecognized value.
    InvalidEnum { field: &'static str, raw: u64 },
}
```

`Display` includes context:
- `"buffer too short for CcsdsPrimaryHeader: need 6 bytes, have 2"`
- `"invalid Priority in field 'priority': raw value 7"`

`std::error::Error` behind the `std` feature flag.

## Compile-Time Size Constants

```rust
CcsdsPrimaryHeader::SIZE_BITS   // 48
CcsdsPrimaryHeader::SIZE_BYTES  // 6
```

Available on `FooRef`, `Foo`, and via the `BitLayout` trait.

## Endianness

Default: **big-endian** (network order). Override per-field:

```rust
#[bitframe(little_endian)]
pub host_field: u16,
```

Attributes use Rust identifiers, not strings. Sub-byte fields are unambiguous — endianness only applies to multi-byte fields.

## Byte-Aligned Field Optimization

When fields align on byte boundaries, the generated code uses direct byte reads (`u16::from_be_bytes(...)`) instead of shifting. Invisible to the user — the API is identical regardless of alignment.

## Scope Boundaries

Out of scope for v1.0: streaming/incremental parsing, variable-length fields, runtime format descriptions, in-memory C-style bitfields.

## Guardrails

- No implicit padding — every bit must be declared.
- Total layout size known at compile time.
- `#![forbid(unsafe_code)]` on all crates.
- Generated code readable via `cargo expand`.
