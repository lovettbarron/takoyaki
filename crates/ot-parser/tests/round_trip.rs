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
