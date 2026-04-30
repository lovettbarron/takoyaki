use ot_parser::sample::SampleSettingsFile;

#[test]
fn test_sample_parse() {
    let bytes = include_bytes!("../../../tests/fixtures/sample.ot");
    assert_eq!(bytes.len(), 832);
    let parsed = SampleSettingsFile::from_bytes(bytes).unwrap();
    assert_eq!(&parsed.header[0..4], &[0xF0, 0x00, 0x00, 0xE8]);
    assert_eq!(parsed.slices.len(), 64);
}

#[test]
fn test_sample_round_trip() {
    let bytes = include_bytes!("../../../tests/fixtures/sample.ot");
    let parsed = SampleSettingsFile::from_bytes(bytes).unwrap();
    let rewritten = parsed.to_bytes().unwrap();
    assert_eq!(bytes.as_ref(), rewritten.as_slice(), "Byte-exact round-trip failed");
}

#[test]
fn test_sample_round_trip_parse_equality() {
    let bytes = include_bytes!("../../../tests/fixtures/sample.ot");
    let parsed1 = SampleSettingsFile::from_bytes(bytes).unwrap();
    let rewritten = parsed1.to_bytes().unwrap();
    let parsed2 = SampleSettingsFile::from_bytes(&rewritten).unwrap();
    assert_eq!(parsed1, parsed2, "parse(serialize(parse(bytes))) != parse(bytes)");
}

#[test]
fn test_sample_wrong_size() {
    let bytes = vec![0u8; 831]; // Too short
    let result = SampleSettingsFile::from_bytes(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_sample_wrong_magic() {
    let mut bytes = vec![0u8; 832];
    bytes[0] = 0xFF; // Wrong magic
    let result = SampleSettingsFile::from_bytes(&bytes);
    assert!(result.is_err());
}

// ── ProjectFile tests ─────────────────────────────────────────────────────────

use ot_parser::project::ProjectFile;

#[test]
fn test_project_parse() {
    let bytes = include_bytes!("../../../tests/fixtures/project.work");
    let parsed = ProjectFile::from_bytes(bytes).unwrap();
    // project.work is text-based; raw bytes are stored verbatim
    assert_eq!(parsed.to_bytes().unwrap().len(), bytes.len());
}

#[test]
fn test_project_round_trip() {
    let bytes = include_bytes!("../../../tests/fixtures/project.work");
    let parsed = ProjectFile::from_bytes(bytes).unwrap();
    let rewritten = parsed.to_bytes().unwrap();
    assert_eq!(bytes.as_ref(), rewritten.as_slice(), "ProjectFile byte-exact round-trip failed");
}

#[test]
fn test_project_preserves_unknown_regions() {
    // All bytes of project.work are preserved verbatim (it is stored as opaque bytes)
    let bytes = include_bytes!("../../../tests/fixtures/project.work");
    let parsed = ProjectFile::from_bytes(bytes).unwrap();
    assert_eq!(parsed.raw.as_slice(), bytes.as_ref());
}

// ── BankFile tests ────────────────────────────────────────────────────────────

use ot_parser::bank::BankFile;

/// BankFile header magic first 4 bytes: "FORM" = [0x46, 0x4F, 0x52, 0x4D]
const BANK_HEADER_MAGIC: &[u8] = &[0x46, 0x4F, 0x52, 0x4D];

#[test]
fn test_bank_parse() {
    let bytes = include_bytes!("../../../tests/fixtures/bank01.work");
    let parsed = BankFile::from_bytes(bytes).unwrap();
    assert_eq!(&parsed.header[0..4], BANK_HEADER_MAGIC);
    assert_eq!(parsed.datatype_version, 23);
}

#[test]
fn test_bank_round_trip() {
    let bytes = include_bytes!("../../../tests/fixtures/bank01.work");
    let parsed = BankFile::from_bytes(bytes).unwrap();
    let rewritten = parsed.to_bytes().unwrap();
    assert_eq!(bytes.as_ref(), rewritten.as_slice(), "BankFile byte-exact round-trip failed");
}

#[test]
fn test_bank_preserves_unknown_regions() {
    // Body between header+version and checksum is preserved verbatim as opaque blob
    let bytes = include_bytes!("../../../tests/fixtures/bank01.work");
    let parsed = BankFile::from_bytes(bytes).unwrap();
    let rewritten = parsed.to_bytes().unwrap();
    assert_eq!(bytes.as_ref(), rewritten.as_slice(), "BankFile opaque body not preserved verbatim");
}

#[test]
fn test_bank_wrong_magic() {
    let mut bytes = vec![0u8; 100];
    bytes[0] = 0xFF; // Wrong magic
    let result = BankFile::from_bytes(&bytes);
    assert!(result.is_err());
}
