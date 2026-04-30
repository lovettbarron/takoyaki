use binrw::{binrw, BinRead, BinWrite};
use std::io::Cursor;

/// Total size of an .ot sidecar file in bytes.
pub const OT_FILE_SIZE: usize = 832;

/// First 4 bytes of the .ot file header magic.
/// Full header magic: [0xF0, 0x00, 0x00, 0xE8, 0x57, 0x45, 0x52, 0x41, 0x00*8]
pub const OT_HEADER_MAGIC: [u8; 4] = [0xF0, 0x00, 0x00, 0xE8];

/// Octatrack .ot sidecar file — 832 bytes total.
///
/// Format verified from OctaChainer `otwriter.h`. All multi-byte fields are
/// big-endian. The 7-byte unknown region at offset 0x10 is preserved verbatim
/// during round-trip (per design decision D-02).
///
/// Layout:
/// ```text
/// Offset  Size  Type        Field
/// 0x00    16    u8[16]      Header/magic
/// 0x10    7     u8[7]       Unknown (preserve verbatim)
/// 0x17    4     u32 BE      Tempo
/// 0x1B    4     u32 BE      Trim length (samples)
/// 0x1F    4     u32 BE      Loop length (samples)
/// 0x23    4     u32 BE      Stretch
/// 0x27    4     u32 BE      Loop flag
/// 0x2B    2     u16 BE      Gain
/// 0x2D    1     u8          Quantize
/// 0x2E    4     u32 BE      Trim start (samples)
/// 0x32    4     u32 BE      Trim end (samples)
/// 0x36    4     u32 BE      Loop point (samples)
/// 0x3A    768   Slice[64]   Slice data (12 bytes each)
/// 0x33A   4     u32 BE      Slice count
/// 0x33E   2     u16 BE      Checksum
/// TOTAL = 832 bytes
/// ```
#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct SampleSettingsFile {
    pub header: [u8; 16],
    /// Undocumented 7-byte region at offset 0x10 — preserved verbatim (D-02).
    pub unknown_0x10: [u8; 7],
    pub tempo: u32,
    pub trim_len: u32,
    pub loop_len: u32,
    pub stretch: u32,
    pub loop_flag: u32,
    pub gain: u16,
    pub quantize: u8,
    pub trim_start: u32,
    pub trim_end: u32,
    pub loop_point: u32,
    #[br(count = 64)]
    pub slices: Vec<Slice>,
    pub slice_count: u32,
    pub checksum: u16,
}

/// A single slice entry within an .ot file.
///
/// Each slice is 12 bytes: start_point, end_point, and loop_point as u32 BE.
#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct Slice {
    pub start_point: u32,
    pub end_point: u32,
    pub loop_point: u32,
}

impl SampleSettingsFile {
    /// Parse a `SampleSettingsFile` from a byte slice.
    ///
    /// Returns `ParseError::UnexpectedSize` if the slice is not exactly 832 bytes.
    /// Returns `ParseError::InvalidMagic` if the first 4 bytes do not match the OT magic.
    /// Returns `ParseError::Parse` if binrw parsing fails.
    pub fn from_bytes(data: &[u8]) -> crate::Result<Self> {
        if data.len() != OT_FILE_SIZE {
            return Err(crate::ParseError::UnexpectedSize {
                expected: OT_FILE_SIZE,
                actual: data.len(),
            });
        }
        if data[0..4] != OT_HEADER_MAGIC {
            return Err(crate::ParseError::InvalidMagic);
        }
        let mut cursor = Cursor::new(data);
        Self::read(&mut cursor).map_err(|e| crate::ParseError::Parse(e.to_string()))
    }

    /// Serialize a `SampleSettingsFile` to bytes.
    ///
    /// The result is always exactly 832 bytes for a valid struct.
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        let mut cursor = Cursor::new(Vec::with_capacity(OT_FILE_SIZE));
        self.write(&mut cursor)
            .map_err(|e| crate::ParseError::Parse(e.to_string()))?;
        Ok(cursor.into_inner())
    }
}
