// Format: see crates/ot-parser/format-spec.md "markers.work / markers.strd (MarkersFile)"
// All field offsets and unknown region sizes derived from clean-room format spec.
//
// MarkersFile layout:
//   Offset    Size     Field
//   0x00      21       header (magic: "FORM\0\0\0\0DPS1SAMP\0\0\0\0\0")
//   0x15      1        datatype_version (4)
//   0x16      N        opaque_body (SlotMarkers × 264 — size = 264 × 784 bytes = 206,976)
//   last-2    2        checksum (u16 BE, stored verbatim)
//
// MarkersFile is fully characterized in the format spec (SlotMarkers = 784 bytes,
// 136 flex + 128 static = 264 slots, total = 207,000 bytes). However, since the
// checksum algorithm is non-trivial (requires bincode + default instance comparison),
// the body is stored as an opaque blob and the checksum is stored verbatim per D-02.
// This guarantees byte-exact round-trip without implementing the checksum algorithm.

/// Total size of the MarkersFile header in bytes.
pub const MARKERS_HEADER_SIZE: usize = 21;

/// Total size of the datatype_version field in bytes.
pub const MARKERS_VERSION_SIZE: usize = 1;

/// Total size of the checksum field in bytes.
pub const MARKERS_CHECKSUM_SIZE: usize = 2;

/// Minimum file size: header + version + checksum (no body).
pub const MARKERS_MIN_SIZE: usize =
    MARKERS_HEADER_SIZE + MARKERS_VERSION_SIZE + MARKERS_CHECKSUM_SIZE;

/// Expected total file size for a real OT markers.work:
/// 21 + 1 + (136 + 128) × 784 + 2 = 207,000 bytes
pub const MARKERS_FULL_SIZE: usize = 207_000;

/// MarkersFile header magic bytes (21 bytes).
/// "FORM" + 4×0x00 + "DPS1" + "SAMP" + 5×0x00
pub const MARKERS_HEADER_MAGIC: [u8; 21] = [
    0x46, 0x4F, 0x52, 0x4D, // "FORM"
    0x00, 0x00, 0x00, 0x00, // 4 null bytes
    0x44, 0x50, 0x53, 0x31, // "DPS1"
    0x53, 0x41, 0x4D, 0x50, // "SAMP"
    0x00, 0x00, 0x00, 0x00, 0x00, // 5 null bytes
];

/// Expected file version byte.
pub const MARKERS_FILE_VERSION: u8 = 4;

/// Parsed representation of an Octatrack markers file (markers.work or markers.strd).
///
/// Contains sample editor data (trim, loop, slice markers) for all slots in a project.
/// 136 flex slots + 128 static slots = 264 SlotMarkers × 784 bytes = 206,976 bytes of body.
///
/// The body is stored as an opaque blob and the checksum verbatim per D-02, ensuring
/// byte-exact round-trip fidelity without implementing the non-trivial checksum algorithm.
///
/// Layout: header[21] + version(u8) + opaque_body(Vec<u8>) + checksum(u16 BE)
///
/// # Size Validation
/// - Minimum: MARKERS_MIN_SIZE bytes
/// - Header magic must match MARKERS_HEADER_MAGIC (first 4 bytes: "FORM")
#[derive(Debug, Clone, PartialEq)]
pub struct MarkersFile {
    /// 21-byte header magic identifying this as a markers file.
    pub header: [u8; 21],
    /// Data type version. Expected value: 4.
    pub datatype_version: u8,
    /// Opaque body: all SlotMarkers data (264 slots × 784 bytes), preserved verbatim (D-02).
    /// Size = total_file_size - MARKERS_HEADER_SIZE - MARKERS_VERSION_SIZE - MARKERS_CHECKSUM_SIZE.
    pub opaque_body: Vec<u8>,
    /// Checksum (u16, big-endian). Stored verbatim for round-trip fidelity.
    /// Non-trivial to recalculate; see format-spec.md Checksum Algorithm section.
    pub checksum: u16,
}

impl MarkersFile {
    /// Parse a `MarkersFile` from a byte slice.
    ///
    /// # Errors
    /// - `ParseError::UnexpectedSize` if slice is shorter than MARKERS_MIN_SIZE bytes.
    /// - `ParseError::InvalidMagic` if the first 4 bytes are not "FORM".
    pub fn from_bytes(data: &[u8]) -> crate::Result<Self> {
        if data.len() < MARKERS_MIN_SIZE {
            return Err(crate::ParseError::UnexpectedSize {
                expected: MARKERS_MIN_SIZE,
                actual: data.len(),
            });
        }
        if &data[0..4] != &MARKERS_HEADER_MAGIC[0..4] {
            return Err(crate::ParseError::InvalidMagic);
        }

        let header: [u8; 21] = data[0..21].try_into().unwrap();
        let datatype_version = data[21];

        // Body is everything between version and the last 2 bytes (checksum)
        let body_start = MARKERS_HEADER_SIZE + MARKERS_VERSION_SIZE;
        let body_end = data.len() - MARKERS_CHECKSUM_SIZE;
        let opaque_body = data[body_start..body_end].to_vec();

        // Checksum: last 2 bytes, big-endian u16
        let checksum = u16::from_be_bytes([data[data.len() - 2], data[data.len() - 1]]);

        Ok(MarkersFile {
            header,
            datatype_version,
            opaque_body,
            checksum,
        })
    }

    /// Serialize a `MarkersFile` to bytes.
    ///
    /// Produces byte-exact output matching the original file because the opaque body
    /// and checksum are stored verbatim.
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        let mut out = Vec::with_capacity(
            MARKERS_HEADER_SIZE
                + MARKERS_VERSION_SIZE
                + self.opaque_body.len()
                + MARKERS_CHECKSUM_SIZE,
        );
        out.extend_from_slice(&self.header);
        out.push(self.datatype_version);
        out.extend_from_slice(&self.opaque_body);
        out.extend_from_slice(&self.checksum.to_be_bytes());
        Ok(out)
    }

    /// Return the total file size in bytes.
    pub fn file_size(&self) -> usize {
        MARKERS_HEADER_SIZE + MARKERS_VERSION_SIZE + self.opaque_body.len() + MARKERS_CHECKSUM_SIZE
    }
}
