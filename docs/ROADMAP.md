# Roadmap

Release plan from v0.1.0 to v1.0, informed by competitive analysis, ecosystem gaps, and a full audit of the current design documents.

## Guiding Principles

1. **Fixed-size first.** Nail the fixed-size header use case before adding flexibility.
2. **Views over copies.** Parsing yields borrowed views; owned values are optional convenience.
3. **No stubs.** If a feature flag exists, it must provide a minimal working implementation.
4. **DX is a feature.** Error messages and docs are part of the product.
5. **Zero runtime dependencies.** The core crate uses only proc-macro-generated code — no bitvec, no runtime libraries.
6. **Resolve before building.** Design inconsistencies are fixed in the milestone where the feature ships, not deferred.

---

## Pre-Implementation: Design Fixes (before v0.1.0 coding begins)

These inconsistencies were found during audit and **must** be resolved before writing implementation code.

### Must Resolve

| # | Issue | Resolution |
|---|-------|------------|
| 1 | **`parse()` return type inconsistency** — DESIGN.md says `(FooRef, &[u8])`, README/DX doc show `FooRef` only | Standardize: `parse()` returns `Result<(FooRef, &[u8]), Error>`. Update README and DX doc. |
| 2 | **Error field mismatch** — DESIGN.md uses `needed_bits/have_bits`, DX doc uses `layout/needed_bytes/have_bytes` | Standardize: `TooShort { needed_bytes: usize, have_bytes: usize }`. The `layout` name goes in `Display`, not the enum fields. |
| 3 | **`#[repr(u2)]` is invalid Rust** — DESIGN.md line 111 uses `#[repr(u2)]` which won't compile | Change to `#[repr(u8)]` with a `#[bitframe_enum(bits = 2)]` attribute, or infer width from discriminant range. |
| 4 | **Proc-macro crate structure missing** — no workspace, no `bitframe-derive` crate | Design the workspace layout: `bitframe` (facade), `bitframe-derive` (proc-macro), `bitframe-core` (types + errors). |
| 5 | **Endianness attribute contradicts own recommendation** — DESIGN.md uses `#[bitframe(endian = "little")]` (string), landscape.md criticizes deku for the same pattern | Use Rust-native syntax: `#[bitframe(little_endian)]` or `#[bitframe(endian = little)]` (ident, not string). |
| 6 | **`finalize()` vs `finish()` naming** — DX doc says `finalize()`, landscape.md says `finish()` | Standardize on `finalize()` (more explicit about completing the builder). |
| 7 | **`TryFrom<&[u8]>` behavior undefined** — does it accept extra bytes like `parse()` or reject them like `parse_exact()`? | `TryFrom` behaves like `parse()` (accepts extra bytes, discards remainder). |

### Should Resolve

| # | Issue | Resolution |
|---|-------|------------|
| 8 | **`Eq` / `Hash` missing from `FooRef`** — no justification for omitting them | Add `Eq` to `FooRef`. Add `Hash` behind `std` feature. |
| 9 | **Writer field-skip semantics undefined** — what if a builder field is not set? | Unset fields default to zero. Add `finalize_checked()` that returns `Err` if any non-padding field was not explicitly set. |
| 10 | **Non-byte-aligned layout edge case** — 7-bit struct: what is `SIZE_BYTES`? | `SIZE_BYTES = ceil(SIZE_BITS / 8)`. `parse()` requires `SIZE_BYTES` bytes. Trailing bits are ignored on read, zeroed on write. |
| 11 | **`to_owned()` vs `ToOwned` trait** — potential conflict | Implement `ToOwned` trait with `Owned = Foo`. This is the idiomatic approach. |
| 12 | **AUDIT.md and landscape.md overlap** — competitive analysis duplicated | Merge into a single `landscape.md`. AUDIT.md focuses on internal quality criteria. |

---

## v0.1.0 — Layout + View (Foundation)

**Theme:** Make fixed-size bit layouts explicit and parseable as zero-copy views.

**Target use cases:** CCSDS primary header, CAN standard ID, ADS-B downlink format, simple sensor frames.

### Crate Structure [MUST] — DONE

- [x] Workspace with two crates:
  - `bitframe` — facade crate, re-exports everything via `prelude` (includes types, errors, traits)
  - `bitframe-derive` — proc-macro crate (`#[bitframe]`, `#[bitframe_enum]`)
- [x] `no_std` support: `bitframe` is `#![no_std]` by default, `std` feature gates `std::error::Error`
- [x] Zero runtime dependencies (proc-macro deps are compile-time only)
- [x] `#![forbid(unsafe_code)]` on both crates

### Bit-Sized Types [MUST] — DONE

- [x] Unsigned newtypes: `u1`, `u2`, `u3` ... `u63` (skip `u8`, `u16`, `u32`, `u64` — use stdlib)
- [x] Each type wraps the smallest containing stdlib integer (`u3` wraps `u8`, `u11` wraps `u16`, etc.)
- [x] API surface per type: `new`, `try_new`, `value`, `WIDTH`, `MAX`, `ZERO`, `from_raw_unchecked`
- [x] Trait impls: `Debug`, `Display`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `LowerHex`, `UpperHex`, `Binary`, `Octal`, `BitAnd`, `BitOr`, `BitXor`, `Not`
- [x] `From<u3> for u8` (infallible widening), `TryFrom<u8> for u3` (fallible narrowing)
- [x] Decision: rolled our own via `define_uint!` macro (no `arbitrary-int` dependency)

### Proc-Macro: `#[bitframe]` [MUST] — DONE

- [x] Accepts `struct` with named fields only (no tuple structs, no unit structs)
- [x] Supported field types: `bool`, `u8`/`u16`/`u32`/`u64`, `u1`..`u63`
- [x] Generates `FooRef<'a>` view type:
  - Stores `&'a [u8]`
  - One accessor method per field (reads bits on demand via compile-time shift/mask)
  - `parse(bytes: &[u8]) -> Result<(FooRef<'_>, &[u8]), Error>` — returns view + remainder
  - `parse_exact(bytes: &[u8]) -> Result<FooRef<'_>, Error>` — rejects extra bytes
  - `TryFrom<&'a [u8]> for FooRef<'a>` — equivalent to `parse()`, discards remainder
  - `AsRef<[u8]>` — access underlying byte slice
- [x] Generates compile-time constants: `SIZE_BITS: usize`, `SIZE_BYTES: usize`
- [x] Trait impls on `FooRef`: `Debug`, `Copy`, `Clone`, `PartialEq`, `Eq`
- [x] Byte-aligned field optimization: direct `from_be_bytes` reads for aligned u8/u16/u32/u64
- [x] Respects visibility: `pub struct Foo` generates `pub FooRef`

### Proc-Macro: `#[bitframe_enum]` [MUST] — DONE

- [x] Accepts enum with explicit discriminants and `#[repr(u8)]` / `#[repr(u16)]`
- [x] Bit width specified via `#[bitframe_enum(bits = 2)]` or inferred from discriminant range
- [x] Exhaustive enums (all 2^N values covered): infallible `from_raw`
- [x] Non-exhaustive enums: fallible `from_raw` returning `Result`

### Padding Fields [MUST] — DONE

- [x] `_`-prefixed fields are readable (accessor generated)
- [x] Every bit in the layout must be accounted for by a named field (no implicit gaps)

### Errors [MUST] — DONE

- [x] `Error::TooShort { needed_bytes, have_bytes }` — buffer too small
- [x] `Error::InvalidEnum { field: &'static str, raw: u64 }` — enum value not recognized
- [x] `Display` impl always available (no_std), `std::error::Error` behind `std` feature

### Macro Diagnostics [MUST] — PARTIAL

- [x] **Unsupported type** — error names the field and lists valid types
- [ ] **Field overlap detection** — error points to the overlapping field with bit ranges
- [ ] **Size mismatch** — "layout is N bits but field declarations sum to M bits"

### Owned Type [SHOULD]

- [ ] `Foo` struct with fields by value
- [ ] `Foo::from_bytes(&[u8]) -> Result<Self, Error>`
- [ ] `FooRef::to_owned(&self) -> Foo` (implements `ToOwned` trait)
- [ ] Trait impls: `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`

### Golden Tests [MUST] — DONE

- [x] CCSDS primary header: known bytes -> known field values (from CCSDS 133.0-B-2)
- [x] CAN standard ID (11-bit): known bytes -> known field values
- [x] ADS-B short message (56-bit DF=11): known bytes -> known field values
- [x] All tests written using `behave` BDD framework

### Bit Numbering Specification [MUST] — DONE

- [x] MSb0 bit numbering, big-endian byte order for multi-byte fields
- [x] Implementation with worked examples in golden tests
- [ ] Formal specification document (deferred)

---

## v0.2.0 — Encoding

**Theme:** Make roundtrip encode/decode a first-class guarantee.

**Target use cases:** Packet construction for telemetry injection, test vector generation, protocol simulators.

### Writer API [MUST]

- [ ] `FooWriter<'a>` builder that writes into caller-provided `&mut [u8]`
- [ ] One setter method per non-padding field
- [ ] `finalize(self) -> Result<(), Error>` — writes remaining zeros for padding fields
- [ ] `finalize_checked(self) -> Result<(), Error>` — returns error if any non-padding field was not set
- [ ] Padding fields are automatically zeroed (no setter generated)

### `encode_to` on Owned Type [MUST]

- [ ] `fn encode_to(&self, out: &mut [u8]) -> Result<(), Error>`
- [ ] Roundtrip guarantee: `Foo::from_bytes(bytes)?.encode_to(&mut buf) => buf == bytes[..SIZE_BYTES]`

### Per-Field Endianness [MUST]

- [ ] Default: big-endian (network order) for multi-byte fields
- [ ] Override: `#[bitframe(little_endian)]` on individual fields
- [ ] Only applies to multi-byte fields (sub-byte fields are inherently unambiguous)
- [ ] Endianness is **not** propagated to nested structs — each struct declares its own

### Signed Bit-Sized Types [MUST]

- [ ] `i2`, `i3` ... `i63` — two's complement, sign-extended to containing `i8`/`i16`/`i32`/`i64`
- [ ] Same API surface as unsigned types: `new`, `try_new`, `value`, `WIDTH`, `MIN`, `MAX`

### Fixed-Count Array Fields [MUST]

- [ ] `[u12; 4]` — 4 x 12-bit values = 48 bits
- [ ] Accessor returns array by value: `fn channels(&self) -> [u12; 4]`
- [ ] Writer accepts array: `.channels([u12::new(0); 4])`

### Feature Flags [SHOULD]

- [ ] `alloc` feature: enables `to_vec()` on views and owned types
- [ ] `alloc` feature: enables `Vec`-backed convenience constructors

---

## v0.3.0 — Composition

**Theme:** Real packets are composed of headers inside headers.

**Target use cases:** CCSDS TM packet (primary + secondary header), CAN frame (arbitration + control + data), nested protocol stacks.

### Nested Layouts [MUST]

- [ ] A field can be another `#[bitframe]` struct
- [ ] Nested struct contributes its `SIZE_BITS` to parent layout
- [ ] Accessor returns a sub-view at the correct bit offset: `fn primary(&self) -> CcsdsPrimaryHeaderRef<'_>`
- [ ] Endianness is per-struct, not inherited from parent

### Mutable Views [MUST]

- [ ] `FooRefMut<'a>` wraps `&'a mut [u8]`
- [ ] One setter method per non-padding field: `fn set_seq_count(&mut self, val: u14)`
- [ ] Modifies the underlying buffer in-place — no copy, no rebuild
- [ ] `parse_mut(bytes: &mut [u8]) -> Result<(FooRefMut<'_>, &mut [u8]), Error>`

### Offset Support [MUST]

- [ ] `FooRef::parse_at(bytes, bit_offset)` for parsing views at arbitrary bit positions within a larger buffer

### Better Generated Docs [SHOULD]

- [ ] Rustdoc on generated types showing field names, bit ranges, and total size
- [ ] `#[doc]` attributes on accessor methods with field width and bit position

### Serde Support [SHOULD]

- [ ] Behind `serde` feature flag
- [ ] `Serialize` / `Deserialize` on owned types only (not views)
- [ ] Serializes as struct with named fields, not as raw bytes

---

## v0.4.0 — Ecosystem Integration

**Theme:** Play well with the wider Rust ecosystem, especially embedded and telemetry.

**Target use cases:** Production embedded firmware, ground station telemetry pipelines, automotive ECUs.

### defmt Support [SHOULD]

- [ ] Behind `defmt` feature flag
- [ ] `defmt::Format` derive on view and owned types
- [ ] Essential for embedded debugging (ARM Cortex-M targets)

### Protocol Examples Crate [SHOULD]

- [ ] `bitframe-protocols` crate with ready-to-use definitions for:
  - CCSDS primary header (133.0-B-2)
  - CCSDS TM/TC secondary headers (PUS ECSS-E-ST-70-41C)
  - CAN 2.0 standard frame header
  - CAN 2.0 extended frame header
  - ADS-B downlink formats (DF 0, 4, 5, 11, 17, 20, 21)
  - ARINC 429 word (with reversed-bit-order label)
- [ ] Each definition includes golden test vectors from the relevant standard
- [ ] Serves as both reference implementations and adoption drivers

### const fn Accessors [SHOULD]

- [ ] Where Rust allows it, make accessors `const fn`
- [ ] Enables static initialization and compile-time packet validation
- [ ] Track Rust const fn stabilization progress

---

## v0.5.0 — Testing & Quality

**Theme:** Make it hard to ship a wrong parser.

### Fuzzing [MUST]

- [ ] `cargo-fuzz` targets for:
  - Parse arbitrary bytes -> no panics
  - Parse -> encode roundtrip -> bytes match
  - Random field values -> encode -> parse -> values match
- [ ] Fuzzing harness template for users to fuzz their own layouts

### Compile-Fail Tests [MUST]

- [ ] `trybuild` test suite for invalid layouts:
  - Overlapping fields
  - Unsupported field types
  - Non-byte-aligned layouts without explicit trailing padding
  - Missing `#[repr]` on enums
  - Duplicate field names

### Benchmarks [MUST]

- [ ] `criterion` benchmarks comparing:
  - bitframe view parse vs deku parse vs manual shift/mask
  - bitframe encode vs deku encode vs manual shift/mask
  - bitframe field access (on-demand) vs pre-decoded struct field access
- [ ] Published results in repository

### Snapshot/Golden Test Helpers [SHOULD]

- [ ] `bitframe::testing` module with helpers for:
  - Asserting field values against known byte sequences
  - Generating test vectors from owned types
  - Comparing wire format across versions

---

## v0.6.0 — Verification & Safety

**Theme:** Prove it correct. Make safety-critical teams trust it.

**Why now:** After fuzzing (v0.5), the next step is formal proof. Aviation (DO-178C), space (ECSS), and automotive (ISO 26262) all require evidence of correctness. No competitor offers this. Kani is mature enough (v0.61+, used by zerocopy) to make this practical.

**Target use cases:** Safety-critical avionics, satellite flight software, automotive ASIL-rated ECUs, any project that must answer "prove your parser is correct."

### Kani Verification Harnesses [MUST]

- [ ] Proof harness for every bit-sized type: `u1`..`u63` roundtrip (`new(v).value() == v` for all valid `v`)
- [ ] Proof harness for parse/encode roundtrip: for any `Foo`, `Foo::from_bytes(bytes)?.encode_to(&mut buf)` produces `buf == bytes[..SIZE_BYTES]`
- [ ] Proof harness for field extraction: for any byte buffer, each accessor reads exactly the declared bit range (no overlap, no out-of-bounds)
- [ ] Proof harness for writer: for any valid field values, `FooWriter` produces bytes that parse back to the same values
- [ ] CI integration: Kani proofs run on every PR (cached for unchanged layouts)

### Property-Based Testing Integration [MUST]

- [ ] `bitframe::testing::Arbitrary` impl for owned types — generates random valid field values via `proptest`
- [ ] `bitframe::testing::roundtrip_property(bytes)` — one-line property test: parse → encode → compare
- [ ] `bitframe::testing::field_bounds_property()` — verify all field values stay within declared ranges
- [ ] Works with both `proptest` and `quickcheck` (trait-based, not framework-locked)
- [ ] Behind `testing` feature flag — zero cost when not testing

### Compile-Time Layout Invariants [MUST]

- [ ] `const_assert!` that `SIZE_BITS` matches sum of all field widths (already in v0.1, but now formalized)
- [ ] `const_assert!` that no field's bit range overlaps another
- [ ] `const_assert!` that enum discriminant values fit within declared bit width
- [ ] `const_assert!` that nested struct sizes are consistent with parent layout

### Bit-Exact Reproducibility [MUST]

- [ ] Guarantee: identical Rust source + identical input bytes = identical accessor output on all platforms, all optimization levels
- [ ] CI matrix: test on `x86_64`, `aarch64`, `thumbv7em` (cross-compiled), `wasm32`
- [ ] Document that bitframe output is deterministic (no platform-dependent padding or alignment)

### Audit-Friendly Code Generation [SHOULD]

- [ ] `cargo expand` output is human-readable and reviewable
- [ ] Generated code annotated with comments: `// field 'apid': bits 5..16, u11, big-endian`
- [ ] Optional `#[bitframe(emit_layout_comment)]` that generates a layout diagram as a doc comment on the generated types
- [ ] Traceability: every generated line maps to a field in the source struct

---

## v0.7.0 — Tooling & Visualization

**Theme:** See what you're parsing. Generate what you need.

**Why now:** The core library is complete (parse, encode, compose, test, prove). Now make it a **protocol development tool** — not just a Rust library, but a platform that generates documentation, dissectors, and interop code from the same struct definition.

**Target use cases:** Protocol documentation for standards bodies, Wireshark integration for debugging, C/Python interop for mixed-language projects, onboarding new team members who need to understand wire formats.

### Wire Format Diagram Generation [MUST]

- [ ] `bitframe::diagram::ascii(CcsdsPrimaryHeader)` → RFC-style ASCII bit diagram:
  ```
  0                   1                   2                   3
  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  |  ver  |T|S|         APID          |SF |       seq_count       |
  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  |            pkt_len                |
  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  ```
- [ ] Available at compile time via proc-macro (generated as doc comment) and at runtime via function call
- [ ] Supports nested structs (indented sub-diagrams)
- [ ] Behind `diagram` feature flag

### Wireshark Dissector Generation [SHOULD]

- [ ] `bitframe-wireshark` companion crate
- [ ] Generates a Lua dissector from any `#[bitframe]` struct:
  ```lua
  -- Auto-generated by bitframe for CcsdsPrimaryHeader
  local proto = Proto("ccsds_primary", "CCSDS Primary Header")
  proto.fields.version = ProtoField.uint8("ccsds.version", "Version", base.DEC, nil, 0xE0)
  ```
- [ ] Generated dissector includes field names, bit ranges, and enum value tables
- [ ] CLI tool: `bitframe-dissector --struct CcsdsPrimaryHeader --output ccsds.lua`
- [ ] Alternative: generate Wireshark C plugin via wsdf-style approach for higher performance

### C Header Generation [SHOULD]

- [ ] `bitframe-cgen` companion crate or CLI
- [ ] Generates C header with:
  - Struct definition with exact byte size (`typedef struct __attribute__((packed)) { ... }`)
  - Accessor macros or inline functions matching bitframe's field extraction
  - Compile-time size assertions
- [ ] Useful for mixed Rust/C projects (embedded firmware, flight software)
- [ ] Guarantees wire-compatible layout between generated C and Rust code

### Protocol Spec Import [SHOULD]

- [ ] `bitframe-import` tool that reads protocol descriptions and generates `#[bitframe]` structs:
  - **DBC files** (CAN bus) → one struct per message, signals as fields
  - **XTCE files** (CCSDS/space) → structs from ParameterType + Container definitions
  - **CSV/JSON tables** (generic) → field name, bit offset, bit width, type
- [ ] Generated code is human-readable and editable (not hidden behind a build script)
- [ ] Round-trip: import → edit → the struct is the source of truth going forward

### Layout Diff Tool [SHOULD]

- [ ] `bitframe diff v1::Header v2::Header` — shows which fields changed, moved, resized
- [ ] Useful for protocol versioning: "v2 added a 4-bit priority field at bit 16, shifting seq_count to bit 20"
- [ ] Output as text diff or JSON for CI integration
- [ ] Detect wire-incompatible changes and warn

---

## v0.8.0 — Performance & Scale

**Theme:** Parse billions of packets. Handle the hot path.

**Why now:** Ground stations, telemetry pipelines, and SDR applications process millions of packets per second. After correctness is proven (v0.6) and tooling exists (v0.7), optimize for throughput.

**Target use cases:** Satellite ground stations (100k+ packets/sec), CAN bus analyzers (8k frames/sec per bus × dozens of buses), ADS-B receivers (1090MHz, thousands of messages/sec), network monitoring.

### Batch Parsing API [MUST]

- [ ] `FooRef::parse_batch(bytes: &[u8]) -> BatchIter<FooRef>` — iterate over consecutive fixed-size packets
- [ ] Iterator yields `(FooRef, offset)` pairs without re-checking buffer length per packet
- [ ] Single bounds check at the start: `bytes.len() / SIZE_BYTES` packets available
- [ ] Enables vectorization-friendly access patterns

### Branchless Field Extraction [MUST]

- [ ] Generated accessors use branchless shift/mask for all field types
- [ ] No conditional branches for endianness (resolved at compile time via const generics)
- [ ] No conditional branches for byte boundary crossing (all paths compile-time determined)
- [ ] Benchmark: verify zero branches in `cargo asm` output for accessor methods

### Byte-Aligned Fast Path [MUST]

- [ ] Fields that start and end on byte boundaries use direct `u16::from_be_bytes()` / `u32::from_be_bytes()`
- [ ] The proc-macro detects alignment at compile time and emits the optimal code path
- [ ] Common case: CCSDS `pkt_len` (bits 32..48) uses `u16::from_be_bytes([bytes[4], bytes[5]])`
- [ ] No runtime alignment check — the decision is made entirely by the macro

### Memory-Mapped Parsing [SHOULD]

- [ ] `FooRef::from_slice_unchecked(bytes: &[u8]) -> FooRef` — skip length check when caller guarantees sufficient bytes
- [ ] Useful for mmap'd files where the OS guarantees page-level access
- [ ] Behind `unchecked` feature flag with clear documentation about safety requirements
- [ ] NOT `unsafe` — the function itself is safe, but callers should understand the panic-if-short guarantee is removed

### Cache-Friendly Multi-Field Access [SHOULD]

- [ ] `header.fields()` → returns all fields as a tuple in one pass (single byte-scan)
- [ ] Useful when you need 3+ fields: avoids repeated byte indexing
- [ ] Benchmark against individual accessor calls to verify improvement

### SIMD Exploration [SHOULD]

- [ ] Research: evaluate BMI2 `PEXT`/`PDEP` for batch bit extraction on x86_64
- [ ] Research: evaluate NEON equivalents for ARM
- [ ] Prototype: SIMD-accelerated batch parsing for identical packet types
- [ ] Document findings even if SIMD doesn't win (AMD Zen 1/2 `PEXT` is microcode-slow)
- [ ] Behind `simd` feature flag, never required

---

## v0.9.0 — Advanced Patterns

**Theme:** Handle the real-world patterns that simple layouts can't express.

**Why now:** Production protocol stacks need more than flat structs. This version addresses the patterns users encounter once they move beyond basic headers — without crossing into deku/binrw's variable-length territory.

**Target use cases:** Protocol version negotiation, multi-variant command/telemetry dispatch, checksum-protected frames, cross-language teams, runtime introspection for debugging tools.

### Variant Layouts (Fixed-Size Unions) [MUST]

- [ ] Dispatch on a field value to select a fixed-size variant:
  ```rust
  #[bitframe]
  pub struct TmPacket {
      pub primary: CcsdsPrimaryHeader,
      #[bitframe(match_on = "primary.apid")]
      pub payload: TmPayload,
  }

  #[bitframe_variants]
  pub enum TmPayload {
      #[bitframe(when = "0x100")]
      Housekeeping(HousekeepingPayload),    // 32 bytes
      #[bitframe(when = "0x200")]
      Science(SciencePayload),              // 32 bytes
  }
  ```
- [ ] All variants must have the same `SIZE_BITS` (this is NOT variable-length)
- [ ] Accessor returns `Result<TmPayloadVariant, Error>` for unknown discriminants
- [ ] The discriminant field is read from the parent view — no context passing, no `ctx` attribute
- [ ] Scope boundary: if variants have different sizes, use manual chaining — bitframe does not do variable-length

### Checksum / CRC Fields [MUST]

- [ ] Declare a field as a checksum over a byte range:
  ```rust
  #[bitframe]
  pub struct ProtectedFrame {
      pub header: FrameHeader,          // 4 bytes
      pub data:   [u8; 28],             // 28 bytes
      #[bitframe(crc16_ccitt, over = "0..32")]
      pub checksum: u16,                // CRC over bytes 0..32
  }
  ```
- [ ] Built-in algorithms: CRC-16-CCITT (CCSDS), CRC-32 (Ethernet), XOR checksum, simple sum
- [ ] On parse: `parse()` succeeds regardless of CRC; `parse_verified()` validates the CRC and returns `Err(Error::ChecksumMismatch)` on failure
- [ ] On encode: `finalize()` automatically computes and writes the checksum
- [ ] Custom algorithms via a trait: `trait Checksum { fn compute(bytes: &[u8]) -> u64; }`

### Field Validators [SHOULD]

- [ ] Declarative constraints beyond range checks:
  ```rust
  #[bitframe]
  pub struct CcsdsPrimaryHeader {
      #[bitframe(must_equal = 0)]
      pub version: u3,                  // CCSDS spec: must always be 0
      pub is_telecommand: bool,
      pub has_secondary: bool,
      pub apid: u11,
      pub seq_flags: u2,
      pub seq_count: u14,
      #[bitframe(valid_range = 1..=65535)]
      pub pkt_len: u16,                 // pkt_len of 0 is invalid per spec
  }
  ```
- [ ] `parse()` ignores validators (accept anything on the wire)
- [ ] `parse_strict()` enforces all validators
- [ ] `validate()` method on views: returns `Vec<ValidationError>` listing all violations
- [ ] Writer setters enforce validators by default (panic on violation)

### Runtime Layout Reflection [SHOULD]

- [ ] `FooRef::layout() -> &'static [FieldDescriptor]` — array of field metadata:
  ```rust
  pub struct FieldDescriptor {
      pub name: &'static str,
      pub bit_offset: usize,
      pub bit_width: usize,
      pub type_name: &'static str,    // "u11", "bool", "SeqFlags"
      pub is_padding: bool,
  }
  ```
- [ ] Enables generic tooling: hex dump with field overlays, protocol analyzers, test vector generators
- [ ] Available at compile time as a `const` array
- [ ] Used internally by diagram generation (v0.7) and diff tool (v0.7)

### Python Bindings [SHOULD]

- [ ] `bitframe-py` crate using PyO3
- [ ] Python class per `#[bitframe]` struct with attribute access:
  ```python
  header = CcsdsPrimaryHeader.parse(bytes)
  print(header.apid)          # 31
  print(header.is_telecommand) # False
  ```
- [ ] Zero-copy: Python object wraps the Rust view, not a copy
- [ ] Useful for ground station scripting, Jupyter notebook analysis, test automation
- [ ] Auto-generates `.pyi` stub files for IDE support

### Bit-Order Control [SHOULD]

- [ ] Per-field or per-struct bit ordering: MSb0 (default) or LSb0
- [ ] Critical for ARINC 429 (label field is LSB-first) and CAN signals (Motorola vs Intel byte order)
- [ ] Explicit syntax: `#[bitframe(bit_order = lsb0)]`
- [ ] Combinations of bit order + byte order are documented with worked examples

---

## v1.0 — Stability

**Theme:** Lock it down.

### Stability Guarantees [MUST]

- [ ] Bit ordering rules documented and frozen
- [ ] Public error types frozen
- [ ] Semver rules for generated code (macro expansion changes are breaking if wire semantics change)
- [ ] Wire semantics stability: golden vectors required for any parse/encode change
- [ ] MSRV policy documented and tested in CI (minimum Rust version guaranteed for 1.x series)

### `no_std` Profile [MUST]

- [ ] CI tests against a `no_std` target (e.g., `thumbv7em-none-eabihf`)
- [ ] Binary size regression tests for embedded targets
- [ ] Document which features require `std` vs `alloc` vs `core`-only

### Documentation [MUST]

- [ ] Complete rustdoc with examples for every public type and trait
- [ ] User guide (mdBook or similar) covering:
  - Getting started
  - Bit numbering and endianness
  - Enum fields
  - Nested layouts
  - Encoding
  - Testing strategies
  - Migration from deku/packed_struct
- [ ] Protocol author's guide (how to define a new protocol with bitframe)

### API Surface Audit [MUST]

- [ ] Review all public types for coherence and minimal surface
- [ ] Ensure the public vocabulary is small: `FooRef`, `FooRefMut`, `FooWriter`, `Foo`, `Error`
- [ ] Remove any accidental public types from macro expansion

---

## Target Use Cases by Domain

These are the domains bitframe is designed to serve, ordered by gap severity (how poorly the current Rust ecosystem serves them):

| Priority | Domain | Key Protocol | Gap |
|----------|--------|-------------|-----|
| 1 | Aviation/Aerospace | ARINC 429, MIL-STD-1553 | No Rust crates exist at all |
| 2 | Automotive | CAN bus signals (DBC) | DBC codegen produces 30k+ lines; no declarative alternative |
| 3 | Space/Satellite | CCSDS, ECSS PUS | 5+ competing crates, all hand-coded bit math |
| 4 | Radio/SDR | ADS-B, DMR, P25 | adsb_deku proves the model but inherits deku's weight |
| 5 | Embedded/IoT | LoRaWAN, BLE, Zigbee, sensor frames | Every project re-invents bit extraction |
| 6 | Industrial | EtherCAT PDI, Modbus coils | Process Data Image bit mapping is unsolved |
| 7 | Networking | Custom headers, MPLS, VXLAN, GRE | Well-served for standard protocols, gaps in custom ones |
| 8 | Gaming | Bit-packed state sync | Real need, smaller community |
| 9 | Storage | Filesystem flags, partition entries | Mostly byte-aligned |
| 10 | Security | ASN.1 bit strings, TLS headers | Well-served by nom-based parsers |

---

## Competitor Learnings (What Not To Do)

These lessons are drawn from deep analysis of deku, bilge, modular-bitfield, packed_struct, binrw, bitvec, and zerocopy.

| # | Lesson | Source |
|---|--------|--------|
| 1 | **Never depend on bitvec.** Effectively abandoned, heavy transitive deps (funty/radium/tap/wyz), slow in debug mode. | deku, packed_struct, nom |
| 2 | **Never require endianness context passing.** Users universally complain about deku's `#[deku(ctx)]` pattern. | deku #79, #217, #501 |
| 3 | **Never use string-based attribute values.** `#[deku(endian = "big")]` prevents IDE support. Use Rust idents. | deku, landscape.md own recommendation |
| 4 | **Never produce opaque proc-macro errors.** Errors must point to the exact field/attribute, not the derive line. | deku, packed_struct |
| 5 | **Never force power-of-two enum variants.** modular-bitfield requires 2^N variants, forcing placeholder variants. | modular-bitfield |
| 6 | **Never require alloc for core parsing.** `FooRef::parse()` must work on bare metal. | deku (pre-v0.20) |
| 7 | **Never break API without a migration path.** deku v0.17 broke all downstream code. | deku |
| 8 | **Never conflate "register bitfield" with "wire format view".** Different problem, different solution. | bilge, modular-bitfield |
| 9 | **Ship benchmarks.** No competitor publishes comparative benchmarks. Users can't evaluate. | all competitors |
| 10 | **Keep the dependency tree tiny.** bitfield-struct's zero-runtime-dep approach is praised. | bitfield-struct |

---

## Key Differentiators (Why bitframe Exists)

No existing crate combines all three:

```
                Bit-level fields
                     /\
                    /  \
                   / bit \
                  / frame \
                 /__________\
  Zero-copy views ── Byte-buffer parsing
```

- **deku**: bit-level + byte-buffer, but copies into owned structs (no views)
- **binary-layout**: views + byte-buffer, but byte-aligned only (no bit fields)
- **bilge/modular-bitfield**: bit-level fields, but no byte-buffer parsing, no views
- **zerocopy**: views, but byte-aligned only
- **nom**: bit-level + zero-copy, but no declarative struct mapping

**bitframe is the only library that declares bit-level struct fields, parses from `&[u8]`, and returns a borrowed zero-copy view.**

Additionally, `FooRefMut` for in-place field mutation over `&mut [u8]` is something **no competitor offers**.
