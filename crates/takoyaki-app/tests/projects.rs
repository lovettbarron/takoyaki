//! Integration tests for project list, search, and filter (BROW-02, MGMT-04)

#[test]
#[ignore = "Requires Plan 01 production code: db::projects::list_projects"]
fn test_list_projects() {
    // BROW-02: list_projects returns all projects from SQLite index
    // Setup: insert mock project rows into in-memory SQLite DB
    // Act: call list_projects with empty filter
    // Assert: returns all inserted projects with name, tempo_bpm, bank_count, last_modified
    todo!("Plan 01 creates db::projects module")
}

#[test]
#[ignore = "Requires Plan 01 production code: db::projects::list_projects"]
fn test_list_projects_filter_name() {
    // MGMT-04: filter projects by name substring
    // Setup: insert 3 projects ("LIVESET", "TECHNO_01", "AMBIENT")
    // Act: call list_projects with filter.name = Some("TECH")
    // Assert: returns only "TECHNO_01"
    todo!("Plan 01 creates db::projects module")
}

#[test]
#[ignore = "Requires Plan 01 production code: db::projects::list_projects"]
fn test_list_projects_filter_bpm() {
    // MGMT-04: filter projects by BPM range
    // Setup: insert projects at 90.0, 120.0, 140.0 BPM
    // Act: call list_projects with filter.bpm_min = Some(100), filter.bpm_max = Some(130)
    // Assert: returns only the 120.0 BPM project
    todo!("Plan 01 creates db::projects module")
}
