# Vision

`bitframe` is a **bit-level packet layout** library for Rust.

## The One-Sentence Pitch

You describe your packet's bit fields in a struct, and `bitframe` generates zero-copy readers and writers that do all the bit math correctly — so you never write another shift/mask bug.

## Problem Statement

In space telemetry, embedded systems, and compact binary protocols, data is packed at the **bit** level. A single CCSDS space packet header crams 6 fields into 6 bytes at widths of 3, 1, 1, 11, 2, 14, and 16 bits. A CAN bus identifier is 11 or 29 bits. Sensor readings are 12-bit ADC values packed back-to-back.

Today, Rust developers handle this by:

1. **Manual shifting/masking** — hard to review, easy to get wrong, and the bugs produce *plausible wrong values* (not crashes, not panics — just quietly wrong data)
2. **"Bitfield" crates** (`modular-bitfield`, `bilge`, `bitfield-struct`) — these model in-memory structs with bit-sized fields, like C bitfields. They copy data into owned structs, which is great for configuration registers but wasteful for high-throughput packet parsing.
3. **General binary parsers** (`nom`, `deku`, `binrw`) — these build owned structs from byte streams. They're designed for variable-length, self-describing formats. Overkill and wrong abstraction for fixed-size bit-packed headers.

None of these give you: **a declared layout that parses directly from raw bytes as a borrowed view, with zero allocation and zero copying.**

## Why This Matters

### Bit-level bugs are silent killers

```
// Intended: read bits 5-15 as an 11-bit APID
let apid = ((bytes[0] as u16 & 0x07) << 8) | bytes[1] as u16;

// Bug: mask should be 0x07, but someone wrote 0x0F (4 bits, not 3)
// Result: APID values look fine most of the time, but occasionally
// include a stray bit from the sec_hdr flag. You won't notice until
// a specific packet combination triggers routing to the wrong handler.
```

These bugs don't crash. They don't panic. They produce **plausible wrong numbers** that silently corrupt your data pipeline. They get caught weeks later, during integration testing, or worse — after launch.

### The cost of "just write the bit math"

- Every protocol change requires re-auditing shift/mask code
- Code review requires mentally computing bit offsets across byte boundaries
- Testing requires manually constructing byte arrays and verifying each field
- No way to auto-generate documentation of the wire format

`bitframe` makes the layout **the source of truth** — reviewable, testable, and correct by construction.

## Core Thesis

**Make bit layouts explicit and testable, and make parsing a view rather than a copy.**

- **Explicit**: the struct declaration IS the wire format specification
- **Testable**: golden vectors (known bytes → known field values) catch regressions
- **View, not copy**: parsing returns a thin wrapper around `&[u8]` that reads bits on demand

## What "Zero-Copy View" Means

```
Traditional parsing:                    bitframe:

bytes ──► decode ──► MyPacket {         bytes ──► MyPacketRef { &bytes }
            version: 5,                   │
            apid: 31,                     ├── .version() reads bits 0-2
            seq_count: 66,                ├── .apid() reads bits 5-15
            ...                           ├── .seq_count() reads bits 18-31
          }                               └── (no copying, no allocation)
          (copies all fields)
```

The view (`MyPacketRef`) holds a reference to the original bytes. Each accessor reads the relevant bits on demand. This is ideal for:

- **High-throughput telemetry**: parse millions of packets per second without allocation pressure
- **Embedded systems**: parse on microcontrollers with kilobytes of RAM
- **Protocol stacks**: peek at headers without copying payloads

## Target Users

Developers working on:

- **Space telemetry (CCSDS)** — packet/frame handling, telecommand
- **Aviation (ARINC 429, ADS-B)** — avionics buses and transponder messages
- **Automotive (CAN bus)** — signal extraction from DBC-defined frames
- **Telemetry pipelines** — ground systems ingesting millions of packets per second
- **Radio/SDR** — ADS-B, DMR, P25 and other bitstream protocols
- **Embedded/IoT** — LoRaWAN, BLE, Zigbee, sensor data on constrained devices
- **Industrial** — EtherCAT process data, Modbus coil registers
- **Low-level networking** — any protocol with bit-packed fixed headers

## Non-Goals

These keep the scope tight:

- **Variable-length fields** or self-describing formats (use serde/protobuf/binrw)
- **Runtime format descriptions** or reflection
- **General binary parsing** (nom/deku/binrw are better fits)
- **In-memory bitfields** without a wire format (use modular-bitfield/bilge)

## What Must Stay True

1. **Borrowed parsing**: the primary parse API returns a view that borrows the input bytes.
2. **No hidden allocations**: encode/decode does not allocate unless the user opts into a convenience API.
3. **No `unsafe` in the public contract**: if `unsafe` exists internally, it must be justified and contained.
4. **Protocol clarity**: bit ordering and endianness rules are explicit in the API and documentation.

## Future-Proofing

Protocols evolve, but bit-packed headers and frames are long-lived (CCSDS primary header hasn't changed in decades). `bitframe` stays future-proof by:

- locking down a single, documented bit numbering model (MSB-first)
- treating variable-length fields as out-of-scope and composable with other parsers
- investing in golden vectors, fuzzing, and diagnostics so macro evolution does not silently change wire semantics
