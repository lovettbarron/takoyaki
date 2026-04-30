// Format: see crates/ot-parser/format-spec.md "project.work / project.strd (ProjectFile)"
// All field offsets and unknown region sizes derived from clean-room format spec.
//
// IMPORTANT: project.work and project.strd are TEXT-BASED files (key=value pairs),
// NOT fixed-layout binary files. They cannot be parsed with binrw. Instead, the
// entire file is stored verbatim as raw bytes. This provides:
//   - Byte-exact round-trip fidelity (required by D-02)
//   - Opaque preservation of all text content (required by D-02)
//   - Safe handling regardless of OS version or field additions
//
// Source: ot-tools-io docs — "project files are actually string data being parsed
// directly without any serde-ing or bincode-ing"

/// Parsed representation of an Octatrack project file (project.work or project.strd).
///
/// The OT project file is text-based (key=value sections), not a fixed-layout binary
/// format. All bytes are stored verbatim to guarantee byte-exact round-trip fidelity
/// and preserve unknown/future fields per design decision D-02.
///
/// The `raw` field contains the complete file content and is preserved through
/// `from_bytes` → `to_bytes` without modification.
///
/// # Size Validation
/// - Minimum: 1 byte (non-empty)
/// - Maximum: None (text files have variable length)
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectFile {
    /// Raw file bytes — the complete project.work / project.strd content verbatim.
    /// All text content (metadata, settings, states, slots) is preserved here.
    pub raw: Vec<u8>,
}

impl ProjectFile {
    /// Parse a `ProjectFile` from a byte slice.
    ///
    /// The entire byte slice is stored verbatim in the `raw` field.
    /// Returns `ParseError::UnexpectedSize` if the slice is empty.
    pub fn from_bytes(data: &[u8]) -> crate::Result<Self> {
        if data.is_empty() {
            return Err(crate::ParseError::UnexpectedSize {
                expected: 1,
                actual: 0,
            });
        }
        Ok(ProjectFile {
            raw: data.to_vec(),
        })
    }

    /// Serialize a `ProjectFile` to bytes.
    ///
    /// Returns the raw bytes verbatim — same bytes that were passed to `from_bytes`.
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        Ok(self.raw.clone())
    }

    /// Return the file size in bytes.
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Return true if the file has no content.
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }
}
