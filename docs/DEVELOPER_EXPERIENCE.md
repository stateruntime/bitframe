# Developer Experience

Using `bitframe` should feel like the language already supports bit-packed structs. If something surprises you, we got it wrong.

Three rules:

1. **The struct is the spec.** If you can read the struct, you understand the protocol.
2. **You write values, not types.** The library checks the range. You never wrap a number in a newtype just to pass it somewhere.
3. **If you can guess the method name, you're right.** No surprises, no abbreviations, no cleverness.

---

## Your First 30 Seconds

```rust
use bitframe::prelude::*;

#[bitframe]
pub struct MyHeader {
    pub tag:    u4,
    pub flags:  u4,
    pub length: u16,
}

let bytes = [0xA5, 0x00, 0x0A];
let (header, rest) = MyHeaderRef::parse(&bytes)?;

assert_eq!(header.tag(), 0xA_u8);       // just compare with a number
assert_eq!(header.flags(), 0x5_u8);
assert_eq!(header.length(), 10);
```

One import. One struct. One parse call. Compare with plain numbers. Done.

## Your First 5 Minutes

```rust
use bitframe::prelude::*;

// 1. Declare — the struct IS the wire format
#[bitframe]
pub struct CcsdsPrimaryHeader {
    pub version:        u3,
    pub is_telecommand: bool,    // named as a question
    pub has_secondary:  bool,    // named as a question
    pub apid:           u11,
    pub seq_flags:      u2,
    pub seq_count:      u14,
    pub pkt_len:        u16,
}

// 2. Parse — zero allocation, zero copying
let bytes = [0xA0, 0x1F, 0xC0, 0x42, 0x00, 0x0A];
let (header, payload) = CcsdsPrimaryHeaderRef::parse(&bytes)?;

// 3. Read fields — they read like English
assert_eq!(header.version(), 5_u8);
assert!(!header.is_telecommand());      // reads: "header is telecommand? no"
assert!(!header.has_secondary());       // reads: "header has secondary? no"
assert_eq!(header.apid(), 31_u16);
assert_eq!(header.pkt_len(), 10);

// 4. Debug — shows all decoded fields, not raw bytes
println!("{header:?}");
// CcsdsPrimaryHeaderRef { version: 5, is_telecommand: false,
//   has_secondary: false, apid: 31, seq_flags: 3, seq_count: 66, pkt_len: 10 }

// 5. Chain — parse the next header from the remaining bytes
let (secondary, data) = TmSecondaryHeaderRef::parse(payload)?;
```

If any step requires reading a doc, the API is wrong.

---

## How the API Reads as English

### Parsing reads as "parse this from these bytes"

```rust
let (header, payload) = CcsdsPrimaryHeaderRef::parse(bytes)?;
//   "parse a CCSDS primary header from bytes"

let header = CcsdsPrimaryHeaderRef::parse_exact(bytes)?;
//   "parse exactly a CCSDS primary header from bytes"
```

### Accessors read as questions or lookups

```rust
header.is_telecommand()   // "is this header telecommand?"         -> bool
header.has_secondary()    // "does this header have a secondary?"  -> bool
header.version()          // "what is the version?"                -> u3
header.apid()             // "what is the APID?"                   -> u11
header.pkt_len()          // "what is the packet length?"          -> u16
```

### Writers read as "set this to that"

```rust
CcsdsPrimaryHeaderWriter::new(&mut buf)
    .version(0)               // "set version to 0"
    .is_telecommand(false)    // "set is_telecommand to false"
    .apid(42)                 // "set APID to 42"
    .pkt_len(100)             // "set packet length to 100"
    .finalize()?;             // "finalize the header"
```

### Mutation reads as "set the field"

```rust
header.set_seq_count(999);    // "set the sequence count to 999"
header.set_apid(42);          // "set the APID to 42"
```

### Error messages read as sentences

```
"buffer too short for CcsdsPrimaryHeader: need 6 bytes, have 2"
"invalid Priority in field 'priority': raw value 7"
```

---

## Convention Over Configuration

### Field order is wire order

The first field in the struct is the first bits on the wire. Always. No `#[bitframe(offset = 5)]` annotations.

### Big-endian is the default

Network protocols use big-endian. You only annotate the exception:

```rust
#[bitframe(little_endian)]
pub host_value: u16,
```

### Padding is declared, not computed

Prefix with `_` and every bit is accounted for:

```rust
pub _reserved: u5,     // readable, not writable, zeroed on encode
```

### Enum width is inferred

The macro computes the minimum bit width from the discriminant values:

```rust
#[bitframe_enum]
#[repr(u8)]
pub enum SeqFlags {
    Continuation = 0,
    First        = 1,
    Last         = 2,
    Standalone   = 3,
}
// 4 variants, max value 3 -> 2 bits (automatic)
```

Only annotate width when you want more room than the minimum:

```rust
#[bitframe_enum(bits = 3)]   // 3-bit wire width, values only use 0-2
#[repr(u8)]
pub enum Priority { Low = 0, Medium = 1, High = 2 }
```

### Exhaustive enums skip `Result`

If every bit pattern has a variant, the accessor is infallible:

```rust
let flags: SeqFlags = header.seq_flags();     // no Result, no unwrap
```

If gaps exist, the accessor returns `Result`:

```rust
let priority = header.priority()?;            // might be unknown
```

The type tells you whether validation can fail. You never have to guess.

---

## You Write Values, Not Types

### Writer setters accept plain numbers

```rust
// You write this:
CcsdsPrimaryHeaderWriter::new(&mut buf)
    .version(0)
    .apid(42)
    .seq_count(1)
    .pkt_len(100)
    .finalize()?;

// Not this:
//  .version(u3::new(0))
//  .apid(u11::new(42))
//  .seq_count(u14::new(1))
```

Each setter takes the backing integer type (`u8` for u1-u7 fields, `u16` for u9-u15, etc.) and validates the range. If `42` fits in 11 bits, it works. If `9999` doesn't, it panics with a clear message: `"9999 exceeds u11 max (2047)"`.

You write what you mean. The library does the checking.

### RefMut setters work the same way

```rust
header.set_seq_count(999);     // just a number
header.set_apid(42);           // just a number
```

### Struct literals use the bit-sized type

When constructing an owned value, you use the newtype — because you're building a data structure, not writing into a buffer:

```rust
let header = CcsdsPrimaryHeader {
    version: u3::new(0),
    is_telecommand: false,
    has_secondary: true,
    apid: u11::new(42),
    seq_flags: u2::new(3),
    seq_count: u14::new(1),
    pkt_len: 100,
};
```

The distinction is intentional: builders (Writer, RefMut) are ergonomic, struct literals are explicit.

---

## Bit-Sized Types Feel Native

`u11` is not a "bitframe wrapper." It's a small integer that knows its width.

### It does what you'd expect

```rust
let apid = u11::new(42);

apid.value()                  // 42_u16 — the raw integer
u16::from(apid)               // 42_u16 — via From
format!("{apid}")             // "42"
format!("{apid:?}")           // "u11(42)"
```

### Compare directly with integers — no `.value()` needed

```rust
assert_eq!(header.apid(), 31_u16);           // u11 == u16 works
assert!(header.seq_count() > 0_u16);         // u14 > u16 works
assert_eq!(header.version(), u3::new(5));     // u3 == u3 works
```

### Format in any base

```rust
format!("{apid:#x}")          // "0x2a"        (hex)
format!("{apid:#b}")          // "0b101010"    (binary)
format!("{apid:#o}")          // "0o52"        (octal)
```

### Bitwise operations just work

```rust
let masked  = flags & u3::new(0b101);     // AND
let merged  = flags | u3::new(0b010);     // OR
let flipped = flags ^ u3::new(0b111);     // XOR
let inverted = !flags;                     // NOT (masked to 3 bits)
```

Result is always the same type — bitwise ops on N-bit values produce N-bit values.

### Constants and construction

```rust
u11::WIDTH         // 11
u11::MAX           // 2047
u11::ZERO          // u11(0)

u11::new(42)       // const fn — compile-time checked for literals
u11::try_new(42)   // -> Result<u11, OutOfRange>
```

`new()` is `const fn`, so literal values are validated at compile time:

```rust
const APID: u11 = u11::new(42);     // zero-cost, checked at compile time
const BAD: u11 = u11::new(9999);    // compile error
```

---

## Views Are Transparent

### Access the underlying bytes

```rust
let (header, _) = CcsdsPrimaryHeaderRef::parse(bytes)?;
let raw: &[u8] = header.as_bytes();   // the 6 bytes this view covers
```

### Debug output shows field values, not raw bytes

```rust
println!("{header:?}");
// CcsdsPrimaryHeaderRef { version: 5, is_telecommand: false,
//   has_secondary: false, apid: 31, seq_flags: 3, seq_count: 66, pkt_len: 10 }
```

Paste this into a bug report. Another engineer understands it instantly.

### Size is always known

```rust
CcsdsPrimaryHeader::SIZE_BITS    // 48
CcsdsPrimaryHeader::SIZE_BYTES   // 6

let mut buf = [0u8; CcsdsPrimaryHeader::SIZE_BYTES];
```

---

## Owned vs View

```rust
// View — borrows the buffer, zero allocation
let (view, _) = CcsdsPrimaryHeaderRef::parse(bytes)?;
let apid = view.apid();         // reads from the original bytes
// view cannot outlive bytes

// Owned — copies fields, independent of buffer
let owned = CcsdsPrimaryHeader::from_bytes(bytes)?;
drop(bytes);
let apid = owned.apid;          // plain struct field

// Convert
let owned = view.to_owned();
```

| Situation | Use |
|-----------|-----|
| Hot path, buffer stays alive | `FooRef` (zero allocation) |
| Store or send across threads | `Foo` (owned) |
| Logging snapshot | `view.to_owned()` |

---

## Generic Code with `BitLayout`

Every `#[bitframe]` type implements `BitLayout` and `Parseable`:

```rust
use bitframe::BitLayout;

fn describe<T: BitLayout>() {
    println!("{} bits ({} bytes)", T::SIZE_BITS, T::SIZE_BYTES);
}

fn parse_and_log<'a, T: Parseable<'a>>(bytes: &'a [u8]) -> Result<T::View, Error>
where T::View: core::fmt::Debug {
    let (view, _) = T::parse(bytes)?;
    log::debug!("parsed: {view:?}");
    Ok(view)
}
```

---

## Composing with Other Parsers

bitframe handles fixed-size headers. Chain them naturally:

```rust
let (header, payload) = CcsdsPrimaryHeaderRef::parse(bytes)?;

match header.apid().value() {
    0x100 => handle_housekeeping(payload),
    0x200 => handle_science(payload),
    _     => log::warn!("unknown APID: {}", header.apid()),
}
```

Or chain another bitframe struct:

```rust
let (primary, rest) = CcsdsPrimaryHeaderRef::parse(bytes)?;
let (secondary, data) = TmSecondaryHeaderRef::parse(rest)?;
```

bitframe parses headers. Your code handles routing.

---

## Error Messages Are Documentation

### Compile-time — points at the exact field

```
error: bitframe layout error
  --> src/packets.rs:5:5
   |
5  |     pub apid: u11,
   |     ^^^^^^^^^^^^^^
   |
   = note: field `apid` (11 bits) overlaps with field `has_secondary` (1 bit)
   = note: total bits before `apid`: 5, but `apid` would occupy bits 5..16
   = help: check field ordering or adjust bit widths
```

```
error: bitframe layout error
  --> src/packets.rs:8:5
   |
8  |     pub data: Vec<u8>,
   |     ^^^^^^^^^^^^^^^^^
   |
   = note: `Vec<u8>` is not a fixed-size bitframe type
   = help: supported types: bool, u8, u16, u32, u64, u1..u63, #[bitframe_enum] enums
   = help: for variable-length data, parse the header with bitframe
           and handle the payload separately
```

Errors point at the field, not the attribute. They tell you what to do, not just what's wrong.

### Runtime — structured and readable

```rust
let short = [0xA0, 0x1F];
let result = CcsdsPrimaryHeaderRef::parse(&short);
// Err(Error::TooShort { needed_bytes: 6, have_bytes: 2 })
// Display: "buffer too short for CcsdsPrimaryHeader: need 6 bytes, have 2"
```

---

## Endianness

Default: **big-endian** (network order). Override per-field:

```rust
#[bitframe]
pub struct MixedPacket {
    pub network_field: u16,                    // big-endian (default)
    #[bitframe(little_endian)]
    pub host_field:    u16,                    // little-endian override
    pub flags:         u4,                     // sub-byte: doesn't apply
}
```

---

## IDE Experience

- **Hover on `FooRef`** shows field names, bit widths, total size
- **Hover on `.apid()`** shows return type and bit position
- **Autocomplete** on a view lists all field accessors
- **Go to definition** on `FooRef` goes to the `#[bitframe]` struct
- **Compile errors** have correct spans — click the error, land on the field

---

## Prelude

One import. Everything works.

```rust
use bitframe::prelude::*;
// #[bitframe], #[bitframe_enum]
// u1..u63, i2..i63
// Error, BitLayout, Parseable
```

---

## Feature Flags

| Flag | Enables | Requires |
|------|---------|----------|
| `std` (default) | `std::error::Error` on errors | `std` |
| `alloc` | `to_vec()` on views and owned types | `alloc` |
| *(none)* | Everything else — views, writers, errors | `core` only |

---

## Testing with behave

Tests read like specifications:

```rust
use behave::prelude::*;
use bitframe::prelude::*;

behave! {
    "CCSDS primary header" {
        setup {
            let bytes = [0xA0, 0x1F, 0xC0, 0x42, 0x00, 0x0A];
            let (header, payload) = CcsdsPrimaryHeaderRef::parse(&bytes)
                .expect("valid CCSDS header");
        }

        "parses version from bits 0..3" {
            expect!(header.version()).to_equal(5_u8)?;
        }

        "parses APID from bits 5..16" {
            expect!(header.apid()).to_equal(31_u16)?;
        }

        "payload is the remaining bytes" {
            expect!(payload).to_be_empty()?;
        }

        "roundtrips through encode" {
            let owned = header.to_owned();
            let mut buf = [0u8; CcsdsPrimaryHeader::SIZE_BYTES];
            owned.encode_to(&mut buf).expect("encode");
            expect!(buf.as_slice()).to_equal(bytes.as_slice())?;
        }
    }
}
```

No `.value()` in assertions — bit-sized types compare directly with integers.

Tests are the contract. If a test reads wrong, the API is wrong.

---

## Naming Conventions

| You write | You get | Reads as |
|-----------|---------|----------|
| `struct Foo` | `FooRef<'a>` | "Foo view" |
| | `FooRefMut<'a>` | "Foo mutable view" |
| | `FooWriter<'a>` | "Foo writer" |
| | `Foo` | "Foo (owned)" |
| `FooRef::parse(bytes)` | `(FooRef, &[u8])` | "parse, get remainder" |
| `FooRef::parse_exact(bytes)` | `FooRef` | "parse, reject extra" |
| `.field_name()` | | "read field" |
| `.set_field_name(val)` | | "set field" |
| `.field_name_raw()` | | "raw bits for enum" |
| `.as_bytes()` | `&[u8]` | "view as bytes" |
| `.to_owned()` | `Foo` | "copy to owned" |

If you guess the method name, you're right.
