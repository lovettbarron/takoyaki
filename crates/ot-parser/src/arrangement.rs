// Format: see crates/ot-parser/format-spec.md "arrNN.work / arrNN.strd (ArrangementFile)"
// All field offsets and unknown region sizes derived from clean-room format spec.
//
// ArrangementFile layout:
//   Offset  Size  Field
//   0x00    21    header (magic: "FORM\0\0\0\0DPS1ARRA\0\0\0\0\0")
//   0x15    1     datatype_version (6)
//   0x16    N     opaque_body (two ArrangementBlock + state data — size unknown, D-02)
//   last-2  2     checksum (u16 BE, stored verbatim)
//
// The body between header+version and checksum is treated as an opaque blob because
// the exact binary size of ArrangeRow (especially ReminderRow with variable String content)
// and ArrangementBlock cannot be independently verified from docs alone.
// Per D-02, all unknown regions are preserved verbatim.

/// Total size of the ArrangementFile header in bytes.
pub const ARR_HEADER_SIZE: usize = 21;

/// Total size of the datatype_version field in bytes.
pub const ARR_VERSION_SIZE: usize = 1;

/// Total size of the checksum field in bytes.
pub const ARR_CHECKSUM_SIZE: usize = 2;

/// Minimum file size: header + version + checksum (no body).
pub const ARR_MIN_SIZE: usize = ARR_HEADER_SIZE + ARR_VERSION_SIZE + ARR_CHECKSUM_SIZE;

/// ArrangementFile header magic bytes (21 bytes).
/// "FORM" + 4×0x00 + "DPS1" + "ARRA" + 5×0x00
pub const ARR_HEADER_MAGIC: [u8; 21] = [
    0x46, 0x4F, 0x52, 0x4D, // "FORM"
    0x00, 0x00, 0x00, 0x00, // 4 null bytes
    0x44, 0x50, 0x53, 0x31, // "DPS1"
    0x41, 0x52, 0x52, 0x41, // "ARRA"
    0x00, 0x00, 0x00, 0x00, 0x00, // 5 null bytes
];

/// Expected file version byte for OT OS 1.40B.
pub const ARR_FILE_VERSION: u8 = 6;

/// Parsed representation of an Octatrack arrangement file (arrNN.work or arrNN.strd).
///
/// Contains two arrangement blocks (current + previous saved state) for one of the
/// 8 arrangement slots in a project. The body is stored as an opaque blob per D-02
/// because the binary size of ArrangementBlock (especially ArrangeRow with its
/// variable-length ReminderRow variant) cannot be determined from public docs alone.
///
/// Layout: header[21] + version(u8) + opaque_body(Vec<u8>) + checksum(u16 BE)
///
/// Round-trip is byte-exact because all data is preserved verbatim.
///
/// # Size Validation
/// - Minimum: ARR_MIN_SIZE bytes
/// - Header magic must match ARR_HEADER_MAGIC (first 4 bytes: "FORM")
#[derive(Debug, Clone, PartialEq)]
pub struct ArrangementFile {
    /// 21-byte header magic identifying this as an arrangement file.
    pub header: [u8; 21],
    /// Data type version. Expected value: 6 for OS 1.40B.
    pub datatype_version: u8,
    /// Opaque body: arrangement blocks and state data, preserved verbatim (D-02).
    /// Size = total_file_size - ARR_HEADER_SIZE - ARR_VERSION_SIZE - ARR_CHECKSUM_SIZE.
    pub opaque_body: Vec<u8>,
    /// Checksum (u16, big-endian). Stored verbatim for round-trip fidelity.
    /// Non-trivial to recalculate; see format-spec.md Checksum Algorithm section.
    pub checksum: u16,
}

impl ArrangementFile {
    /// Parse an `ArrangementFile` from a byte slice.
    ///
    /// # Errors
    /// - `ParseError::UnexpectedSize` if slice is shorter than ARR_MIN_SIZE bytes.
    /// - `ParseError::InvalidMagic` if the first 4 bytes are not "FORM".
    pub fn from_bytes(data: &[u8]) -> crate::Result<Self> {
        if data.len() < ARR_MIN_SIZE {
            return Err(crate::ParseError::UnexpectedSize {
                expected: ARR_MIN_SIZE,
                actual: data.len(),
            });
        }
        if &data[0..4] != &ARR_HEADER_MAGIC[0..4] {
            return Err(crate::ParseError::InvalidMagic);
        }

        let header: [u8; 21] = data[0..21].try_into().unwrap();
        let datatype_version = data[21];

        // Body is everything between version and the last 2 bytes (checksum)
        let body_start = ARR_HEADER_SIZE + ARR_VERSION_SIZE;
        let body_end = data.len() - ARR_CHECKSUM_SIZE;
        let opaque_body = data[body_start..body_end].to_vec();

        // Checksum: last 2 bytes, big-endian u16
        let checksum = u16::from_be_bytes([data[data.len() - 2], data[data.len() - 1]]);

        Ok(ArrangementFile {
            header,
            datatype_version,
            opaque_body,
            checksum,
        })
    }

    /// Serialize an `ArrangementFile` to bytes.
    ///
    /// Produces byte-exact output matching the original file because the opaque body
    /// and checksum are stored verbatim.
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        let mut out = Vec::with_capacity(
            ARR_HEADER_SIZE + ARR_VERSION_SIZE + self.opaque_body.len() + ARR_CHECKSUM_SIZE,
        );
        out.extend_from_slice(&self.header);
        out.push(self.datatype_version);
        out.extend_from_slice(&self.opaque_body);
        out.extend_from_slice(&self.checksum.to_be_bytes());
        Ok(out)
    }

    /// Return the total file size in bytes.
    pub fn file_size(&self) -> usize {
        ARR_HEADER_SIZE + ARR_VERSION_SIZE + self.opaque_body.len() + ARR_CHECKSUM_SIZE
    }
}
