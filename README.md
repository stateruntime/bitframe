# bitframe

[![crates.io](https://img.shields.io/crates/v/bitframe.svg)](https://crates.io/crates/bitframe)
[![docs.rs](https://docs.rs/bitframe/badge.svg)](https://docs.rs/bitframe)
[![CI](https://github.com/stateruntime/bitframe/actions/workflows/ci.yml/badge.svg)](https://github.com/stateruntime/bitframe/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/bitframe.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.75-blue.svg)](https://blog.rust-lang.org/)

**Describe your packet's bit fields in a struct. Get zero-copy parsing for free.**

```rust
use bitframe::prelude::*;

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

let (header, payload) = CcsdsPrimaryHeaderRef::parse(bytes)?;
assert_eq!(header.apid(), 31_u16);
```

That's it. You describe the layout. `bitframe` does the bit math.

Status: **v0.1.0** — parsing is implemented. Writers and mutable views are planned for future releases.

---

## The Problem

Bit-packed protocols are everywhere: satellite telemetry, CAN bus, ADS-B, sensor data. Parsing them by hand means shift/mask code that's hard to review and produces **silent corruption** — not crashes, just quietly wrong values.

```rust
// Can you spot the bug? Neither can your reviewer.
let apid = ((bytes[0] as u16 & 0x07) << 8) | bytes[1] as u16;
// Is the mask 0x07 or 0x0F? Is this bits 5-15 or 4-15?
// You won't know until the wrong satellite gets a command.
```

## The Solution

The struct **is** the spec. If you can read the struct, you understand the protocol.

```rust
#[bitframe]
pub struct CcsdsPrimaryHeader {
    pub version:        u3,     // 3 bits   — bits 0..3
    pub is_telecommand: bool,   // 1 bit    — bit 3
    pub has_secondary:  bool,   // 1 bit    — bit 4
    pub apid:           u11,    // 11 bits  — bits 5..16
    pub seq_flags:      u2,     // 2 bits   — bits 16..18
    pub seq_count:      u14,    // 14 bits  — bits 18..32
    pub pkt_len:        u16,    // 16 bits  — bits 32..48
}                               // Total: 48 bits = 6 bytes, verified at compile time
```

Parsing gives you a **zero-copy view** — a thin wrapper around `&[u8]` that reads fields on demand:

```rust
// Parse: zero allocation, zero copying
let (header, payload) = CcsdsPrimaryHeaderRef::parse(bytes)?;

// Read fields — they read like English
if header.is_telecommand() { /* ... */ }    // "is this header telecommand?"
if header.has_secondary()  { /* ... */ }    // "does this header have a secondary?"

// Compare directly with numbers — no .value() needed
assert_eq!(header.apid(), 31_u16);
assert_eq!(header.version(), 5_u8);

// If the buffer is too short, you get a clear error — not garbage
// Err(Error::TooShort { needed_bytes: 6, have_bytes: 2 })
```

---

## Start Fast

```bash
cargo add bitframe
```

Create `src/main.rs`:

```rust
use bitframe::prelude::*;

#[bitframe]
pub struct MyHeader {
    pub tag:    u4,
    pub flags:  u4,
    pub length: u16,
}

fn main() {
    let bytes = [0xA5, 0x00, 0x0A];
    match MyHeaderRef::parse(&bytes) {
        Ok((header, _rest)) => {
            println!("tag={}, flags={}, length={}", header.tag(), header.flags(), header.length());
        }
        Err(e) => eprintln!("parse error: {e}"),
    }
}
```

```bash
cargo run
# tag=10, flags=5, length=10
```

---

## How It Works

```
Raw bytes:  [0xA0, 0x1F, 0xC0, 0x42, 0x00, 0x0A]

Byte 0          Byte 1        Byte 2        Byte 3        Byte 4-5
+-----------+---+-----------+----+----------+-------------------+
| ver 3b    |T|S|  APID 11b |SF 2|seq_ct 14b|    pkt_len 16b   |
+-----------+---+-----------+----+----------+-------------------+

CcsdsPrimaryHeaderRef::parse(&bytes) gives you:
  .version()          -> 5         reads bits 0..3
  .is_telecommand()   -> false     reads bit 3
  .has_secondary()    -> false     reads bit 4
  .apid()             -> 31       reads bits 5..16
  .seq_flags()        -> 3        reads bits 16..18
  .seq_count()        -> 66       reads bits 18..32
  .pkt_len()          -> 10       reads bits 32..48
```

No heap allocation. No struct copying. The view borrows your `&[u8]` — like `&str` borrows a `String`.

---

## Features

| Flag | Enables | Requires |
|------|---------|----------|
| `std` (default) | `std::error::Error` on errors | `std` |
| *(none)* | Everything else — views, parsing, errors | `core` only |

---

## When to Use bitframe

| If you need... | Use |
|---|---|
| Parse variable-length formats (JSON, protobuf, custom TLVs) | `serde`, `binrw`, `nom` |
| In-memory bitfields (register getters/setters on an integer) | `bilge`, `modular-bitfield`, `bitfield-struct` |
| Byte-aligned zero-copy views | `zerocopy`, `binary-layout` |
| **Bit-level fixed-size headers parsed from `&[u8]` as a borrowed view** | **`bitframe`** |

---

## Real-World Use Cases

**Space telemetry (CCSDS)** — Every satellite uses 6-byte headers with fields at 3, 1, 1, 11, 2, 14, and 16 bits.

**Automotive (CAN bus)** — CAN signals live at arbitrary bit positions within 8-byte frames. Manual parsing is a constant source of bugs.

**Aviation (ADS-B/ARINC 429)** — Aircraft transponder messages are 56 or 112 bits with fields at 5-bit, 3-bit, and 24-bit boundaries. ARINC 429 words pack reversed-bit labels, SDI, data, and parity into 32 bits.

**Embedded sensors** — ADC readings packed as 12-bit values, status codes as 4-bit nibbles. Self-documenting with bitframe.

**Industrial (EtherCAT)** — Process Data Images map device I/O to arbitrary bit offsets within shared memory buffers.

---

## What You Can and Cannot Do

**Can:**
- Declare fixed-size bit-packed layouts as plain Rust structs
- Parse zero-copy views from `&[u8]` with on-demand field access
- Use on `no_std` / `no_alloc` targets (embedded, WASM)
- Compare bit-sized types directly with integers (`u11 == u16`)

**Roadmap:**
- Encode fields into `&mut [u8]` with range validation (v0.2)
- Mutate individual fields in-place via `FooRefMut` (v0.3)
- Nest one `#[bitframe]` struct inside another (v0.3)

**Cannot:**
- Parse variable-length or self-describing formats (use `deku`, `binrw`, or `nom`)
- Replace in-memory register bitfields (use `bilge` or `bitfield-struct`)
- Handle runtime-defined layouts or reflection (fixed at compile time)
- Parse from streams — bitframe operates on `&[u8]` slices

---

## Why Rely On It

- `#![forbid(unsafe_code)]` on all crates — no unsafe anywhere
- Zero runtime dependencies — only proc-macro compile-time deps
- Clippy pedantic + nursery enabled
- `cargo deny` enforced — no unmaintained deps, no license issues
- Tests use [behave](https://crates.io/crates/behave) BDD framework — specs read like protocol documentation
- Reference implementation test vectors from [spacepackets](https://egit.irs.uni-stuttgart.de/rust/spacepackets) (CCSDS), [The 1090MHz Riddle](https://mode-s.org/decode/) (ADS-B), and [SocketCAN](https://www.tcpdump.org/linktypes/LINKTYPE_CAN_SOCKETCAN.html) (CAN/J1939)
- MSRV 1.75 tested in CI
- Dual feature-set CI: `--all-features` and `--no-default-features`
- Roadmap includes Kani formal verification and property-based testing

---

## Documentation

- [Vision](docs/VISION.md) — Why this exists
- [Design](docs/DESIGN.md) — Planned API shape
- [Developer Experience](docs/DEVELOPER_EXPERIENCE.md) — DX standards
- [Roadmap](docs/ROADMAP.md) — Release plan v0.1 -> v1.0
- [Landscape](docs/landscape.md) — Ecosystem positioning
- [Audit](docs/AUDIT.md) — Quality gates and acceptance criteria
- [Release](docs/RELEASE.md) — How to publish a new version
- [Contributing](CONTRIBUTING.md) — How to contribute

## Security

See [SECURITY.md](SECURITY.md) for vulnerability reporting.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
