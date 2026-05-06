//! Integration tests for project detail, banks, and samples (BROW-03, BROW-04, BROW-05)
//! Tests the underlying parsing functions against mock OT volume fixtures.

use std::path::Path;

fn fixture_path(relative: &str) -> std::path::PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    Path::new(&manifest)
        .parent().unwrap()  // crates/
        .parent().unwrap()  // project root
        .join("tests/fixtures/mock_ot_volume")
        .join(relative)
}

#[test]
fn test_get_project_samples_from_fixture() {
    // BROW-04: parse_project_work returns real occupied slots from fixture
    let project_work_path = fixture_path("SETS/LIVESET/PROJECT_01/project.work");
    assert!(project_work_path.exists(), "Fixture must exist: {}", project_work_path.display());

    let raw = std::fs::read(&project_work_path).expect("Should read project.work fixture");
    let parsed = takoyaki_app::commands::samples::parse_project_work(&raw);

    // FLEX0 is occupied (../AUDIO/kick_44100.wav from fixture)
    assert!(parsed.flex_slots[0].is_some(), "FLEX0 should be occupied");
    assert_eq!(
        parsed.flex_slots[0].as_deref(),
        Some("../AUDIO/kick_44100.wav"),
        "FLEX0 path should match fixture"
    );
    // FLEX1 through FLEX127 should be empty
    assert!(parsed.flex_slots[1].is_none(), "FLEX1 should be empty");

    // STAT0 is occupied (../AUDIO/pad_48000.wav from fixture)
    assert!(parsed.static_slots[0].is_some(), "STAT0 should be occupied");
    assert_eq!(
        parsed.static_slots[0].as_deref(),
        Some("../AUDIO/pad_48000.wav"),
        "STAT0 path should match fixture"
    );
    // STAT1 through STAT127 should be empty
    assert!(parsed.static_slots[1].is_none(), "STAT1 should be empty");

    // Total: 128 flex + 128 static
    assert_eq!(parsed.flex_slots.len(), 128);
    assert_eq!(parsed.static_slots.len(), 128);
}

#[test]
fn test_get_project_detail_tempo_from_fixture() {
    // BROW-05: parse_project_work extracts real tempo from fixture
    let project_work_path = fixture_path("SETS/LIVESET/PROJECT_01/project.work");
    let raw = std::fs::read(&project_work_path).expect("Should read project.work fixture");
    let parsed = takoyaki_app::commands::samples::parse_project_work(&raw);

    // Fixture has TEMPO:1200 (120 BPM at TEMPO_SCALE_FACTOR=10.0)
    assert_eq!(parsed.tempo_raw, Some(1200), "Tempo raw should be 1200");
    let display_bpm = parsed.tempo_raw.unwrap() as f32 / 10.0;
    assert!((display_bpm - 120.0).abs() < 0.01, "Display BPM should be 120.0, got {}", display_bpm);
}

#[test]
fn test_get_project_banks_bank_file_check() {
    // BROW-03: bank file presence check against fixture
    // bank01.work exists in fixture but contains PLACEHOLDER_BANK_WORK (not valid FORM header)
    let project_dir = fixture_path("SETS/LIVESET/PROJECT_01");
    let bank_path = project_dir.join("bank01.work");
    assert!(bank_path.exists(), "bank01.work fixture should exist");

    // Read and try to parse -- should fail because placeholder is not valid FORM format
    let data = std::fs::read(&bank_path).expect("Should read bank01.work");
    let result = ot_parser::BankFile::from_bytes(&data);
    assert!(result.is_err(), "Placeholder bank file should not parse as valid BankFile");

    // bank02.work does not exist -- should also be treated as not populated
    let bank02_path = project_dir.join("bank02.work");
    assert!(!bank02_path.exists(), "bank02.work should not exist in fixture");
}
