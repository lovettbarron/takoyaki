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

#[test]
#[ignore = "Requires Plan 02 production code: health::perform_health_check with SlotCheckInput"]
fn test_health_missing_file() {
    // DETC-01: missing sample file detected as Error severity
    // Setup: construct a SlotCheckInput with a path that does not exist
    // Act: run the health check logic on this slot
    // Assert: result contains HealthIssue::Error with "File not found" in detail
    todo!("Plan 02 creates health::perform_health_check — integration test needs async runtime")
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

#[test]
#[ignore = "Requires Plan 02 production code: health::perform_health_check with SlotCheckInput"]
fn test_health_unused_sample() {
    // DETC-03: slot with no track references detected as Info severity
    // Setup: construct a SlotCheckInput with occupied=true, empty track_references
    // Act: run the health check logic on this slot
    // Assert: result contains HealthIssue::Info with "not referenced" in detail
    todo!("Plan 02 creates health::perform_health_check — integration test needs async runtime")
}
