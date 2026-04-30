// Format: see crates/ot-parser/format-spec.md "bankNN.work / bankNN.strd (BankFile)"
// All field offsets and unknown region sizes derived from clean-room format spec.
//
// BankFile layout:
//   Offset  Size  Field
//   0x00    21    header (magic bytes: "FORM\0\0\0\0DPS1BANK\0\0\0\0\0")
//   0x15    1     datatype_version (23 = 0x17 for OT OS 1.40B)
//   0x16    N     opaque_body (all pattern/part/state data — size unknown, D-02)
//   last-2  2     checksum (u16 BE, stored verbatim for round-trip fidelity)
//
// The body between header+version and checksum is treated as an opaque blob
// because the exact sizes of Pattern, Part, and their sub-structures cannot be
// independently verified without running ot-tools-io against real OT files.
// Per D-02, all unknown regions are preserved verbatim.


/// Total size of the BankFile header in bytes.
pub const BANK_HEADER_SIZE: usize = 21;

/// Total size of the datatype_version field in bytes.
pub const BANK_VERSION_SIZE: usize = 1;

/// Total size of the checksum field in bytes.
pub const BANK_CHECKSUM_SIZE: usize = 2;

/// Minimum file size: header + version + checksum (no body).
pub const BANK_MIN_SIZE: usize = BANK_HEADER_SIZE + BANK_VERSION_SIZE + BANK_CHECKSUM_SIZE;

/// BankFile header magic bytes (21 bytes).
/// "FORM" + 4×0x00 + "DPS1" + "BANK" + 5×0x00
pub const BANK_HEADER_MAGIC: [u8; 21] = [
    0x46, 0x4F, 0x52, 0x4D, // "FORM"
    0x00, 0x00, 0x00, 0x00, // 4 null bytes
    0x44, 0x50, 0x53, 0x31, // "DPS1"
    0x42, 0x41, 0x4E, 0x4B, // "BANK"
    0x00, 0x00, 0x00, 0x00, 0x00, // 5 null bytes
];

/// Expected file version byte for OT OS 1.40B.
pub const BANK_FILE_VERSION: u8 = 23;

/// Parsed representation of an Octatrack bank file (bankNN.work or bankNN.strd).
///
/// The bank file contains pattern and part data for one of the 16 banks in an
/// OT project. The body between the header+version and the checksum is stored as
/// an opaque blob (per D-02) because exact sub-structure sizes are not independently
/// verifiable from public documentation alone.
///
/// Layout: header[21] + version(u8) + opaque_body(Vec<u8>) + checksum(u16 BE)
///
/// Round-trip is byte-exact because all unknown data is preserved verbatim.
///
/// # Size Validation
/// - Minimum: BANK_MIN_SIZE bytes (header + version + empty body + checksum)
/// - Header magic must match BANK_HEADER_MAGIC (first 4 bytes: "FORM")
#[derive(Debug, Clone, PartialEq)]
pub struct BankFile {
    /// 21-byte header magic identifying this as a bank file.
    pub header: [u8; 21],
    /// Data type version. Expected value: 23 (0x17) for OS 1.40B.
    pub datatype_version: u8,
    /// Opaque body: all pattern and part data, preserved verbatim (D-02).
    /// Size = total_file_size - BANK_HEADER_SIZE - BANK_VERSION_SIZE - BANK_CHECKSUM_SIZE.
    pub opaque_body: Vec<u8>,
    /// Checksum (u16, big-endian). Stored verbatim for round-trip fidelity.
    /// Non-trivial to recalculate; see format-spec.md Checksum Algorithm section.
    pub checksum: u16,
}

impl BankFile {
    /// Parse a `BankFile` from a byte slice.
    ///
    /// # Errors
    /// - `ParseError::UnexpectedSize` if slice is shorter than BANK_MIN_SIZE bytes.
    /// - `ParseError::InvalidMagic` if the first 4 bytes are not "FORM" (0x46,0x4F,0x52,0x4D).
    pub fn from_bytes(data: &[u8]) -> crate::Result<Self> {
        if data.len() < BANK_MIN_SIZE {
            return Err(crate::ParseError::UnexpectedSize {
                expected: BANK_MIN_SIZE,
                actual: data.len(),
            });
        }
        if &data[0..4] != &BANK_HEADER_MAGIC[0..4] {
            return Err(crate::ParseError::InvalidMagic);
        }

        let header: [u8; 21] = data[0..21].try_into().unwrap();
        let datatype_version = data[21];

        // Body is everything between version and the last 2 bytes (checksum)
        let body_end = data.len() - BANK_CHECKSUM_SIZE;
        let opaque_body = data[BANK_HEADER_SIZE + BANK_VERSION_SIZE..body_end].to_vec();

        // Checksum: last 2 bytes, big-endian u16
        let checksum = u16::from_be_bytes([data[data.len() - 2], data[data.len() - 1]]);

        Ok(BankFile {
            header,
            datatype_version,
            opaque_body,
            checksum,
        })
    }

    /// Serialize a `BankFile` to bytes.
    ///
    /// Produces byte-exact output matching the original file because the opaque body
    /// and checksum are stored verbatim.
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        let mut out = Vec::with_capacity(
            BANK_HEADER_SIZE + BANK_VERSION_SIZE + self.opaque_body.len() + BANK_CHECKSUM_SIZE,
        );
        out.extend_from_slice(&self.header);
        out.push(self.datatype_version);
        out.extend_from_slice(&self.opaque_body);
        out.extend_from_slice(&self.checksum.to_be_bytes());
        Ok(out)
    }

    /// Return the total file size in bytes.
    pub fn file_size(&self) -> usize {
        BANK_HEADER_SIZE + BANK_VERSION_SIZE + self.opaque_body.len() + BANK_CHECKSUM_SIZE
    }
}
