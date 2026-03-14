//! Behavior tests for the `bitframe` public surface.
//!
//! These tests use the `behave` DSL so they read like specifications.

#![allow(clippy::unwrap_used, dead_code)]

use behave::prelude::*;
use bitframe::prelude::*;

behave! {
    "bitframe crate" {
        "exports a prelude module" {
            {
                #[allow(unused_imports)]
                use bitframe::prelude::*;
            }

            expect!(true).to_be_true()?;
        }
    }

    "bit-sized types" {
        "construction" {
            "u3::new creates a valid value" {
                let v = u3::new(5);
                expect!(v.value()).to_equal(5u8)?;
            }

            "u3::new(0) creates zero" {
                let v = u3::new(0);
                expect!(v.value()).to_equal(0u8)?;
            }

            "u3::new at MAX succeeds" {
                let v = u3::new(7);
                expect!(v.value()).to_equal(7u8)?;
            }

            "u11::new creates a valid value" {
                let v = u11::new(2047);
                expect!(v.value()).to_equal(2047u16)?;
            }

            "u31::new creates a valid value" {
                let v = u31::new(100_000);
                expect!(v.value()).to_equal(100_000u32)?;
            }

            "u63::new creates a valid value" {
                let v = u63::new(1_000_000);
                expect!(v.value()).to_equal(1_000_000u64)?;
            }

            "try_new succeeds for valid value" {
                let v = u3::try_new(5);
                expect!(v.is_ok()).to_be_true()?;
                expect!(v.unwrap().value()).to_equal(5u8)?;
            }

            "try_new fails for overflow" {
                let v = u3::try_new(8);
                expect!(v.is_err()).to_be_true()?;
            }

            "try_new error has correct metadata" {
                let err = u3::try_new(8).unwrap_err();
                expect!(err.type_name()).to_equal("u3")?;
                expect!(err.max()).to_equal(7u64)?;
                expect!(err.actual()).to_equal(8u64)?;
                expect!(err.bits()).to_equal(3u32)?;
            }

            "ZERO constant is zero" {
                expect!(u3::ZERO.value()).to_equal(0u8)?;
                expect!(u11::ZERO.value()).to_equal(0u16)?;
                expect!(u31::ZERO.value()).to_equal(0u32)?;
                expect!(u63::ZERO.value()).to_equal(0u64)?;
            }

            "const construction works" {
                {
                    const V: u3 = u3::new(5);
                    expect!(V.value()).to_equal(5u8)?;
                }
            }
        }

        "constants" {
            "WIDTH is correct" {
                expect!(u1::WIDTH).to_equal(1u32)?;
                expect!(u3::WIDTH).to_equal(3u32)?;
                expect!(u7::WIDTH).to_equal(7u32)?;
                expect!(u11::WIDTH).to_equal(11u32)?;
                expect!(u15::WIDTH).to_equal(15u32)?;
                expect!(u24::WIDTH).to_equal(24u32)?;
                expect!(u31::WIDTH).to_equal(31u32)?;
                expect!(u48::WIDTH).to_equal(48u32)?;
                expect!(u63::WIDTH).to_equal(63u32)?;
            }

            "MAX is correct" {
                expect!(u1::MAX).to_equal(1u8)?;
                expect!(u2::MAX).to_equal(3u8)?;
                expect!(u3::MAX).to_equal(7u8)?;
                expect!(u7::MAX).to_equal(127u8)?;
                expect!(u9::MAX).to_equal(511u16)?;
                expect!(u11::MAX).to_equal(2047u16)?;
                expect!(u15::MAX).to_equal(32767u16)?;
                expect!(u24::MAX).to_equal(16_777_215u32)?;
                expect!(u31::MAX).to_equal(2_147_483_647u32)?;
            }
        }

        "value extraction" {
            "value() returns the stored integer" {
                let v = u11::new(42);
                expect!(v.value()).to_equal(42u16)?;
            }

            "From widening works" {
                let v = u11::new(42);
                let raw: u16 = u16::from(v);
                expect!(raw).to_equal(42u16)?;
            }

            "TryFrom narrowing succeeds for valid value" {
                let result = u11::try_from(42u16);
                expect!(result.is_ok()).to_be_true()?;
                expect!(result.unwrap().value()).to_equal(42u16)?;
            }

            "TryFrom narrowing fails for overflow" {
                let result = u3::try_from(8u8);
                expect!(result.is_err()).to_be_true()?;
            }
        }

        "cross-type comparison" {
            "u11 equals backing type" {
                let v = u11::new(42);
                expect!(v == 42u16).to_be_true()?;
                expect!(42u16 == v).to_be_true()?;
            }

            "u3 equals backing type" {
                let v = u3::new(5);
                expect!(v == 5u8).to_be_true()?;
            }

            "u3 not equal to different value" {
                let v = u3::new(5);
                expect!(v == 6u8).to_be_false()?;
            }

            "u11 ordering with backing type" {
                let v = u11::new(42);
                expect!(v > 41u16).to_be_true()?;
                expect!(v < 43u16).to_be_true()?;
                expect!(41u16 < v).to_be_true()?;
            }
        }

        "bitwise operations" {
            "AND" {
                let a = u3::new(0b101);
                let b = u3::new(0b110);
                expect!((a & b).value()).to_equal(0b100u8)?;
            }

            "OR" {
                let a = u3::new(0b101);
                let b = u3::new(0b010);
                expect!((a | b).value()).to_equal(0b111u8)?;
            }

            "XOR" {
                let a = u3::new(0b101);
                let b = u3::new(0b110);
                expect!((a ^ b).value()).to_equal(0b011u8)?;
            }

            "NOT masks to bit width" {
                let a = u3::new(0b101);
                expect!((!a).value()).to_equal(0b010u8)?;
            }

            "NOT of zero is MAX" {
                expect!((!u3::ZERO).value()).to_equal(u3::MAX)?;
            }
        }

        "formatting" {
            "Debug shows type name" {
                let v = u11::new(42);
                {
                    let s = format!("{v:?}");
                    expect!(s.as_str()).to_equal("u11(42)")?;
                }
            }

            "Display shows value only" {
                let v = u11::new(42);
                {
                    let s = format!("{v}");
                    expect!(s.as_str()).to_equal("42")?;
                }
            }

            "LowerHex" {
                let v = u11::new(42);
                {
                    let s = format!("{v:#x}");
                    expect!(s.as_str()).to_equal("0x2a")?;
                }
            }

            "UpperHex" {
                let v = u11::new(42);
                {
                    let s = format!("{v:#X}");
                    expect!(s.as_str()).to_equal("0x2A")?;
                }
            }

            "Binary" {
                let v = u11::new(42);
                {
                    let s = format!("{v:#b}");
                    expect!(s.as_str()).to_equal("0b101010")?;
                }
            }

            "Octal" {
                let v = u11::new(42);
                {
                    let s = format!("{v:#o}");
                    expect!(s.as_str()).to_equal("0o52")?;
                }
            }
        }

        "edge cases" {
            "u1 min width" {
                expect!(u1::MAX).to_equal(1u8)?;
                expect!(u1::WIDTH).to_equal(1u32)?;

                let v = u1::new(1);
                expect!(v.value()).to_equal(1u8)?;
                expect!(u1::try_new(2).is_err()).to_be_true()?;
            }

            "u63 max width" {
                expect!(u63::WIDTH).to_equal(63u32)?;
                expect!(u63::MAX).to_equal((1u64 << 63) - 1)?;

                let v = u63::new(u63::MAX);
                expect!(v.value()).to_equal(u63::MAX)?;
            }

            "boundary values" {
                // Test max value for each backing type transition
                let v7 = u7::new(127);
                expect!(v7.value()).to_equal(127u8)?;

                let v9 = u9::new(511);
                expect!(v9.value()).to_equal(511u16)?;

                let v15 = u15::new(32767);
                expect!(v15.value()).to_equal(32767u16)?;

                let v17 = u17::new(131_071);
                expect!(v17.value()).to_equal(131_071u32)?;

                let v33 = u33::new((1u64 << 33) - 1);
                expect!(v33.value()).to_equal((1u64 << 33) - 1)?;
            }
        }

        "OutOfRange error" {
            "displays meaningful message" {
                let err = u3::try_new(8).unwrap_err();
                {
                    let msg = format!("{err}");
                    expect!(msg.as_str()).to_equal(
                        "value 8 exceeds 3-bit maximum of 7 for type u3"
                    )?;
                }
            }

            "debug format" {
                let err = u3::try_new(8).unwrap_err();
                {
                    let msg = format!("{err:?}");
                    expect!(msg.as_str()).to_equal(
                        "OutOfRange { type: u3, max: 7, actual: 8 }"
                    )?;
                }
            }
        }
    }

    "Error enum" {
        "TooShort displays context" {
            let err = Error::TooShort { needed_bytes: 6, have_bytes: 2 };
            {
                let msg = format!("{err}");
                expect!(msg.as_str()).to_equal(
                    "buffer too short: need 6 bytes, have 2"
                )?;
            }
        }

        "InvalidEnum displays context" {
            let err = Error::InvalidEnum { field: "priority", raw: 7 };
            {
                let msg = format!("{err}");
                expect!(msg.as_str()).to_equal(
                    "invalid enum value in field 'priority': raw value 7"
                )?;
            }
        }

        "TooShort is pattern matchable" {
            let err = Error::TooShort { needed_bytes: 6, have_bytes: 2 };
            {
                let matched = matches!(err, Error::TooShort { needed_bytes: 6, .. });
                expect!(matched).to_be_true()?;
            }
        }

        "errors are cloneable and comparable" {
            let err1 = Error::TooShort { needed_bytes: 6, have_bytes: 2 };
            let err2 = err1.clone();
            expect!(err1 == err2).to_be_true()?;
        }
    }

    "#[bitframe] proc-macro" {
        "generates a view type for CCSDS Primary Header" {
            {
                #[bitframe]
                pub struct CcsdsPrimaryHeader {
                    pub version: u3,
                    pub is_telecommand: bool,
                    pub has_secondary: bool,
                    pub apid: u11,
                    pub seq_flags: u2,
                    pub seq_count: u14,
                    pub pkt_len: u16,
                }

                // CCSDS example: version=0, type=0, sec=1, apid=42,
                // seq_flags=3, seq_count=1, pkt_len=100
                // Bits: 000 0 1 00000101010 11 00000000000001 0000000001100100
                // Byte 0: 000_0_1_000 = 0x08
                // Byte 1: 00101010 = 0x2A
                // Byte 2: 11_000000 = 0xC0
                // Byte 3: 00000001 = 0x01 (wait, seq_count is 14 bits)
                // Let me recalculate carefully:
                // version(3): 000
                // is_telecommand(1): 0
                // has_secondary(1): 1
                // apid(11): 00000101010
                // Total so far: 3+1+1+11 = 16 bits = 2 bytes
                // Byte 0: 000_0_1_000 = 0b00001000 = 0x08
                // Byte 1: 00101010 = 0b00101010 = 0x2A
                //
                // seq_flags(2): 11
                // seq_count(14): 00000000000001
                // Total: 2+14 = 16 bits = 2 bytes
                // Byte 2: 11_000000 = 0b11000000 = 0xC0
                // Byte 3: 00000001 = 0b00000001 = 0x01
                //
                // pkt_len(16): 0000000001100100 = 100
                // Byte 4: 0x00
                // Byte 5: 0x64
                let bytes: &[u8] = &[0x08, 0x2A, 0xC0, 0x01, 0x00, 0x64];
                let (header, rest) = CcsdsPrimaryHeaderRef::parse(bytes).unwrap();

                expect!(rest.len()).to_equal(0usize)?;
                expect!(header.version().value()).to_equal(0u8)?;
                expect!(header.is_telecommand()).to_be_false()?;
                expect!(header.has_secondary()).to_be_true()?;
                expect!(header.apid().value()).to_equal(42u16)?;
                expect!(header.seq_flags().value()).to_equal(3u8)?;
                expect!(header.seq_count().value()).to_equal(1u16)?;
                expect!(header.pkt_len()).to_equal(100u16)?;
            }
        }

        "SIZE constants are correct" {
            {
                #[bitframe]
                pub struct TestLayout {
                    pub a: u3,
                    pub b: bool,
                    pub c: u4,
                    pub d: u16,
                }

                expect!(TestLayoutRef::SIZE_BITS).to_equal(24usize)?;
                expect!(TestLayoutRef::SIZE_BYTES).to_equal(3usize)?;
            }
        }

        "parse rejects too-short buffer" {
            {
                #[bitframe]
                pub struct Small {
                    pub a: u8,
                    pub b: u8,
                }

                let bytes: &[u8] = &[0x42];
                let result = SmallRef::parse(bytes);
                expect!(result.is_err()).to_be_true()?;
            }
        }

        "parse_exact requires exact length" {
            {
                #[bitframe]
                pub struct Exact {
                    pub val: u8,
                }

                // Too short
                let result1 = ExactRef::parse_exact(&[]);
                expect!(result1.is_err()).to_be_true()?;

                // Exact
                let result2 = ExactRef::parse_exact(&[0x42]);
                expect!(result2.is_ok()).to_be_true()?;
                expect!(result2.unwrap().val()).to_equal(0x42u8)?;

                // Too long
                let result3 = ExactRef::parse_exact(&[0x42, 0x43]);
                expect!(result3.is_err()).to_be_true()?;
            }
        }

        "parse returns remainder" {
            {
                #[bitframe]
                pub struct OneByteLayout {
                    pub val: u8,
                }

                let bytes: &[u8] = &[0x42, 0x43, 0x44];
                let (view, rest) = OneByteLayoutRef::parse(bytes).unwrap();
                expect!(view.val()).to_equal(0x42u8)?;
                expect!(rest.len()).to_equal(2usize)?;
            }
        }

        "TryFrom works" {
            {
                #[bitframe]
                pub struct TryMe {
                    pub val: u16,
                }

                let bytes: &[u8] = &[0x00, 0x42, 0xFF];
                let view = TryMeRef::try_from(bytes).unwrap();
                expect!(view.val()).to_equal(0x0042u16)?;
            }
        }

        "as_bytes returns the underlying slice" {
            {
                #[bitframe]
                pub struct AsB {
                    pub a: u8,
                    pub b: u8,
                }

                let bytes: &[u8] = &[0xAB, 0xCD];
                let view = AsBRef::parse_exact(bytes).unwrap();
                let raw = view.as_bytes();
                expect!(raw.len()).to_equal(2usize)?;
                expect!(raw[0]).to_equal(0xABu8)?;
                expect!(raw[1]).to_equal(0xCDu8)?;
            }
        }

        "Debug formatting shows field values" {
            {
                #[bitframe]
                pub struct DebugTest {
                    pub flag: bool,
                    pub val: u7,
                }

                let bytes: &[u8] = &[0xFF]; // flag=1, val=127
                let view = DebugTestRef::parse_exact(bytes).unwrap();
                let s = format!("{view:?}");
                // Should contain field names and values
                expect!(s.contains("flag")).to_be_true()?;
                expect!(s.contains("val")).to_be_true()?;
            }
        }

        "BitLayout trait is implemented" {
            {
                #[bitframe]
                pub struct TraitTest {
                    pub a: u3,
                    pub b: u5,
                    pub c: u16,
                }

                fn check_layout<T: BitLayout>() -> (usize, usize) {
                    (T::SIZE_BITS, T::SIZE_BYTES)
                }

                let (bits, bytes) = check_layout::<TraitTest>();
                expect!(bits).to_equal(24usize)?;
                expect!(bytes).to_equal(3usize)?;
            }
        }

        "Parseable trait is implemented" {
            {
                #[bitframe]
                pub struct ParseableTest {
                    pub val: u16,
                }

                fn parse_generic<'a, T: Parseable<'a>>(
                    bytes: &'a [u8],
                ) -> Result<T::View, Error> {
                    let (view, _) = T::parse(bytes)?;
                    Ok(view)
                }

                let bytes: &[u8] = &[0x00, 0x42];
                let view = parse_generic::<ParseableTest>(bytes).unwrap();
                expect!(view.val()).to_equal(0x0042u16)?;
            }
        }

        "cross-byte field extraction" {
            {
                #[bitframe]
                pub struct CrossByte {
                    pub a: u4,
                    pub b: u12,
                    pub c: u8,
                }

                // a(4)=0xF, b(12)=0xABC, c(8)=0x42
                // Bits: 1111_1010_10111100_01000010
                // Byte 0: 0b11111010 = 0xFA
                // Byte 1: 0b10111100 = 0xBC
                // Byte 2: 0b01000010 = 0x42
                let bytes: &[u8] = &[0xFA, 0xBC, 0x42];
                let view = CrossByteRef::parse_exact(bytes).unwrap();
                expect!(view.a().value()).to_equal(0xFu8)?;
                expect!(view.b().value()).to_equal(0xABCu16)?;
                expect!(view.c()).to_equal(0x42u8)?;
            }
        }
    }

    "#[bitframe_enum] proc-macro" {
        "exhaustive enum — infallible from_raw" {
            {
                #[bitframe_enum]
                #[repr(u8)]
                pub enum SeqFlags {
                    Continuation = 0,
                    First = 1,
                    Last = 2,
                    Standalone = 3,
                }

                expect!(SeqFlags::WIDTH).to_equal(2usize)?;

                // Infallible — all 2^2 values covered
                let flags = SeqFlags::from_raw(u2::new(3));
                expect!(flags == SeqFlags::Standalone).to_be_true()?;

                // Round-trip
                let raw = SeqFlags::Standalone.to_raw();
                expect!(raw.value()).to_equal(3u8)?;
            }
        }

        "non-exhaustive enum — fallible from_raw" {
            {
                #[bitframe_enum(bits = 3)]
                #[repr(u8)]
                pub enum Priority {
                    Low = 0,
                    Medium = 1,
                    High = 2,
                }

                expect!(Priority::WIDTH).to_equal(3usize)?;

                // Valid value
                let p = Priority::from_raw(u3::new(1));
                expect!(p.is_ok()).to_be_true()?;
                expect!(p.unwrap() == Priority::Medium).to_be_true()?;

                // Invalid value
                let err = Priority::from_raw(u3::new(5));
                expect!(err.is_err()).to_be_true()?;
            }
        }

        "enum to_raw round-trips" {
            {
                #[bitframe_enum]
                #[repr(u8)]
                pub enum Direction {
                    North = 0,
                    East = 1,
                    South = 2,
                    West = 3,
                }

                let dir = Direction::East;
                let raw = dir.to_raw();
                let back = Direction::from_raw(raw);
                expect!(back == Direction::East).to_be_true()?;
            }
        }

        "enum width is inferred from discriminants" {
            {
                #[bitframe_enum]
                #[repr(u8)]
                pub enum SmallEnum {
                    A = 0,
                    B = 1,
                }

                // Max discriminant is 1, so width is 1 bit
                expect!(SmallEnum::WIDTH).to_equal(1usize)?;
            }
        }
    }

    "golden test: CCSDS Space Packet Primary Header" {
        // CCSDS 133.0-B-2 Space Packet Protocol
        // 48 bits / 6 bytes:
        //   version(3), type(1), sec_hdr_flag(1), apid(11),
        //   seq_flags(2), seq_count(14), data_length(16)

        "parses a known-good CCSDS packet header" {
            {
                #[bitframe]
                pub struct CcsdsHeader {
                    pub version: u3,
                    pub is_telecommand: bool,
                    pub has_secondary: bool,
                    pub apid: u11,
                    pub seq_flags: u2,
                    pub seq_count: u14,
                    pub data_length: u16,
                }

                expect!(CcsdsHeaderRef::SIZE_BITS).to_equal(48usize)?;
                expect!(CcsdsHeaderRef::SIZE_BYTES).to_equal(6usize)?;

                // Construct a known CCSDS header:
                //   version=0, type=1(TC), sec_hdr=1, apid=0x123,
                //   seq_flags=3(standalone), seq_count=42, data_length=255
                //
                // Bit layout (MSb first):
                //   000  1  1  00100100011
                //   = 0b00011001_00100011 = 0x19, 0x23
                //   11 00000000101010
                //   = 0b11000000_00101010 = 0xC0, 0x2A
                //   0000000011111111 = 0x00, 0xFF
                let bytes: &[u8] = &[0x19, 0x23, 0xC0, 0x2A, 0x00, 0xFF];
                let (hdr, rest) = CcsdsHeaderRef::parse(bytes).unwrap();

                expect!(rest.len()).to_equal(0usize)?;
                expect!(hdr.version().value()).to_equal(0u8)?;
                expect!(hdr.is_telecommand()).to_be_true()?;
                expect!(hdr.has_secondary()).to_be_true()?;
                expect!(hdr.apid().value()).to_equal(0x123u16)?;
                expect!(hdr.seq_flags().value()).to_equal(3u8)?;
                expect!(hdr.seq_count().value()).to_equal(42u16)?;
                expect!(hdr.data_length()).to_equal(255u16)?;
            }
        }

        "rejects truncated buffer" {
            {
                #[bitframe]
                pub struct CcsdsHdr2 {
                    pub version: u3,
                    pub is_telecommand: bool,
                    pub has_secondary: bool,
                    pub apid: u11,
                    pub seq_flags: u2,
                    pub seq_count: u14,
                    pub data_length: u16,
                }

                let short: &[u8] = &[0x08, 0x2A];
                let err = CcsdsHdr2Ref::parse(short).unwrap_err();
                {
                    let matched = matches!(
                        err,
                        Error::TooShort {
                            needed_bytes: 6,
                            have_bytes: 2,
                        }
                    );
                    expect!(matched).to_be_true()?;
                }
            }
        }

        "parse returns payload remainder" {
            {
                #[bitframe]
                pub struct CcsdsHdr3 {
                    pub version: u3,
                    pub is_telecommand: bool,
                    pub has_secondary: bool,
                    pub apid: u11,
                    pub seq_flags: u2,
                    pub seq_count: u14,
                    pub data_length: u16,
                }

                // 6-byte header + 3-byte payload
                let bytes: &[u8] = &[0x08, 0x2A, 0xC0, 0x01, 0x00, 0x64, 0xDE, 0xAD, 0xBE];
                let (hdr, payload) = CcsdsHdr3Ref::parse(bytes).unwrap();

                expect!(hdr.as_bytes().len()).to_equal(6usize)?;
                expect!(payload.len()).to_equal(3usize)?;
                expect!(payload[0]).to_equal(0xDEu8)?;
            }
        }
    }

    "golden test: CAN Standard Frame" {
        // Simplified CAN 2.0A standard frame header (32 bits / 4 bytes)
        //   id(11), rtr(1), ide(1), _reserved(1), dlc(4), _pad(14)

        "parses a known CAN frame" {
            {
                #[bitframe]
                pub struct CanStdFrame {
                    pub id: u11,
                    pub rtr: bool,
                    pub ide: bool,
                    pub _reserved: u1,
                    pub dlc: u4,
                    pub _pad: u14,
                }

                expect!(CanStdFrameRef::SIZE_BITS).to_equal(32usize)?;
                expect!(CanStdFrameRef::SIZE_BYTES).to_equal(4usize)?;

                // id=0x7FF (max), rtr=0, ide=0, reserved=0, dlc=8, pad=0
                // Bits: 11111111111 0 0 0 1000 00000000000000
                // Byte 0: 11111111 = 0xFF
                // Byte 1: 111_0_0_0_10 = 0b11100010 = 0xE2 (wait - 11 bits of id)
                // Let me be careful:
                // id(11): 11111111111
                // rtr(1): 0
                // ide(1): 0
                // reserved(1): 0
                // dlc(4): 1000
                // pad(14): 00000000000000
                //
                // Byte 0: bits[0..8] = 11111111 = 0xFF
                // Byte 1: bits[8..16] = 111_0_0_0_10 = 0b11100010 = 0xE2
                // Byte 2: bits[16..24] = 00_000000 = 0x00 (rest of dlc=00, pad starts)
                // Byte 3: bits[24..32] = 00000000 = 0x00
                let bytes: &[u8] = &[0xFF, 0xE2, 0x00, 0x00];
                let (frame, _) = CanStdFrameRef::parse(bytes).unwrap();

                expect!(frame.id().value()).to_equal(0x7FFu16)?;
                expect!(frame.rtr()).to_be_false()?;
                expect!(frame.ide()).to_be_false()?;
                expect!(frame.dlc().value()).to_equal(8u8)?;
            }
        }
    }

    "#[bitframe] advanced layouts" {
        "non-byte-aligned multi-byte fields (13-bit + 19-bit = 32 bits)" {
            {
                #[bitframe]
                pub struct Odd32 {
                    pub a: u13,
                    pub b: u19,
                }

                expect!(Odd32Ref::SIZE_BITS).to_equal(32usize)?;
                expect!(Odd32Ref::SIZE_BYTES).to_equal(4usize)?;

                // a(13)=0b1010101010101 = 5461
                // b(19)=0b111_00001111_00001111 = 462607
                // Combined 32 bits: 1010101010101_1110000111100001111
                // Byte 0: 10101010 = 0xAA
                // Byte 1: 10101_111 = 0b10101111 = 0xAF
                // Byte 2: 00001111 = 0x0F
                // Byte 3: 00001111 = 0x0F
                let bytes: &[u8] = &[0xAA, 0xAF, 0x0F, 0x0F];
                let (view, _) = Odd32Ref::parse(bytes).unwrap();

                expect!(view.a().value()).to_equal(5461u16)?;
                expect!(view.b().value()).to_equal(462_607u32)?;
            }
        }

        "u32 field type in bitframe struct" {
            {
                #[bitframe]
                pub struct Has32 {
                    pub val: u32,
                }

                let bytes: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF];
                let view = Has32Ref::parse_exact(bytes).unwrap();
                expect!(view.val()).to_equal(0xDEAD_BEEFu32)?;
            }
        }

        "u64 field type in bitframe struct" {
            {
                #[bitframe]
                pub struct Has64 {
                    pub val: u64,
                }

                let bytes: &[u8] = &[0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
                let view = Has64Ref::parse_exact(bytes).unwrap();
                expect!(view.val()).to_equal(0x0123_4567_89AB_CDEFu64)?;
            }
        }

        "AsRef<[u8]> on generated views" {
            {
                #[bitframe]
                pub struct AsRefTest {
                    pub a: u8,
                    pub b: u8,
                }

                let bytes: &[u8] = &[0xAB, 0xCD];
                let view = AsRefTestRef::parse_exact(bytes).unwrap();
                let slice: &[u8] = view.as_ref();
                expect!(slice.len()).to_equal(2usize)?;
                expect!(slice[0]).to_equal(0xABu8)?;
                expect!(slice[1]).to_equal(0xCDu8)?;
            }
        }

        "Copy semantics — cloned view is independent" {
            {
                #[bitframe]
                pub struct CopyTest {
                    pub val: u16,
                }

                let bytes1: &[u8] = &[0x00, 0x42];
                let bytes2: &[u8] = &[0x00, 0xFF];
                let view1 = CopyTestRef::parse_exact(bytes1).unwrap();
                let view2 = CopyTestRef::parse_exact(bytes2).unwrap();

                // Views point to different data
                expect!(view1.val()).to_equal(0x0042u16)?;
                expect!(view2.val()).to_equal(0x00FFu16)?;

                // Copy semantics
                let view1_copy = view1;
                expect!(view1_copy.val()).to_equal(view1.val())?;
            }
        }

        "error propagation with ? operator" {
            {
                #[bitframe]
                pub struct PropTest {
                    pub val: u16,
                }

                fn parse_prop(bytes: &[u8]) -> Result<u16, Error> {
                    let (view, _) = PropTestRef::parse(bytes)?;
                    Ok(view.val())
                }

                let ok_result = parse_prop(&[0x00, 0x42]);
                expect!(ok_result.is_ok()).to_be_true()?;
                expect!(ok_result.unwrap()).to_equal(0x0042u16)?;

                let err_result = parse_prop(&[0x00]);
                expect!(err_result.is_err()).to_be_true()?;
            }
        }
    }

    "reference vectors: CCSDS (spacepackets / KubOS)" {
        // Test vectors from published CCSDS implementations:
        // - spacepackets (us-irs): https://egit.irs.uni-stuttgart.de/rust/spacepackets
        // - KubOS ccsds-spacepacket: https://github.com/kubos/ccsds-spacepacket

        "PUS TC[17,1] Ping — spacepackets" {
            {
                #[bitframe]
                pub struct CcsdsVec {
                    pub version: u3,
                    pub is_telecommand: bool,
                    pub has_secondary: bool,
                    pub apid: u11,
                    pub seq_flags: u2,
                    pub seq_count: u14,
                    pub data_length: u16,
                }

                // TC[17,1] Connection Test from spacepackets test suite
                // version=0, type=1(TC), sec_hdr=1, apid=0x73(115)
                // seq_flags=3(standalone), seq_count=25, data_length=6
                let bytes: &[u8] = &[0x18, 0x73, 0xC0, 0x19, 0x00, 0x06];
                let (hdr, _) = CcsdsVecRef::parse(bytes).unwrap();

                expect!(hdr.version().value()).to_equal(0u8)?;
                expect!(hdr.is_telecommand()).to_be_true()?;
                expect!(hdr.has_secondary()).to_be_true()?;
                expect!(hdr.apid().value()).to_equal(0x73u16)?;
                expect!(hdr.seq_flags().value()).to_equal(3u8)?;
                expect!(hdr.seq_count().value()).to_equal(25u16)?;
                expect!(hdr.data_length()).to_equal(6u16)?;
            }
        }

        "PUS TM[17,2] Ping Reply — spacepackets" {
            {
                #[bitframe]
                pub struct CcsdsVec2 {
                    pub version: u3,
                    pub is_telecommand: bool,
                    pub has_secondary: bool,
                    pub apid: u11,
                    pub seq_flags: u2,
                    pub seq_count: u14,
                    pub data_length: u16,
                }

                // TM[17,2] Connection Test Reply from spacepackets test suite
                // version=0, type=0(TM), sec_hdr=1, apid=0x73(115)
                // seq_flags=3(standalone), seq_count=25, data_length=8
                let bytes: &[u8] = &[0x08, 0x73, 0xC0, 0x19, 0x00, 0x08];
                let (hdr, _) = CcsdsVec2Ref::parse(bytes).unwrap();

                expect!(hdr.version().value()).to_equal(0u8)?;
                expect!(hdr.is_telecommand()).to_be_false()?;
                expect!(hdr.has_secondary()).to_be_true()?;
                expect!(hdr.apid().value()).to_equal(0x73u16)?;
                expect!(hdr.seq_flags().value()).to_equal(3u8)?;
                expect!(hdr.seq_count().value()).to_equal(25u16)?;
                expect!(hdr.data_length()).to_equal(8u16)?;
            }
        }

        "Minimal TM idle packet — KubOS" {
            {
                #[bitframe]
                pub struct CcsdsVec3 {
                    pub version: u3,
                    pub is_telecommand: bool,
                    pub has_secondary: bool,
                    pub apid: u11,
                    pub seq_flags: u2,
                    pub seq_count: u14,
                    pub data_length: u16,
                }

                // Minimal TM packet from KubOS ccsds-spacepacket test suite
                // version=0, type=0(TM), sec_hdr=0, apid=0
                // seq_flags=3(standalone), seq_count=0, data_length=64
                let bytes: &[u8] = &[0x00, 0x00, 0xC0, 0x00, 0x00, 0x40];
                let (hdr, _) = CcsdsVec3Ref::parse(bytes).unwrap();

                expect!(hdr.version().value()).to_equal(0u8)?;
                expect!(hdr.is_telecommand()).to_be_false()?;
                expect!(hdr.has_secondary()).to_be_false()?;
                expect!(hdr.apid().value()).to_equal(0u16)?;
                expect!(hdr.seq_flags().value()).to_equal(3u8)?;
                expect!(hdr.seq_count().value()).to_equal(0u16)?;
                expect!(hdr.data_length()).to_equal(64u16)?;
            }
        }
    }

    "reference vectors: ADS-B (The 1090MHz Riddle / adsb_deku)" {
        // Test vectors from published ADS-B implementations:
        // - "The 1090 Megahertz Riddle" (Junzi Sun, TU Delft)
        // - adsb_deku: https://github.com/rsadsb/adsb_deku

        "DF=11 All-Call Reply — The 1090MHz Riddle" {
            {
                // ADS-B DF=11 (All-Call Reply): 56 bits / 7 bytes
                // Fields: df(5), ca(3), aa(24), pi(24)
                #[bitframe]
                pub struct AdsbDf11 {
                    pub df: u5,
                    pub ca: u3,
                    pub aa: u24,
                    pub pi: u24,
                }

                expect!(AdsbDf11Ref::SIZE_BITS).to_equal(56usize)?;

                // Raw: 5D 48 4F DE A2 48 F5
                // From "The 1090 Megahertz Riddle" by Junzi Sun (TU Delft)
                // df=11(0b01011), ca=5(0b101), aa=0x484FDE, pi=0xA248F5
                let bytes: &[u8] = &[0x5D, 0x48, 0x4F, 0xDE, 0xA2, 0x48, 0xF5];
                let (msg, _) = AdsbDf11Ref::parse(bytes).unwrap();

                expect!(msg.df().value()).to_equal(11u8)?;
                expect!(msg.ca().value()).to_equal(5u8)?;
                expect!(msg.aa().value()).to_equal(0x00_48_4F_DEu32)?;
                expect!(msg.pi().value()).to_equal(0x00_A2_48_F5u32)?;
            }
        }

        "DF=17 ADS-B KLM1023 — The 1090MHz Riddle" {
            {
                // ADS-B DF=17 (Extended Squitter): 112 bits / 14 bytes
                // Fields: df(5), ca(3), aa(24), me(56), pi(24)
                #[bitframe]
                pub struct AdsbDf17 {
                    pub df: u5,
                    pub ca: u3,
                    pub aa: u24,
                    pub me: u56,
                    pub pi: u24,
                }

                expect!(AdsbDf17Ref::SIZE_BITS).to_equal(112usize)?;

                // Raw: 8D 48 40 D6 20 2C C3 71 C3 2C E0 57 60 98
                // From "The 1090 Megahertz Riddle" — KLM1023
                // df=17(0b10001), ca=5(0b101), aa=0x4840D6 (ICAO address)
                let bytes: &[u8] = &[
                    0x8D, 0x48, 0x40, 0xD6, 0x20, 0x2C, 0xC3, 0x71,
                    0xC3, 0x2C, 0xE0, 0x57, 0x60, 0x98,
                ];
                let (msg, _) = AdsbDf17Ref::parse(bytes).unwrap();

                expect!(msg.df().value()).to_equal(17u8)?;
                expect!(msg.ca().value()).to_equal(5u8)?;
                expect!(msg.aa().value()).to_equal(0x00_48_40_D6u32)?;
            }
        }

        "DF=17 ADS-B — adsb_deku test vector" {
            {
                #[bitframe]
                pub struct AdsbDf17B {
                    pub df: u5,
                    pub ca: u3,
                    pub aa: u24,
                    pub me: u56,
                    pub pi: u24,
                }

                // Raw: 8D A2 C1 BD 58 7B A2 AD B3 17 99 CB 80 2B
                // From adsb_deku test suite
                // df=17, ca=5, aa=0xA2C1BD
                let bytes: &[u8] = &[
                    0x8D, 0xA2, 0xC1, 0xBD, 0x58, 0x7B, 0xA2, 0xAD,
                    0xB3, 0x17, 0x99, 0xCB, 0x80, 0x2B,
                ];
                let (msg, _) = AdsbDf17BRef::parse(bytes).unwrap();

                expect!(msg.df().value()).to_equal(17u8)?;
                expect!(msg.ca().value()).to_equal(5u8)?;
                expect!(msg.aa().value()).to_equal(0x00_A2_C1_BDu32)?;
            }
        }
    }

    "reference vectors: SocketCAN (LINKTYPE_CAN_SOCKETCAN)" {
        // SocketCAN LINKTYPE_CAN_SOCKETCAN pcap format (used by Wireshark, candump)
        // Header: 4 bytes CAN ID (with EFF/RTR/ERR flags in MSBs) + 4 bytes metadata
        // Reference: https://www.tcpdump.org/linktypes/LINKTYPE_CAN_SOCKETCAN.html

        "OBD-II RPM Request (Standard Frame, ID=0x7DF)" {
            {
                // SocketCAN pcap header: 64 bits / 8 bytes
                // CAN ID field (32 bits, big-endian in frame but LE on wire — we
                // store the big-endian view here for bitframe):
                //   bit 0: EFF flag (0=standard, 1=extended)
                //   bit 1: RTR flag
                //   bit 2: ERR flag
                //   bits 3..32: CAN ID (29-bit space, only 11 used for standard)
                #[bitframe]
                pub struct SocketCanHeader {
                    pub eff: bool,
                    pub rtr: bool,
                    pub err: bool,
                    pub can_id: u29,
                    pub len: u8,
                    pub _fd_flags: u8,
                    pub _reserved: u8,
                    pub _len8_dlc: u8,
                }

                expect!(SocketCanHeaderRef::SIZE_BITS).to_equal(64usize)?;

                // OBD-II broadcast request: ID=0x7DF, standard frame, 8 data bytes
                // Big-endian ID field: 0_0_0_00000000000000000011111011111
                // = 0b00000000_00000000_00000111_11011111
                // Byte 0: 0x00, Byte 1: 0x00, Byte 2: 0x07, Byte 3: 0xDF
                // len=8, fd_flags=0, reserved=0, len8_dlc=0
                let bytes: &[u8] = &[0x00, 0x00, 0x07, 0xDF, 0x08, 0x00, 0x00, 0x00];
                let (hdr, _) = SocketCanHeaderRef::parse(bytes).unwrap();

                expect!(hdr.eff()).to_be_false()?;
                expect!(hdr.rtr()).to_be_false()?;
                expect!(hdr.err()).to_be_false()?;
                expect!(hdr.can_id().value()).to_equal(0x7DFu32)?;
                expect!(hdr.len()).to_equal(8u8)?;
            }
        }

        "J1939 EEC1 Engine Speed (Extended Frame, ID=0x0CF00401)" {
            {
                #[bitframe]
                pub struct SocketCanHeader2 {
                    pub eff: bool,
                    pub rtr: bool,
                    pub err: bool,
                    pub can_id: u29,
                    pub len: u8,
                    pub _fd_flags: u8,
                    pub _reserved: u8,
                    pub _len8_dlc: u8,
                }

                // J1939 EEC1 (Electronic Engine Controller 1): PGN 0xF004
                // Extended frame ID = 0x0CF00401
                // EFF=1, RTR=0, ERR=0
                // Big-endian ID field: 1_0_0_0_1100_11110000_00000100_00000001
                // = 0b10001100_11110000_00000100_00000001
                // Byte 0: 0x8C, Byte 1: 0xF0, Byte 2: 0x04, Byte 3: 0x01
                let bytes: &[u8] = &[0x8C, 0xF0, 0x04, 0x01, 0x08, 0x00, 0x00, 0x00];
                let (hdr, _) = SocketCanHeader2Ref::parse(bytes).unwrap();

                expect!(hdr.eff()).to_be_true()?;
                expect!(hdr.rtr()).to_be_false()?;
                expect!(hdr.err()).to_be_false()?;
                expect!(hdr.can_id().value()).to_equal(0x0CF0_0401u32)?;
                expect!(hdr.len()).to_equal(8u8)?;
            }
        }
    }

    "golden test: ADS-B Short Message (DF=11)" {
        // Simplified ADS-B Mode S short message
        // 56 bits / 7 bytes:
        //   df(5), vs(1), cc(1), _unused(1), sl(3), _spare(2), ri(4), _spare2(2),
        //   altitude(13), _parity(24)

        "parses a known ADS-B message" {
            {
                #[bitframe]
                pub struct AdsbShort {
                    pub df: u5,
                    pub vs: bool,
                    pub cc: bool,
                    pub _unused: u1,
                    pub sl: u3,
                    pub _spare: u2,
                    pub ri: u4,
                    pub _spare2: u2,
                    pub altitude: u13,
                    pub _parity: u24,
                }

                expect!(AdsbShortRef::SIZE_BITS).to_equal(56usize)?;
                expect!(AdsbShortRef::SIZE_BYTES).to_equal(7usize)?;

                // df=11(0b01011), vs=0, cc=1, unused=0,
                // sl=5(0b101), spare=0, ri=3(0b0011), spare2=0,
                // altitude=1234(0b0010011010010), parity=0xABCDEF
                //
                // Bits (56 total):
                // 01011 0 1 0 101 00 0011 00 0010011010010 101010111100110111101111
                //
                // Byte 0: 01011_0_1_0 = 0b01011010 = 0x5A
                // Byte 1: 101_00_001 = 0b10100001 = 0xA1 (wait, let me redo)
                // sl(3)=101, spare(2)=00, ri starts
                // Byte 1: 101_00_011 = 0b10100011 = 0xA3
                // Wait no. Let me be precise:
                // bit  0..5:  df=01011
                // bit  5:    vs=0
                // bit  6:    cc=1
                // bit  7:    unused=0
                // bit  8..11: sl=101
                // bit 11..13: spare=00
                // bit 13..17: ri=0011
                // bit 17..19: spare2=00
                // bit 19..32: altitude=0010011010010
                // bit 32..56: parity=101010111100110111101111
                //
                // Byte 0 [0..8]:  01011_0_1_0 = 0x5A
                // Byte 1 [8..16]: 101_00_001 = 0b10100001 ... wait
                // bit 8=sl[0]=1, bit 9=sl[1]=0, bit 10=sl[2]=1
                // bit 11=spare[0]=0, bit 12=spare[1]=0
                // bit 13=ri[0]=0, bit 14=ri[1]=0, bit 15=ri[2]=1
                // Byte 1: 1_0_1_0_0_0_0_1 = 0b10100001 = 0xA1
                // Hmm wait ri=3=0b0011, so ri[0]=0,ri[1]=0,ri[2]=1,ri[3]=1
                // bit 13=0, bit 14=0, bit 15=1
                // Byte 1: 10100001 = 0xA1
                // bit 16=ri[3]=1
                // bit 17=spare2[0]=0, bit 18=spare2[1]=0
                // bit 19..32: altitude=0010011010010
                // Byte 2 [16..24]: 1_0_0_00100 = 0b10000100 = 0x84
                // Byte 3 [24..32]: 11010010 = 0xD2
                // Byte 4..6: parity=0xABCDEF
                let bytes: &[u8] = &[0x5A, 0xA1, 0x84, 0xD2, 0xAB, 0xCD, 0xEF];
                let (msg, _) = AdsbShortRef::parse(bytes).unwrap();

                expect!(msg.df().value()).to_equal(11u8)?;
                expect!(msg.vs()).to_be_false()?;
                expect!(msg.cc()).to_be_true()?;
                expect!(msg.sl().value()).to_equal(5u8)?;
                expect!(msg.ri().value()).to_equal(3u8)?;
                expect!(msg.altitude().value()).to_equal(1234u16)?;
                expect!(msg._parity().value()).to_equal(0x00AB_CDEFu32)?;
            }
        }
    }
}

#[cfg(feature = "std")]
#[test]
fn error_implements_std_error() {
    fn is_std_error<T: std::error::Error>(_: &T) -> bool {
        true
    }
    let err = Error::TooShort {
        needed_bytes: 6,
        have_bytes: 2,
    };
    assert!(is_std_error(&err));
}
