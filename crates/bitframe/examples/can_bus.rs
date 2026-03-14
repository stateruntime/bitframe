//! CAN 2.0A Standard Frame Header
//!
//! A simplified CAN bus standard-ID frame header (32 bits / 4 bytes).
//! The 11-bit identifier and 4-bit DLC span byte boundaries, which
//! bitframe handles automatically.
//!
//! Run with: `cargo run --example can_bus`

#![allow(clippy::print_stdout)]

use bitframe::prelude::*;

#[bitframe]
pub struct CanStdFrame {
    /// 11-bit standard identifier
    pub id: u11,
    /// Remote Transmission Request
    pub rtr: bool,
    /// Identifier Extension (0 = standard frame)
    pub ide: bool,
    /// Reserved bit
    pub _reserved: u1,
    /// Data Length Code (0-8 bytes)
    pub dlc: u4,
    /// Padding to fill 32 bits
    pub _pad: u14,
}

fn main() -> Result<(), bitframe::Error> {
    // CAN frame: ID=0x123, RTR=0, IDE=0, DLC=8
    // id(11)=0b00100100011, rtr=0, ide=0, reserved=0, dlc(4)=0b1000
    // Byte 0: 00100100 = 0x24
    // Byte 1: 011_0_0_0_10 = 0b01100010 = 0x62 (wait, careful)
    // Bits 0-7:  id[0..8] = 00100100 = 0x24
    // Bits 8-10: id[8..11] = 011
    // Bit 11: rtr = 0
    // Bit 12: ide = 0
    // Bit 13: reserved = 0
    // Bits 14-17: dlc = 1000
    // Byte 1: 011_0_0_0_10 = 0b01100010 = 0x62
    // Byte 2: 00_000000 = 0x00
    // Byte 3: 00000000 = 0x00
    let bytes: &[u8] = &[0x24, 0x62, 0x00, 0x00];

    let (frame, _) = CanStdFrameRef::parse(bytes)?;

    println!("=== CAN Standard Frame ===");
    println!("  ID:       {:#05x} ({})", frame.id(), frame.id());
    println!("  RTR:      {}", frame.rtr());
    println!("  IDE:      {}", frame.ide());
    println!("  DLC:      {}", frame.dlc());
    println!(
        "  Size:     {} bits / {} bytes",
        CanStdFrameRef::SIZE_BITS,
        CanStdFrameRef::SIZE_BYTES
    );
    println!();
    println!("Debug: {frame:?}");

    Ok(())
}
