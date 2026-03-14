# Landscape

Where `bitframe` fits in the Rust ecosystem, the domains it serves, and why it exists.

*Competitive data as of March 2026. Star counts and download numbers will drift over time.*

## Identity

`bitframe` is a **bit-level packet layout** tool that generates **zero-copy views**.

It is not "just a bitfield crate" and not "a general binary parsing framework".
The focus is fixed-size packet headers and frames that must be read from raw bytes safely and repeatably.

### One piece of vocabulary: "view"

In `bitframe`, a **view** is:

- a small wrapper around `&[u8]` (pointer + length)
- with methods like `.apid()` / `.seq_count()` that **read bits out of that slice on demand**
- without allocating or copying the whole header into an owned struct

If you know Rust strings: `&str` is a *view* over bytes owned by a `String`. `bitframe` views are the
same idea, but for bit-packed packet headers.

---

## Target Domains

### 1. Aviation/Aerospace — Highest Priority

**Protocols:** ARINC 429 (32-bit words), MIL-STD-1553 (20-bit words), STANAG.

**Why critical:** ARINC 429 packs fields at 8-bit (reversed label), 2-bit SDI, 19-bit data, 1-bit sign,
1-bit status, 1-bit parity. MIL-STD-1553 uses 20-bit words (16 data + sync + parity). These are the
most demanding use cases for bit-level layout.

**Current Rust ecosystem:** Essentially nothing. No Rust crates exist for ARINC 429 or MIL-STD-1553 parsing.
This is a massive gap. The aviation/defense industry is increasingly interested in Rust for safety-critical
systems, but protocol tooling does not exist.

**Why bitframe wins:** ARINC 429 label bits are transmitted LSB-first (reversed within the label octet) —
a constant source of bugs in C. A declarative bit-layout library with proper bit-ordering control eliminates
this class of error.

**Example layout:**
```rust
#[bitframe]
pub struct Arinc429Word {
    pub label:    u8,      // 8 bits (reversed bit order — LSB transmitted first)
    pub sdi:      u2,      // Source/Destination Identifier
    pub data:     u19,     // Data field (BNR or BCD encoded)
    pub ssm:      u2,      // Sign/Status Matrix
    pub parity:   bool,    // Odd parity
}
```

### 2. Automotive (CAN Bus / CAN FD)

**Protocols:** CAN 2.0 (8-byte frames, 11/29-bit IDs), CAN FD (64-byte frames), LIN bus.

**Why critical:** CAN signals are defined at arbitrary bit positions within data frames. A single
message can pack 8+ signals at odd bit widths (e.g., 12-bit engine RPM starting at bit 4). DBC
files define thousands of such signals.

**Current Rust ecosystem:** `can-dbc` parses DBC files. `dbcc` generates Rust code from DBC — but
produces 30,000+ lines for a single vehicle (e.g., Tesla Model 3), crushing IDE performance.
`socketcan-rs` handles Linux SocketCAN. No crate provides declarative signal extraction.

**Why bitframe wins:** Replaces code generation with declarative struct definitions. One struct per
message type, with signals as bit-sized fields. Supports both Motorola (big-endian) and Intel
(little-endian) byte ordering via per-field endianness.

**Example layout:**
```rust
#[bitframe]
pub struct CanExtendedId {
    pub id:   u29,    // 29-bit extended identifier
    pub rtr:  bool,   // Remote Transmission Request
    pub ide:  bool,   // Identifier Extension
    pub dlc:  u4,     // Data Length Code
}
```

### 3. Space/Satellite (CCSDS)

**Protocols:** CCSDS Space Packet (133.0-B-2), ECSS PUS TM/TC (ECSS-E-ST-70-41C), CFDP.

**Why critical:** The CCSDS primary header packs 6 fields into 6 bytes at widths of 3, 1, 1, 11,
2, 14, and 16 bits. Every space mission since the 1990s uses this format.

**Current Rust ecosystem:** Fragmented — at least 5 crates (`spacepackets`, `ccsds_primary_header`,
`ccsds`, `spacepacket`, `ccsds_spacepacket`) for the same primary header, each hand-coding the
bit manipulation differently. `spacepackets` is the most complete.

**Why bitframe wins:** A single `#[bitframe]` struct replaces all hand-coded bit math. Mission-specific
secondary headers can be defined declaratively instead of writing custom shift/mask code for each
telemetry dictionary entry.

### 4. Radio/SDR (ADS-B, DMR, P25)

**Protocols:** ADS-B (56/112-bit messages), DMR (TDMA), P25, TETRA.

**Why critical:** These are literal bitstream protocols. ADS-B messages have fields at 5-bit downlink
format, 3-bit capability, 24-bit ICAO address, and 56-bit message content. Processing requires
extracting fields at arbitrary bit positions.

**Current Rust ecosystem:** `adsb_deku` uses deku for bit-level ADS-B parsing — validating the
declarative approach but inheriting deku's bitvec dependency weight and performance concerns.
No Rust crates for DMR or TETRA.

**Why bitframe wins:** Same expressiveness as deku for ADS-B but with zero-copy views and no bitvec dependency.

### 5. Embedded/IoT (BLE, LoRa, Zigbee, Sensors)

**Protocols:** LoRaWAN frames, BLE advertising, IEEE 802.15.4/Zigbee, NMEA, UBX.

**Why critical:** BLE advertising has 4-bit header fields. LoRaWAN has 3-bit version, 5-bit MHDR type.
These protocols run on Cortex-M0/M4 with 16-256 KB RAM — every byte matters.

**Current Rust ecosystem:** `lorawan-encoding` handles LoRaWAN. `dot15d4-frame` handles 802.15.4.
Each re-implements its own bit extraction. No unified approach.

**Why bitframe wins:** `no_std` + zero-alloc + zero-copy is mandatory in this domain. bitframe's
proc-macro approach generates inline shift/mask code with no runtime library overhead.

### 6. Industrial (EtherCAT, Modbus)

**Protocols:** EtherCAT process data, Modbus coil registers, PROFIBUS.

**Why critical:** EtherCAT Process Data Images map device I/O to arbitrary bit offsets within shared
memory. Accessing a single boolean output might mean reading bit 3 of byte 47 in a 200-byte PDI.

**Current Rust ecosystem:** `ethercrab` is a production EtherCAT stack. `rmodbus` handles Modbus.
Both implement bit extraction internally without a shared primitive.

### 7. Networking (Protocol Headers)

**Protocols:** IPv4/IPv6, TCP options, MPLS (20-bit labels), VXLAN (24-bit VNI), GRE.

**Why critical:** IPv4 has 4-bit fields, 3-bit flags, 13-bit fragment offset. Standard protocols are
well-served by `etherparse` and `pnet`, but custom encapsulation headers need extensibility.

**Current Rust ecosystem:** `etherparse` (zero-alloc, used in Android), `pnet`, `smoltcp` all hard-code
supported protocols. Adding custom headers requires modifying the library or manual parsing.

### 8. Gaming (Bit-Packed State Sync)

**Protocols:** Custom game state, input synchronization, delta compression.

**Why critical:** Competitive multiplayer packs player inputs into minimal bits to fit within MTU.
A boolean takes 1 bit, rotation 10 bits, health 7 bits.

### 9. Storage (Filesystem Metadata)

**Protocols:** Partition tables, inode flags, superblock fields.

Mostly byte-aligned. Bit-level access needed primarily for flag fields and permission bits.

### 10. Security/Crypto (ASN.1, TLS)

**Protocols:** TLS record headers, X.509 key usage bit strings.

Well-served by existing nom-based parsers (`tls-parser`, `x509-parser`).

---

## Deep Competitive Analysis

### Competitor Matrix

| Library | Stars | Downloads/mo | Last Release | Category | Zero-copy? | Bit-level? |
|---------|------:|-------------:|-------------|----------|:----------:|:----------:|
| **nom** | ~9.4k | ~50M | v8.0.0 (2025) | Parser combinator | Yes | Yes (verbose) |
| **deku** | 1.4k | 418k | v0.20.3 (Jan 2026) | Derive binary ser/de | No | Yes |
| **binrw** | 811 | 677k | v0.15.1 (Mar 2026) | Derive binary reader/writer | No | No |
| **modular-bitfield** | 219 | 546k | v0.13.1 (Dec 2025) | Register bitfield | No | Yes |
| **bilge** | 195 | 59k | v0.3.0 (Sep 2025) | Register bitfield | No | Yes |
| **packed_struct** | 176 | 674k | v0.10.1 (Nov 2022) | Bit-level pack/unpack | No | Yes |
| **bitfield-struct** | 114 | 1.2M | v0.12.1 (Oct 2025) | Register bitfield | No | Yes |
| **zerocopy** | 2.2k | ~114M | v0.8.x (Mar 2026) | Byte-level zero-copy | Yes | No |
| **binary-layout** | ~90 | ~100k | 2024 | Byte-level zero-copy layout | Yes | No |
| **bitbybit** | — | 107k | v2.0.0 (Feb 2026) | Attribute bitfield | No | Yes |

---

### 1. deku — Primary Competitor

**What it does:** Derive macro for symmetric bit-level serialization/deserialization.

**Stats:** 5.2M total downloads, 50 reverse dependencies, Rust 1.81+ MSRV.

**What deku did RIGHT:**
- Symmetric read/write from a single struct definition
- Bit-level granularity is first-class, not bolted on
- Enum variant dispatch with `#[deku(id = 0x01)]`
- Recent no_alloc work (v0.20) for embedded targets

**What deku did WRONG:**
- **Copies into owned structs** — no borrowed views, no on-demand field access
- **Heavy dependency tree** — bitvec (effectively abandoned), funty, radium, tap, wyz (~2 MiB)
- **Slow generated code** — bitvec adds overhead even for byte-aligned fields
- **Endianness context passing** — child structs must explicitly receive endianness via `ctx` (#79, #217, #501)
- **String-based attribute values** — `#[deku(endian = "big")]` prevents IDE support
- **Opaque compile errors** — point to `#[derive]` line, not the actual attribute
- **No mutable in-place editing** — modify one field = deserialize + modify + re-serialize
- **73% test coverage** (vs binrw's 96%)
- **Breaking API changes** — v0.17 renamed core traits and changed all function signatures

**Key open issues:** #631 (embedded-io), #501 (endian confusion), #217 (project-level settings), #302 (checksums).

### 2. bilge

**What it does:** Register-style bitfield library using `#[bitsize(N)]` with `arbitrary-int` types.

**Stats:** 678k total downloads, actively maintained (Sep 2025).

**What bilge did RIGHT:**
- "Parse, don't validate" — `FromBits` vs `TryFromBits` with `FILLED` const flag
- Modular derive system — third-party macros can extend
- Benchmarks match handwritten code
- `arbitrary-int` types feel native (`u4`, `u7`, `u14`)

**What bilge did WRONG:**
- **Cannot pack/unpack to `&[u8]`** — most requested feature (#33), still open
- **No endianness support** — users must handle byte ordering manually (#7)
- **Max size u128** — structures larger than 128 bits must be split
- **no_std broken by itertools** (#94)
- **Pre-1.0 stability**

### 3. modular-bitfield

**What it does:** Builder-pattern bitfield structs with `B1`-`B128` field types.

**Stats:** 7.7M total downloads. Was abandoned, now maintained by new owner (Dec 2025).

**What it did RIGHT:**
- Mature and battle-tested (789 downstream crates)
- Compile-time safety, performance parity with handwritten code
- Clean builder API (`.with_header(1).with_body(2)`)

**What it did WRONG:**
- **No const fn** (#26, open since 2020)
- **No MSB byte ordering** (#46)
- **Power-of-two enum variant requirement**
- **Monolithic macro** — cannot extend with custom derives
- **Historical maintenance gap** caused community exodus

### 4. packed_struct

**What it does:** Derive macro for bit-level packing/unpacking with MSB0/LSB0 options.

**Stats:** 8.7M total downloads. **Effectively dormant** (last release Nov 2022, no activity 2+ years).

**What it did RIGHT:**
- Explicit bit-range syntax (`bits="0..=2"`) mirrors protocol specs
- MSB0/LSB0 configurable bit numbering
- Runtime packing visualization

**What it did WRONG:**
- **Abandoned** — 32 open issues, 6 pending PRs
- **Verbose types** — `Integer<u8, packed_bits::Bits::<3>>` vs bilge's `u3`
- **Not C-compatible** — generated structs NOT byte-compatible with C equivalents
- **Copies into owned structs** — `pack()` returns `[u8; N]` by value
- **Compile time explosion** for large arrays (#110)
- **No floating point** (#108)
- **bitvec dependency**

### 5. bitfield-struct

**What it does:** Attribute macro generating getter/builder/setter methods.

**Stats:** 9.6M total downloads, actively maintained. Highest download count among bitfield crates.

**What it did RIGHT:**
- **Zero runtime dependencies** — pure proc-macro
- Three accessor patterns per field (getter, `with_*` builder, `set_*` mutation)
- Compile-time field offset/size constants
- defmt integration
- Read-only / write-only fields for hardware registers

**What it did WRONG:**
- Integer-backed, not byte-backed — cannot overlay on `&[u8]`
- No explicit bit-range syntax
- Limited to primitive backing types (max u128)

### 6. binrw

**Stats:** 6.8M total downloads, 96% test coverage.

**Relevance:** Byte-level only — no bit-level support. But sets the quality bar for error diagnostics
and test coverage that bitframe should match.

### 7. zerocopy

**Stats:** 466M total downloads. Maintained by Google. Formally verified with Kani.

**Relevance:** Proves the zero-copy view pattern works at scale. But byte-aligned only — cannot
handle sub-byte fields. The `rand` dependency debate showed that large unsafe codebases are
controversial as transitive dependencies.

### 8. bitvec

**Stats:** 172M total downloads. **Effectively abandoned** (last release July 2022, 100 open issues).

**Relevance:** Infrastructure crate used by deku. Its maintenance uncertainty and performance
regressions are a systemic problem. nom removed it as a default dependency. bitframe must
**never** depend on bitvec.

---

## The Gap bitframe Fills

**No existing crate combines all three capabilities:**

| Capability | deku | binary-layout | bilge/modular-bitfield | zerocopy | nom | **bitframe** |
|------------|:----:|:-------------:|:----------------------:|:--------:|:---:|:------------:|
| Bit-level fields | Yes | No | Yes | No | Yes | **Yes** |
| Zero-copy views | No | Yes | No | Yes | Yes | **Yes** |
| Byte-buffer parsing | Yes | Yes | No | Yes | Yes | **Yes** |
| Declarative struct | Yes | Macro | Yes | Derive | No | **Yes** |
| In-place mutation | No | Partial | No | No | No | **Yes** |
| Zero runtime deps | No | Yes | No | No | No | **Yes** |

bitframe is the **only** library that combines bit-level struct fields + `&[u8]` parsing + borrowed zero-copy views.

---

## Most Requested Features Across All Competitors

Ranked by frequency of issues, discussions, and forum posts:

1. **Byte serialization/deserialization** — bilge #33, modular-bitfield users wanting pack/unpack to `&[u8]`
2. **Endianness control** — bilge #7, deku #79/#217, modular-bitfield #46
3. **Enum support with fallback** — non-exhaustive enums, raw value accessors, catch-all variants
4. **no_std / no_alloc** — deku #375, bilge #94, embedded-io requests
5. **Const fn accessors** — modular-bitfield #26, bilge future goal
6. **Serde integration** — modular-bitfield #74
7. **Nested/composed structures** — deku context passing pain, bilge #91
8. **Mutable views** — no existing crate offers `FooRefMut`

bitframe addresses **all eight** of these in its roadmap.

---

## Positioning

**"bitframe is to deku what `&str` is to `String`."**

| Dimension | deku | bitframe |
|-----------|------|----------|
| Parse output | Owned struct (copies all fields) | Borrowed view (`FooRef<'a>`) |
| Field access | All fields decoded upfront | Fields read on-demand from bytes |
| Allocation | Requires alloc by default | Zero allocation by default |
| Modification | Deserialize, mutate, reserialize | `FooRefMut` edits in-place |
| Scope | General binary formats | Fixed-size bit-packed headers |
| Dependencies | bitvec + deku_derive + no_std_io (~2 MiB) | Zero runtime deps |
| Performance | Runtime bit extraction via bitvec | Compile-time shift/mask constants |
| Wire clarity | Field widths in attributes (`#[deku(bits = 3)]`) | Field widths in types (`u3`) |

**They are complementary.** A CCSDS ground system might use bitframe for fixed primary headers
and deku for variable-length payloads.

---

## Practical "Which One?" Guidance

- **General parser** (variable-length, conditionals, complex pipelines): use `deku` / `binrw` / `nom`
- **In-memory bitfield** (register setters/getters on an integer): use `bilge` / `modular-bitfield` / `bitfield-struct`
- **Byte-aligned zero-copy**: use `zerocopy` or `binary-layout`
- **Bit-level fixed-size header as a borrowed view over `&[u8]`**: use **`bitframe`**
