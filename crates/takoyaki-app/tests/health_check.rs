//! Unit tests for health check engine (DETC-01, DETC-02, DETC-03)

use std::path::Path;

// Fixture path helper
fn fixture_path(relative: &str) -> std::path::PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    Path::new(&manifest)
        .parent()
        .unwrap() // crates/
        .parent()
        .unwrap() // project root
        .join("tests/fixtures/mock_ot_volume")
        .join(relative)
}

#[tokio::test]
async fn test_health_missing_file() {
    // DETC-01: missing sample file detected as Error severity
    let volume_path = fixture_path("");
    let slot_inputs = vec![
        takoyaki_app::health::SlotCheckInput {
            slot_type: "flex".to_string(),
            slot_index: 0,
            occupied: true,
            raw_path: Some("../AUDIO/nonexistent_file.wav".to_string()),
            track_references: vec![],
        },
    ];

    let project_path = fixture_path("SETS/LIVESET/PROJECT_01")
        .to_string_lossy()
        .to_string();
    let issues = takoyaki_app::health::perform_health_check(
        &project_path,
        &volume_path,
        &slot_inputs,
    ).await;

    assert!(!issues.is_empty(), "Should detect at least one issue for missing file");
    let has_error = issues.iter().any(|issue| {
        matches!(issue, takoyaki_app::health::HealthIssue::Error { .. })
    });
    assert!(has_error, "Missing file should produce Error severity, got: {:?}", issues);
}

#[test]
fn test_health_wrong_sample_rate() {
    // DETC-02: 48 kHz WAV detected as Warning severity via read_audio_spec + check_format_compatibility
    let path = fixture_path("AUDIO/pad_48000.wav");
    assert!(path.exists(), "Fixture file must exist: {}", path.display());

    let spec = takoyaki_app::health::read_audio_spec(&path).expect("Should read WAV spec");
    let issues = takoyaki_app::health::check_format_compatibility(&spec);

    assert!(!issues.is_empty(), "48kHz WAV should have format issues");
    let has_wrong_rate = issues.iter().any(|issue| {
        matches!(issue, takoyaki_app::health::FormatIssue::WrongSampleRate(48000))
    });
    assert!(has_wrong_rate, "Should detect WrongSampleRate(48000), got: {:?}", issues);
}

#[test]
fn test_health_correct_sample_rate() {
    // DETC-02 (negative case): 44.1 kHz WAV produces no format issues
    let path = fixture_path("AUDIO/kick_44100.wav");
    assert!(path.exists(), "Fixture file must exist: {}", path.display());

    let spec = takoyaki_app::health::read_audio_spec(&path).expect("Should read WAV spec");
    let issues = takoyaki_app::health::check_format_compatibility(&spec);

    assert!(
        issues.is_empty(),
        "44.1kHz WAV should have no format issues, got: {:?}",
        issues
    );
}

#[test]
fn test_health_unsupported_format() {
    // DETC-02: non-WAV/AIFF file detected via check_format_compatibility
    let path = fixture_path("AUDIO/not_audio.txt");
    assert!(path.exists(), "Fixture file must exist: {}", path.display());

    let spec = takoyaki_app::health::read_audio_spec(&path).expect("Should return Unknown");
    assert!(
        matches!(spec, takoyaki_app::health::AudioSpec::Unknown { .. }),
        "Text file should produce AudioSpec::Unknown, got {:?}", spec
    );

    let issues = takoyaki_app::health::check_format_compatibility(&spec);
    assert_eq!(issues.len(), 1, "Should have exactly one format issue");
    assert!(
        matches!(issues[0], takoyaki_app::health::FormatIssue::UnsupportedFormat(_)),
        "Should detect UnsupportedFormat, got: {:?}", issues
    );
}

#[tokio::test]
async fn test_health_unused_sample_suppressed_when_no_track_refs() {
    // DETC-03: When track_references is empty (bank body opaque), the "unused sample"
    // Info issue should NOT be emitted (Phase 7 suppression guard).
    let volume_path = fixture_path("");
    let slot_inputs = vec![
        takoyaki_app::health::SlotCheckInput {
            slot_type: "flex".to_string(),
            slot_index: 0,
            occupied: true,
            raw_path: Some("AUDIO/kick_44100.wav".to_string()),
            track_references: vec![], // Empty -- DETC-03 should be suppressed
        },
    ];

    let project_path = fixture_path("SETS/LIVESET/PROJECT_01")
        .to_string_lossy()
        .to_string();
    let issues = takoyaki_app::health::perform_health_check(
        &project_path,
        &volume_path,
        &slot_inputs,
    ).await;

    // Should NOT have any Info issues about "not referenced by any track"
    let has_unused_info = issues.iter().any(|issue| {
        match issue {
            takoyaki_app::health::HealthIssue::Info { detail, .. } => {
                detail.contains("not referenced")
            }
            _ => false,
        }
    });
    assert!(!has_unused_info, "DETC-03 should be suppressed when track_references is empty, got: {:?}", issues);
}
