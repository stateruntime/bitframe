# CLAUDE.md

Instructions for AI agents working on this repository.

## Build & Test

```bash
cargo fmt --all --check          # formatting
cargo clippy --all-features --all-targets -- -D warnings  # linting
cargo test --all-features        # all tests
cargo test --no-default-features # no_std compatibility
cargo doc --no-deps --all-features  # docs (RUSTDOCFLAGS=-Dwarnings)
cargo deny check                 # dependency audit
```

Or use `just check` to run the full CI pipeline locally.

## Versioning

- `VERSION` file is the **single source of truth**
- Must match `version` in `Cargo.toml` (CI enforces this)
- To release: update `VERSION`, sync `Cargo.toml`, update `CHANGELOG.md`, tag `vX.Y.Z`

## Code Rules

- `unsafe_code` is **forbidden** — no exceptions
- These are **denied**: `unwrap_used`, `expect_used`, `panic`, `unimplemented`, `todo`, `dbg_macro`, `print_stdout`, `print_stderr`
- Clippy `pedantic` + `nursery` are warnings
- `missing_docs` is a warning (will become deny before v1.0)
- Tests use the `behave` BDD framework — write specs, not just assertions
- MSRV: Rust 1.75

## Project Structure

```
bitframe/
├── Cargo.toml                  # Workspace root (virtual manifest)
├── crates/
│   ├── bitframe/
│   │   ├── Cargo.toml          # Facade crate manifest
│   │   ├── src/
│   │   │   ├── lib.rs          # Re-exports via prelude
│   │   │   ├── types.rs        # u1..u63 bit-sized types
│   │   │   ├── error.rs        # Error enum (TooShort, InvalidEnum)
│   │   │   └── traits.rs       # BitLayout, Parseable traits
│   │   ├── tests/behavior.rs   # BDD specs using behave
│   │   └── examples/           # Runnable examples (ccsds, can_bus, custom_sensor)
│   └── bitframe-derive/
│       ├── Cargo.toml          # Proc-macro crate manifest
│       └── src/
│           ├── lib.rs          # Proc-macro entry points
│           ├── bitframe_attr.rs        # #[bitframe] expansion
│           ├── bitframe_enum_attr.rs   # #[bitframe_enum] expansion
│           ├── field.rs        # Field type → bit width parsing
│           ├── extract.rs      # Bit extraction codegen
│           └── codegen.rs      # FooRef view type codegen
├── docs/                       # Design documents
├── .github/workflows/          # CI and release automation
├── VERSION                     # Single source of truth for version
├── CHANGELOG.md                # Keep a Changelog format
└── justfile                    # Local CI runner
```

## Status

**v0.1.0 — parsing is implemented.** The workspace contains two crates (`bitframe` + `bitframe-derive`), bit-sized types `u1`..`u63`, `#[bitframe]` and `#[bitframe_enum]` proc-macros, and a BDD test suite with CCSDS, CAN, and ADS-B golden tests. Writers and mutable views are planned for v0.2+.

## Key Design Decisions

1. **Views over copies** — `FooRef<'a>` borrows `&[u8]`, reads fields on demand
2. **Convention over configuration** — field order = wire order, big-endian default, enum width inferred
3. **You write values, not types** — Writer setters accept plain integers, validate range internally
4. **Zero runtime dependencies** — only proc-macro compile-time deps
5. **The struct is the spec** — if you can read the struct, you understand the protocol

See `docs/DESIGN.md` for the full API contract and `docs/DEVELOPER_EXPERIENCE.md` for DX standards.
