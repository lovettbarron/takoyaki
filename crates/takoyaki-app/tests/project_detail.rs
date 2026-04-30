//! Integration tests for project detail, banks, and samples (BROW-03, BROW-04, BROW-05)

#[test]
#[ignore = "Requires Plan 01 production code and Phase 1 OT parser fixtures"]
fn test_get_project_banks() {
    // BROW-03: get_project_banks returns 16 entries with populated flags
    // Setup: parse fixture bank files
    // Act: call get_project_banks
    // Assert: returns Vec<BankSummary> with length 16, each with bank_index and populated flag
    todo!("Plan 01 creates commands::projects::get_project_banks")
}

#[test]
#[ignore = "Requires Plan 01 production code and Phase 1 OT parser fixtures"]
fn test_get_project_samples() {
    // BROW-04: get_project_samples returns Flex[128] + Static[128]
    // Setup: parse fixture project.work
    // Act: call get_project_samples
    // Assert: returns SampleSlotResponse with flex.len() == 128 and static_slots.len() == 128
    todo!("Plan 01 creates commands::samples::get_project_samples")
}

#[test]
#[ignore = "Requires Plan 01 production code and Phase 1 OT parser fixtures"]
fn test_get_project_detail() {
    // BROW-05: get_project_detail returns tempo, bank names, part names, machine types
    // Setup: parse fixture project.work + bank files
    // Act: call get_project_detail
    // Assert: ProjectDetail has tempo_bpm as f32 (not raw integer), banks with parts and tracks
    // Assert: tempo is divided by 10 from raw value (assumption A2 guard)
    todo!("Plan 01 creates commands::projects::get_project_detail")
}
