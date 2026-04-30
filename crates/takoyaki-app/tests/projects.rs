//! Integration tests for project list, search, and filter (BROW-02, MGMT-04)

/// Helper: create an in-memory database and populate it with test project rows.
fn setup_db() -> rusqlite::Connection {
    // We reach into the internal db module via the public API surface we control.
    // Since Database is not re-exported, we open a raw rusqlite connection and
    // run the same schema DDL that db::Database::initialize() would apply.
    let conn = rusqlite::Connection::open_in_memory().expect("open_in_memory");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS projects (
            id            TEXT PRIMARY KEY,
            set_name      TEXT NOT NULL DEFAULT '',
            project_name  TEXT NOT NULL DEFAULT '',
            card_path     TEXT NOT NULL DEFAULT '',
            tempo_bpm     REAL,
            bank_count    INTEGER,
            last_modified TEXT
        );",
    )
    .expect("create schema");
    conn
}

fn insert_project(
    conn: &rusqlite::Connection,
    id: &str,
    project_name: &str,
    tempo_bpm: Option<f64>,
    last_modified: Option<&str>,
) {
    conn.execute(
        "INSERT INTO projects (id, set_name, project_name, card_path, tempo_bpm, last_modified)
         VALUES (?1, 'LIVESET', ?2, '/card/SETS/LIVESET/' || ?2, ?3, ?4)",
        rusqlite::params![id, project_name, tempo_bpm, last_modified],
    )
    .expect("insert project");
}

#[test]
fn test_list_projects() {
    // BROW-02: list_projects returns all projects from SQLite index
    use takoyaki_app::db::projects::{list_projects, ProjectFilter};

    let conn = setup_db();
    insert_project(&conn, "id-1", "LIVESET_01", Some(120.0), Some("2026-01-01"));
    insert_project(&conn, "id-2", "TECHNO_01", Some(140.0), Some("2026-02-01"));
    insert_project(&conn, "id-3", "AMBIENT", Some(90.0), Some("2026-03-01"));

    let filter = ProjectFilter {
        name: None,
        bpm_min: None,
        bpm_max: None,
        modified_since: None,
    };
    let results = list_projects(&conn, &filter).expect("list_projects");

    assert_eq!(results.len(), 3, "Should return all 3 projects");
    // Results are ordered by last_modified DESC
    assert_eq!(results[0].project_name, "AMBIENT");
    assert_eq!(results[1].project_name, "TECHNO_01");
    assert_eq!(results[2].project_name, "LIVESET_01");
}

#[test]
fn test_list_projects_filter_name() {
    // MGMT-04: filter projects by name substring
    use takoyaki_app::db::projects::{list_projects, ProjectFilter};

    let conn = setup_db();
    insert_project(&conn, "id-1", "LIVESET_01", Some(120.0), Some("2026-01-01"));
    insert_project(&conn, "id-2", "TECHNO_01", Some(140.0), Some("2026-01-02"));
    insert_project(&conn, "id-3", "AMBIENT", Some(90.0), Some("2026-01-03"));

    let filter = ProjectFilter {
        name: Some("TECH".to_string()),
        bpm_min: None,
        bpm_max: None,
        modified_since: None,
    };
    let results = list_projects(&conn, &filter).expect("list_projects with name filter");

    assert_eq!(results.len(), 1, "Should return only TECHNO_01");
    assert_eq!(results[0].project_name, "TECHNO_01");
}

#[test]
fn test_list_projects_filter_bpm() {
    // MGMT-04: filter projects by BPM range
    use takoyaki_app::db::projects::{list_projects, ProjectFilter};

    let conn = setup_db();
    insert_project(&conn, "id-1", "LIVESET_01", Some(90.0), Some("2026-01-01"));
    insert_project(&conn, "id-2", "TECHNO_01", Some(120.0), Some("2026-01-02"));
    insert_project(&conn, "id-3", "AMBIENT", Some(140.0), Some("2026-01-03"));

    let filter = ProjectFilter {
        name: None,
        bpm_min: Some(100),
        bpm_max: Some(130),
        modified_since: None,
    };
    let results = list_projects(&conn, &filter).expect("list_projects with bpm filter");

    assert_eq!(results.len(), 1, "Should return only the 120.0 BPM project");
    assert_eq!(results[0].project_name, "TECHNO_01");
    assert!(
        (results[0].tempo_bpm.unwrap() - 120.0_f32).abs() < 0.01,
        "BPM should be 120.0"
    );
}
