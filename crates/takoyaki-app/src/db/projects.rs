//! SQLite query functions for the project index.
//!
//! Threat model T-02-01: All user-supplied filter values use parameterized queries
//! via rusqlite `params![]` — never string interpolation.

use rusqlite::{params, Connection};
use specta::Type;

/// Filter parameters for project list queries (MGMT-04).
///
/// All fields optional — an empty filter returns all projects.
#[derive(Debug, serde::Deserialize, Type)]
pub struct ProjectFilter {
    pub name: Option<String>,
    pub bpm_min: Option<u16>,
    pub bpm_max: Option<u16>,
    /// ISO 8601 date string, e.g. "2026-01-01"
    pub modified_since: Option<String>,
}

/// A project row as returned by the list query (BROW-02).
#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct ProjectSummary {
    pub id: String,
    pub set_name: String,
    pub project_name: String,
    pub card_path: String,
    /// Actual BPM value (e.g. 120.0), NOT the raw stored integer.
    pub tempo_bpm: Option<f32>,
    /// Number of banks used out of 16.
    pub bank_count: Option<u8>,
    /// ISO date string.
    pub last_modified: Option<String>,
}

/// A full project row for insertion / upsert.
pub struct ProjectRow {
    pub id: String,
    pub set_name: String,
    pub project_name: String,
    pub card_path: String,
    pub tempo_bpm: Option<f32>,
    pub bank_count: Option<u8>,
    pub last_modified: Option<String>,
}

/// Insert or replace a project row in the `projects` table.
pub fn upsert_project(conn: &Connection, row: &ProjectRow) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO projects
            (id, set_name, project_name, card_path, tempo_bpm, bank_count, last_modified)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            row.id,
            row.set_name,
            row.project_name,
            row.card_path,
            row.tempo_bpm,
            row.bank_count.map(|v| v as i64),
            row.last_modified,
        ],
    )?;
    Ok(())
}

/// Query the project index with optional filtering (BROW-02, MGMT-04).
///
/// All filter values are passed as parameterized query parameters — no string
/// interpolation of user-supplied values (T-02-01 mitigation).
pub fn list_projects(
    conn: &Connection,
    filter: &ProjectFilter,
) -> rusqlite::Result<Vec<ProjectSummary>> {
    // Build WHERE clause with positional placeholders.
    // "1=1" as the base ensures a syntactically valid WHERE clause even with no filters.
    let mut conditions: Vec<&str> = vec!["1=1"];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    if let Some(ref name) = filter.name {
        conditions.push("project_name LIKE ?");
        params.push(Box::new(format!("%{name}%")));
    }
    if let Some(bpm_min) = filter.bpm_min {
        conditions.push("tempo_bpm >= ?");
        params.push(Box::new(bpm_min as f64));
    }
    if let Some(bpm_max) = filter.bpm_max {
        conditions.push("tempo_bpm <= ?");
        params.push(Box::new(bpm_max as f64));
    }
    if let Some(ref since) = filter.modified_since {
        conditions.push("last_modified >= ?");
        params.push(Box::new(since.clone()));
    }

    let sql = format!(
        "SELECT id, set_name, project_name, card_path, tempo_bpm, bank_count, last_modified
         FROM projects
         WHERE {}
         ORDER BY last_modified DESC NULLS LAST",
        conditions.join(" AND ")
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok(ProjectSummary {
            id: row.get(0)?,
            set_name: row.get(1)?,
            project_name: row.get(2)?,
            card_path: row.get(3)?,
            tempo_bpm: row.get(4)?,
            bank_count: row.get::<_, Option<i64>>(5)?.map(|v| v as u8),
            last_modified: row.get(6)?,
        })
    })?;

    rows.collect()
}

/// Look up the `card_path` for a project by its UUID (used by health check and detail commands).
pub fn get_card_path(conn: &Connection, project_id: &str) -> rusqlite::Result<String> {
    conn.query_row(
        "SELECT card_path FROM projects WHERE id = ?1",
        params![project_id],
        |row| row.get(0),
    )
}

/// Delete all rows from the `projects` table (called before re-indexing on card mount).
pub fn clear_projects(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM projects", [])?;
    Ok(())
}
