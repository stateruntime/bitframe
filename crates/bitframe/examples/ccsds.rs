//! CCSDS Space Packet Primary Header (CCSDS 133.0-B-2)
//!
//! The CCSDS primary header is a 6-byte (48-bit) fixed-size header used by
//! every satellite communication system. Fields span non-byte boundaries,
//! making it a perfect showcase for bitframe.
//!
//! Run with: `cargo run --example ccsds`

#![allow(clippy::print_stdout)]

use bitframe::prelude::*;

#[bitframe]
pub struct CcsdsPrimaryHeader {
    /// Packet Version Number (always 0 for current CCSDS)
    pub version: u3,
    /// 0 = telemetry, 1 = telecommand
    pub is_telecommand: bool,
    /// Whether a secondary header is present
    pub has_secondary: bool,
    /// Application Process Identifier
    pub apid: u11,
    /// Sequence flags: 0=continuation, 1=first, 2=last, 3=standalone
    pub seq_flags: u2,
    /// Packet sequence count (wraps at 16383)
    pub seq_count: u14,
    /// Data length minus 1 (number of octets in packet data field - 1)
    pub data_length: u16,
}

fn main() -> Result<(), bitframe::Error> {
    // A real CCSDS packet: telemetry, APID 0x123, standalone, seq 42, 255 data bytes
    let packet: &[u8] = &[
        0x19, 0x23, // version=0, type=1(TC), sec=1, apid=0x123
        0xC0, 0x2A, // seq_flags=3(standalone), seq_count=42
        0x00, 0xFF, // data_length=255
        0xDE, 0xAD, 0xBE, 0xEF, // (payload bytes)
    ];

    let (header, payload) = CcsdsPrimaryHeaderRef::parse(packet)?;

    println!("=== CCSDS Primary Header ===");
    println!("  Version:        {}", header.version());
    println!("  Telecommand:    {}", header.is_telecommand());
    println!("  Secondary hdr:  {}", header.has_secondary());
    println!("  APID:           {:#05x}", header.apid());
    println!("  Seq flags:      {}", header.seq_flags());
    println!("  Seq count:      {}", header.seq_count());
    println!("  Data length:    {}", header.data_length());
    println!("  Payload bytes:  {}", payload.len());
    println!();
    println!("Debug: {header:?}");

    Ok(())
}
