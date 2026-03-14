# Changelog

All notable changes to `bitframe` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**Every PR with user-facing changes must add an entry under [Unreleased].**

## [Unreleased]

### Added

- Reference implementation test vectors from published sources:
  - CCSDS: PUS TC[17,1] and TM[17,2] from spacepackets, idle packet from KubOS
  - ADS-B: DF=11 and DF=17 messages from "The 1090 Megahertz Riddle" (TU Delft) and adsb_deku
  - CAN/J1939: OBD-II standard frame and J1939 EEC1 extended frame via SocketCAN LINKTYPE_CAN_SOCKETCAN format

## [0.1.0] — 2026-03-13

### Added

- Workspace structure with two crates: `bitframe` (facade) and `bitframe-derive` (proc-macro)
- Bit-sized unsigned types `u1`..`u63` with full trait implementations (Debug, Display, Hex, Binary, Octal, BitOps, PartialEq/Ord with backing type)
- `#[bitframe]` proc-macro generating zero-copy `FooRef<'a>` view types over `&[u8]`
- `#[bitframe_enum]` proc-macro with automatic exhaustive/non-exhaustive detection
- `Error` type with `TooShort` and `InvalidEnum` variants
- `BitLayout` and `Parseable` traits for generic code over layouts
- `parse()` returns `Result<(FooRef, &[u8]), Error>` with remainder
- `parse_exact()` for strict length matching
- `TryFrom<&[u8]>` and `AsRef<[u8]>` on generated views
- Byte-aligned field optimization in codegen (direct `from_be_bytes` reads)
- BDD test suite using `behave` with CCSDS, CAN, and ADS-B golden tests
- `no_std` support (default `std` feature gates `std::error::Error`)
- CI pipeline: fmt, clippy (pedantic + nursery), tests, docs, MSRV 1.75, cargo-deny
- Release automation for crates.io via tag-triggered workflow

[Unreleased]: https://github.com/piotrzkowskij/bitframe/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/piotrzkowskij/bitframe/releases/tag/v0.1.0
