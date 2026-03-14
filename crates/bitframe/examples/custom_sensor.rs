//! Custom 3-byte sensor reading with `#[bitframe_enum]`
//!
//! A simple custom protocol: 3 bytes carrying a 12-bit temperature,
//! a 2-bit status enum, and a 10-bit sequence number.
//!
//! Run with: `cargo run --example custom_sensor`

#![allow(clippy::print_stdout, missing_docs)]

use bitframe::prelude::*;

/// Sensor status — 2-bit exhaustive enum (all 4 values covered).
#[bitframe_enum]
#[repr(u8)]
pub enum SensorStatus {
    Ok = 0,
    Warning = 1,
    Error = 2,
    Offline = 3,
}

#[bitframe]
pub struct SensorReading {
    /// 12-bit temperature (0.1 degree units, 0-409.5)
    pub temperature: u12,
    /// 2-bit status code
    pub status: u2,
    /// 10-bit sequence number (wraps at 1023)
    pub seq: u10,
}

fn main() -> Result<(), bitframe::Error> {
    // Temperature = 256 (25.6 degrees), status = 1 (Warning), seq = 42
    // temperature(12) = 0b000100000000 = 256
    // status(2) = 0b01
    // seq(10) = 0b0000101010 = 42
    //
    // Bits: 000100000000_01_0000101010
    // Byte 0: 00010000 = 0x10
    // Byte 1: 0000_01_00 = 0b00000100 = 0x04
    // Byte 2: 00101010 = 0x2A
    let bytes: &[u8] = &[0x10, 0x04, 0x2A];

    let (reading, _) = SensorReadingRef::parse(bytes)?;

    let temp_raw = reading.temperature().value();
    let status = SensorStatus::from_raw(reading.status());

    println!("=== Sensor Reading ===");
    println!(
        "  Temperature:  {} ({:.1} degrees)",
        temp_raw,
        f64::from(temp_raw) / 10.0
    );
    println!("  Status:       {status:?}");
    println!("  Sequence:     {}", reading.seq());
    println!();
    println!("Debug: {reading:?}");

    Ok(())
}
