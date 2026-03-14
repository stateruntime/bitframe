# Audit

A product audit and quality gate for `bitframe`. Competitive analysis has moved to [landscape.md](landscape.md).

---

## What We're Building

`bitframe` is a **bit-level packet layout** tool that generates:

- a **borrowed view** over `&[u8]` (zero-copy field access)
- a **mutable view** over `&mut [u8]` (in-place field mutation)
- a **writer** that encodes into a caller-provided buffer

The design center is fixed-size packet headers and frames used in telemetry, embedded buses, and network protocols.

## The Real Problem We Solve

In bit-packed protocols, the common failure mode is **silent corruption**:

- field widths drift between implementations
- endianness/bit numbering assumptions are implicit
- validation is scattered or missing
- manual shifts are hard to review and harder to regression test

When this happens, systems keep running while producing wrong values. `bitframe` makes layouts explicit, testable, and reviewable.

## What We Do Right (Non-Negotiables)

1. **Views, not structs:** decoding returns a view that borrows the input bytes.
2. **No hidden allocation:** core decode/encode does not allocate.
3. **Fixed-size formats:** layout size is known at compile time.
4. **Bit-ordering is explicit:** one default model, documented, with deliberate overrides.
5. **Errors are structured:** no stringly-typed parsing failures.
6. **Zero runtime dependencies:** only proc-macro compile-time deps. No bitvec, no runtime libraries.

## Where We Can Lose

- **Macro error messages are bad.** If users need `cargo expand` to debug, we lose.
- **Ambiguous bit semantics.** If endianness and bit numbering are unclear, we lose trust fast.
- **Too much generality.** If we become a general parsing framework, we lose simplicity.
- **Slow adoption loop.** If the first 5 minutes require reading a design doc, we lose.
- **Dependency weight creep.** If we add runtime deps, we lose our advantage over deku.

## Future Pressures

- **Long-lived wire formats:** packet layouts shipped today may be parsed for decades.
- **Evolving standards:** protocol revisions will add fields or variants.
- **Macro expectations:** users demand excellent diagnostics and stable semantics across versions.
- **Embedded growth:** Rust adoption in safety-critical embedded (aviation, automotive, space) is accelerating.

Our response: strong compatibility discipline (golden vectors + roundtrips), explicit bit semantics documentation, and modular features rather than a kitchen-sink framework.

---

## Internal Audit: Design Document Consistency

### Resolved Issues (fixed in v0.1.0)

| # | Severity | Issue | Files Affected | Resolution |
|---|----------|-------|---------------|------------|
| 1 | **High** | `parse()` return type: tuple vs single value | DESIGN.md, README.md, DX doc, landscape.md | `parse()` returns `Result<(FooRef, &[u8]), Error>` everywhere |
| 2 | **High** | Error struct fields: bits vs bytes, with/without layout name | DESIGN.md, DX doc | `TooShort { needed_bytes, have_bytes }` — layout name in `Display` only |
| 3 | **High** | `#[repr(u2)]` is invalid Rust | DESIGN.md | Use `#[repr(u8)]` + `#[bitframe_enum(bits = 2)]` |
| 4 | **High** | No proc-macro crate structure | All docs | Design workspace: `bitframe`, `bitframe-derive`, `bitframe-core` |
| 5 | **High** | Endianness attribute uses strings that landscape.md criticizes | DESIGN.md, landscape.md | Use Rust idents: `#[bitframe(little_endian)]` |
| 6 | **Medium** | `finalize()` vs `finish()` naming inconsistency | DX doc, landscape.md | Standardize on `finalize()` |
| 7 | **Medium** | `TryFrom<&[u8]>` behavior undefined relative to `parse()`/`parse_exact()` | DESIGN.md | `TryFrom` behaves like `parse()` (accepts extra bytes) |
| 8 | **Medium** | `Eq`/`Hash` missing from `FooRef` without justification | DESIGN.md | Add `Eq`. Add `Hash` behind `std` feature. |
| 9 | **Medium** | Writer field-skip semantics undefined | DESIGN.md | Default to zero. Add `finalize_checked()` for strict mode. |
| 10 | **Medium** | Non-byte-aligned layout edge case unspecified | DESIGN.md | `SIZE_BYTES = ceil(SIZE_BITS / 8)`. Trailing bits ignored on read, zeroed on write. |
| 11 | **Medium** | Bit-sized type implementation entirely undesigned | DESIGN.md, DX doc | Document type design: newtype over smallest containing int, full trait coverage. |
| 12 | **Medium** | AUDIT.md and landscape.md overlap | AUDIT.md, landscape.md | AUDIT.md = internal quality. landscape.md = competitive analysis + use cases. |
| 13 | **Low** | `to_owned()` interaction with `ToOwned` trait | DESIGN.md | Implement `ToOwned` trait. |
| 14 | **Low** | VERSION file sync with Cargo.toml is manual | Cargo.toml, VERSION | Accept as-is until release automation is built. |
| 15 | **Low** | landscape.md contains specific star/download counts that will become stale | landscape.md | Add "as of March 2026" caveat; accept staleness. |
| 16 | **Low** | `space-protocols` reference in AUDIT.md is unexplained | AUDIT.md | Remove or replace with `bitframe-protocols` (planned examples crate). |

### Missing Considerations (design gaps)

| # | Gap | Impact | Status |
|---|-----|--------|--------|
| 1 | Zero-width fields (`u0`) | Edge case | Open — emit compile error |
| 2 | Empty structs (`#[bitframe] struct Empty {}`) | Edge case | Open — emit compile error |
| 3 | Interaction with `#[repr(C)]` on annotated struct | User confusion | Open — strip or error on `#[repr]` attrs |
| 4 | Visibility of generated types | API correctness | **Done** — respects source struct visibility |
| 5 | Multiple `#[bitframe]` structs in same module | Name collisions | **Done** — tested, no conflicts |
| 6 | Exhaustive enum → non-exhaustive migration | Breaking change | v0.2.0 — document migration path |
| 7 | Checksum/CRC validation hooks | Common protocol need | Out of scope — document as composition pattern |
| 8 | Conditional secondary headers | Common protocol need | Out of scope — document as composition pattern |
| 9 | `no_std` testing strategy | Quality | **Partial** — `--no-default-features` tested in CI |
| 10 | Compile-fail test infrastructure | Quality | Open — add `trybuild` dev-dependency |
| 11 | Benchmark infrastructure | Credibility | v0.5.0 — add `criterion` benchmarks |
| 12 | Macro hygiene (generated names) | Correctness | **Done** — tested in golden tests |
| 13 | Incremental compilation impact | DX | v0.2.0 — profile and minimize |
| 14 | `cargo expand` readability of generated code | DX | **Done** — clean expansion with doc comments |

---

## Developer Experience Bar

To be credible, we must deliver:

- **Excellent diagnostics:** "field overlaps previous field at bits N..M", not "trait bound not satisfied."
- **Debuggable output:** `cargo expand` on a `#[bitframe]` struct produces readable Rust code.
- **Golden vectors:** published byte sequences from real protocol specs (CCSDS, CAN, ADS-B).
- **Spec-style tests:** acceptance tests written with `behave` so they read as contracts.
- **First-5-minutes success:** `cargo add bitframe`, write a struct, parse bytes. No design doc needed.

## v1.0 Acceptance Criteria

We do not ship v1.0 until:

- [ ] Bit ordering + endianness semantics are locked and documented with worked examples
- [ ] Error types are stable and cover all claimed failure modes
- [ ] Encode/decode roundtrips are tested for all golden vectors (including byte-boundary edge cases)
- [ ] Fuzzing has run for 10M+ iterations with zero panics
- [ ] Public API surface is reviewed and minimized
- [ ] `no_std` compatibility is CI-verified on an embedded target
- [ ] MSRV is tested in CI
- [ ] Published benchmarks demonstrate competitive performance vs manual bit manipulation
- [ ] User guide covers all features with examples
- [ ] At least 3 real-world protocol definitions ship in `bitframe-protocols`

## Testing Quality Gates

| Gate | Tool | Threshold |
|------|------|-----------|
| Unit + integration | `cargo test` | 100% pass |
| BDD specs | `behave` | All specs pass, no `pending` specs in released version |
| Compile-fail | `trybuild` | All invalid layouts rejected with correct diagnostics |
| Fuzzing | `cargo-fuzz` | 10M+ iterations, zero panics |
| Lints | `clippy` (pedantic + nursery) | Zero warnings |
| Dependencies | `cargo deny` | Zero advisories, zero unapproved licenses |
| Coverage | `cargo-llvm-cov` | Target 90%+ line coverage |
| MSRV | CI matrix | Passes on declared `rust-version` |
| `no_std` | CI cross-compile | Compiles for `thumbv7em-none-eabihf` |
| Benchmarks | `criterion` | No regressions between releases |
