//! Parser and rewriter for OT `project.work` text files.
//!
//! `project.work` is a plain ASCII key=value text file. Slot entries use
//! `TYPE=FLEX` or `TYPE=STATIC` as inline discriminators (not bracketed section
//! headers). Each slot block contains `SLOT=NNN` and `PATH=...` lines.
//!
//! Assumption A1 (RESEARCH.md): The slot section uses `TYPE=FLEX`/`TYPE=STATIC`
//! inline discriminators. This needs verification against a real OT card.
//!
//! Threat model T-04-01: `validate_ot_name` restricts names to A-Z, a-z, 0-9,
//! underscore, max 16 chars — no path separators are possible.

use crate::error::AppError;
use serde::Serialize;
use specta::Type;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The type of a sample slot in the OT project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
pub enum SlotType {
    Flex,
    Static,
}

/// A sample slot path entry extracted from `project.work`.
#[derive(Debug, Clone, Serialize, Type)]
pub struct SlotPath {
    pub slot_type: SlotType,
    /// 1-indexed slot number (1..=128).
    pub slot_number: u8,
    /// OT-relative path, e.g. `../AUDIO/kick.wav`
    pub path: String,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Extract all slot PATH assignments from `project.work` bytes.
///
/// Iterates lines, tracking `TYPE=FLEX` / `TYPE=STATIC` inline discriminators
/// and `SLOT=NNN` lines. Captures `PATH=<value>` into [`SlotPath`] entries.
/// Returns slots in parse order.
pub fn extract_slot_paths(project_work_bytes: &[u8]) -> Vec<SlotPath> {
    let text = String::from_utf8_lossy(project_work_bytes);
    let mut results = Vec::new();
    let mut current_type: Option<SlotType> = None;
    let mut current_slot: Option<u8> = None;

    for line in text.lines() {
        let line = line.trim();
        if line == "TYPE=FLEX" {
            current_type = Some(SlotType::Flex);
            current_slot = None;
        } else if line == "TYPE=STATIC" {
            current_type = Some(SlotType::Static);
            current_slot = None;
        } else if let Some(rest) = line.strip_prefix("SLOT=") {
            // SLOT= values may be zero-padded (e.g. "001")
            current_slot = rest.trim().parse().ok();
        } else if let Some(rest) = line.strip_prefix("PATH=") {
            if let (Some(slot_type), Some(slot_number)) = (current_type, current_slot) {
                results.push(SlotPath {
                    slot_type,
                    slot_number,
                    path: rest.to_string(),
                });
            }
        }
    }
    results
}

/// Rewrite a single slot's `PATH=` value in `project.work` bytes.
///
/// Finds the `PATH=` line for the given `slot_type` + `slot_number` combination
/// and replaces only that line's value. All other content is preserved
/// byte-for-byte. If the slot is not found, `raw` is returned unchanged.
pub fn rewrite_slot_path(
    raw: &[u8],
    slot_type: SlotType,
    slot_number: u8,
    new_path: &str,
) -> Vec<u8> {
    let text = match std::str::from_utf8(raw) {
        Ok(t) => t,
        Err(_) => {
            // Non-UTF-8 content: return unchanged (safety fallback)
            return raw.to_vec();
        }
    };

    let mut current_type: Option<SlotType> = None;
    let mut current_slot: Option<u8> = None;
    let mut output = String::with_capacity(text.len() + new_path.len());

    // Detect the line ending style used in this file (CR+LF or LF).
    let uses_crlf = text.contains("\r\n");

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed == "TYPE=FLEX" {
            current_type = Some(SlotType::Flex);
            current_slot = None;
        } else if trimmed == "TYPE=STATIC" {
            current_type = Some(SlotType::Static);
            current_slot = None;
        } else if let Some(rest) = trimmed.strip_prefix("SLOT=") {
            current_slot = rest.trim().parse().ok();
        }

        let is_target_path = trimmed.starts_with("PATH=")
            && current_type == Some(slot_type)
            && current_slot == Some(slot_number);

        if is_target_path {
            output.push_str(&format!("PATH={}", new_path));
        } else {
            output.push_str(line);
        }

        if uses_crlf {
            output.push_str("\r\n");
        } else {
            output.push('\n');
        }
    }

    // Preserve trailing newline absence if original had none
    if !text.ends_with('\n') && output.ends_with('\n') {
        output.pop();
        if uses_crlf && output.ends_with('\r') {
            output.pop();
        }
    }

    output.into_bytes()
}

/// Validate an OT project/directory name.
///
/// OT name rules:
/// - Must not be empty
/// - Max 16 characters
/// - Characters: A-Z, a-z, 0-9, underscore only (lowercase accepted; OT displays uppercase)
///
/// Threat model T-04-01: no path separators (`/`, `\`, `.`) are allowed, preventing
/// directory traversal when the name is joined with the SETS path.
pub fn validate_ot_name(name: &str) -> Result<(), AppError> {
    if name.is_empty() {
        return Err(AppError::Parse("OT name cannot be empty".to_string()));
    }
    if name.len() > 16 {
        return Err(AppError::Parse(
            "OT name exceeds 16 character limit".to_string(),
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_')
    {
        return Err(AppError::Parse(
            "OT name must contain only A-Z, 0-9, underscore".to_string(),
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal project.work fixture with 2 FLEX slots and 1 STATIC slot.
    fn fixture_two_flex_one_static() -> &'static [u8] {
        b"TYPE=FLEX\n\
          SLOT=001\n\
          PATH=../AUDIO/kick.wav\n\
          GAIN=48\n\
          TYPE=FLEX\n\
          SLOT=002\n\
          PATH=../AUDIO/snare.wav\n\
          GAIN=48\n\
          TYPE=STATIC\n\
          SLOT=001\n\
          PATH=../AUDIO/pad.wav\n\
          GAIN=48\n"
    }

    #[test]
    fn test_extract_flex_and_static_slots() {
        let slots = extract_slot_paths(fixture_two_flex_one_static());
        assert_eq!(slots.len(), 3, "Expected 3 slots");

        assert_eq!(slots[0].slot_type, SlotType::Flex);
        assert_eq!(slots[0].slot_number, 1);
        assert_eq!(slots[0].path, "../AUDIO/kick.wav");

        assert_eq!(slots[1].slot_type, SlotType::Flex);
        assert_eq!(slots[1].slot_number, 2);
        assert_eq!(slots[1].path, "../AUDIO/snare.wav");

        assert_eq!(slots[2].slot_type, SlotType::Static);
        assert_eq!(slots[2].slot_number, 1);
        assert_eq!(slots[2].path, "../AUDIO/pad.wav");
    }

    #[test]
    fn test_extract_empty_input() {
        let slots = extract_slot_paths(b"");
        assert!(slots.is_empty(), "Empty input should produce empty Vec");
    }

    #[test]
    fn test_rewrite_path_updates_correct_slot() {
        let raw = fixture_two_flex_one_static();
        let rewritten = rewrite_slot_path(raw, SlotType::Flex, 1, "../AUDIO/new_kick.wav");
        let rewritten_str = String::from_utf8(rewritten).unwrap();

        // The rewritten slot should have the new path
        assert!(
            rewritten_str.contains("PATH=../AUDIO/new_kick.wav"),
            "Rewritten content should contain new path"
        );

        // The other slots should be unchanged
        assert!(
            rewritten_str.contains("PATH=../AUDIO/snare.wav"),
            "Other FLEX slot should be unchanged"
        );
        assert!(
            rewritten_str.contains("PATH=../AUDIO/pad.wav"),
            "STATIC slot should be unchanged"
        );

        // The old path for slot 1 FLEX should no longer appear
        assert!(
            !rewritten_str.contains("PATH=../AUDIO/kick.wav"),
            "Old path for FLEX slot 1 should be replaced"
        );
    }

    #[test]
    fn test_validate_ot_name_valid() {
        assert!(
            validate_ot_name("MY_PROJECT").is_ok(),
            "MY_PROJECT should be valid"
        );
        assert!(
            validate_ot_name("LIVESET_01").is_ok(),
            "LIVESET_01 should be valid"
        );
        assert!(
            validate_ot_name("abc123").is_ok(),
            "Lowercase letters should be accepted"
        );
        assert!(
            validate_ot_name("A").is_ok(),
            "Single character should be valid"
        );
    }

    #[test]
    fn test_validate_ot_name_too_long() {
        // 17 characters — exceeds 16 char limit
        let name = "ABCDEFGHIJKLMNOPQ";
        assert_eq!(name.len(), 17);
        let result = validate_ot_name(name);
        assert!(result.is_err(), "17-char name should fail");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("16 character limit"),
            "Error should mention 16 character limit"
        );
    }

    #[test]
    fn test_validate_ot_name_invalid_chars() {
        let result = validate_ot_name("MY PROJECT");
        assert!(result.is_err(), "Space in name should fail");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("A-Z, 0-9, underscore"),
            "Error should describe allowed characters"
        );

        // Also test other invalid characters
        assert!(validate_ot_name("MY/PROJECT").is_err(), "Slash should fail");
        assert!(validate_ot_name("MY.PROJECT").is_err(), "Dot should fail");
    }

    #[test]
    fn test_validate_ot_name_empty() {
        let result = validate_ot_name("");
        assert!(result.is_err(), "Empty name should fail");
        assert!(
            result.unwrap_err().to_string().contains("cannot be empty"),
            "Error should mention empty"
        );
    }
}
