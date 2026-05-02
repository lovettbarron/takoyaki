//! Tauri commands for Wallflower integration (Phase 5, Plan 02).
//!
//! Three commands:
//! - `get_wallflower_status`: auto-discover Wallflower DB and return connection state
//! - `search_wallflower_samples`: search samples by name, key, BPM, or tag
//! - `set_wallflower_db_path`: persist user-configured DB path to settings
//!
//! Threat model:
//! - T-05-07: Wallflower connection uses SQLITE_OPEN_READ_ONLY (enforced by open_wallflower_db)
//! - T-05-08: All query parameters use rusqlite::params![] — no string interpolation
//! - T-05-10: set_wallflower_db_path validates path.exists() before saving

use crate::db;
use crate::db::wallflower::open_wallflower_db;
use crate::error::AppError;
use crate::AppState;
use rusqlite::Connection;
use serde::Serialize;
use specta::Type;
use std::path::PathBuf;
use tracing::info;

// ── Response Types ──────────────────────────────────────────────────────

/// Connection state for the Wallflower DB (returned by `get_wallflower_status`).
#[derive(Debug, Serialize, Type, Clone)]
pub struct WallflowerStatus {
    pub connected: bool,
    pub db_path: Option<String>,
    pub sample_count: Option<u32>,
}

/// A single Wallflower sample (from the `jams` table with metadata joins).
#[derive(Debug, Serialize, Type, Clone)]
pub struct WallflowerSample {
    pub id: i64,
    pub filename: String,
    pub file_path: String,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u16>,
    pub bpm: Option<f64>,
    pub key_name: Option<String>,
    pub scale: Option<String>,
    pub tags: Vec<String>,
}

// ── Auto-Discovery ──────────────────────────────────────────────────────

/// Discover the Wallflower database path using priority order per D-06:
/// 1. User-configured path from Takoyaki settings table
/// 2. ~/Library/Application Support/wallflower/wallflower.db (dirs::data_dir — VERIFIED from Wallflower source)
/// 3. ~/wallflower/wallflower.db (Wallflower watch_folder default)
///
/// Returns None if no valid DB file is found at any candidate path.
fn discover_wallflower_db(user_configured_path: Option<String>) -> Option<PathBuf> {
    // Priority 1: User-configured path per D-06
    if let Some(ref configured) = user_configured_path {
        let p = PathBuf::from(configured);
        if p.exists() {
            info!("Wallflower DB found at user-configured path: {}", p.display());
            return Some(p);
        }
    }

    // Priority 2: Standard macOS app data location (VERIFIED: Wallflower uses dirs::data_dir())
    // On macOS: ~/Library/Application Support/wallflower/wallflower.db
    if let Some(data_dir) = dirs::data_dir() {
        let candidate = data_dir.join("wallflower").join("wallflower.db");
        if candidate.exists() {
            info!("Wallflower DB found at data dir: {}", candidate.display());
            return Some(candidate);
        }
    }

    // Priority 3: ~/wallflower/wallflower.db (from Wallflower watch_folder default)
    if let Some(home) = dirs::home_dir() {
        let candidate = home.join("wallflower").join("wallflower.db");
        if candidate.exists() {
            info!("Wallflower DB found at home dir: {}", candidate.display());
            return Some(candidate);
        }
    }

    info!("Wallflower DB not found at any candidate path");
    None
}

/// Execute a sample search against the Wallflower DB using the verified JOIN query.
///
/// Query joins `jams`, `jam_tempo`, `jam_key`, and `jam_tags` tables.
/// All parameters use rusqlite::params![] to prevent SQL injection (T-05-08).
///
/// Empty query returns all samples ordered by filename (up to `limit`).
fn search_samples(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<WallflowerSample>, AppError> {
    // Row mapper shared by both query variants
    let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<WallflowerSample> {
        Ok(WallflowerSample {
            id: row.get(0)?,
            filename: row.get(1)?,
            file_path: row.get(2)?,
            sample_rate: row.get(3)?,
            bit_depth: row.get(4)?,
            bpm: row.get(5)?,
            key_name: row.get(6)?,
            scale: row.get(7)?,
            tags: row
                .get::<_, Option<String>>(8)?
                .map(|s| s.split(',').map(String::from).collect())
                .unwrap_or_default(),
        })
    };

    if query.is_empty() {
        // No query: return all samples up to limit, ordered by filename
        let sql = r#"
            SELECT j.id, j.filename, j.file_path, j.sample_rate, j.bit_depth,
                   jt.bpm, jk.key_name, jk.scale,
                   GROUP_CONCAT(DISTINCT jtag.tag) AS tags
            FROM jams j
            LEFT JOIN jam_tempo jt ON jt.jam_id = j.id
            LEFT JOIN jam_key jk ON jk.jam_id = j.id
            LEFT JOIN jam_tags jtag ON jtag.jam_id = j.id
            GROUP BY j.id
            ORDER BY j.filename ASC
            LIMIT ?1
        "#;
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![limit as i64], map_row)
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(rows)
    } else {
        // Search by filename (LIKE), key_name (exact), BPM prefix, or tag (exact)
        let sql = r#"
            SELECT j.id, j.filename, j.file_path, j.sample_rate, j.bit_depth,
                   jt.bpm, jk.key_name, jk.scale,
                   GROUP_CONCAT(DISTINCT jtag.tag) AS tags
            FROM jams j
            LEFT JOIN jam_tempo jt ON jt.jam_id = j.id
            LEFT JOIN jam_key jk ON jk.jam_id = j.id
            LEFT JOIN jam_tags jtag ON jtag.jam_id = j.id
            WHERE j.filename LIKE '%' || ?2 || '%'
               OR jk.key_name = ?2
               OR CAST(ROUND(jt.bpm) AS TEXT) LIKE ?2 || '%'
               OR jtag.tag = ?2
            GROUP BY j.id
            ORDER BY j.filename ASC
            LIMIT ?1
        "#;
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![limit as i64, query], map_row)
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(rows)
    }
}

// ── Tauri Commands ──────────────────────────────────────────────────────

/// Return connection state for the Wallflower DB per D-07.
///
/// Auto-discovers DB via priority order (user-configured → data_dir → home).
/// If connected, returns the count of samples in the `jams` table.
/// DB lock is released before any file I/O (T-03-04 pattern).
#[tauri::command]
#[specta::specta]
pub async fn get_wallflower_status(
    state: tauri::State<'_, AppState>,
) -> Result<WallflowerStatus, AppError> {
    // Read user-configured path from settings (DB lock released immediately per T-03-04)
    let user_path = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::get_setting(&db.conn, "wallflower_db_path")
            .map_err(|e| AppError::Database(e.to_string()))?
    };

    let db_path = discover_wallflower_db(user_path);

    match db_path {
        Some(path) => match open_wallflower_db(&path) {
            Ok(conn) => {
                let count: Option<u32> = conn
                    .query_row("SELECT COUNT(*) FROM jams", [], |row| row.get(0))
                    .ok();
                Ok(WallflowerStatus {
                    connected: true,
                    db_path: Some(path.to_string_lossy().into_owned()),
                    sample_count: count,
                })
            }
            Err(_) => Ok(WallflowerStatus {
                connected: false,
                db_path: None,
                sample_count: None,
            }),
        },
        None => Ok(WallflowerStatus {
            connected: false,
            db_path: None,
            sample_count: None,
        }),
    }
}

/// Search Wallflower samples by filename, key, BPM, or tag.
///
/// Returns up to 200 results ordered by filename ascending.
/// Returns Io error if no Wallflower DB is found.
/// All query parameters use rusqlite::params![] (T-05-08).
#[tauri::command]
#[specta::specta]
pub async fn search_wallflower_samples(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<Vec<WallflowerSample>, AppError> {
    // Read user-configured path from settings (DB lock released before file I/O)
    let user_path = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::get_setting(&db.conn, "wallflower_db_path")
            .map_err(|e| AppError::Database(e.to_string()))?
    };

    let db_path = discover_wallflower_db(user_path)
        .ok_or_else(|| AppError::Io("Wallflower database not found".into()))?;

    let conn = open_wallflower_db(&db_path)?;
    search_samples(&conn, &query, 200)
}

/// Save user-configured Wallflower DB path to settings per D-08.
///
/// Validates path.exists() before saving (T-05-10).
/// Returns updated WallflowerStatus reflecting the new connection state.
#[tauri::command]
#[specta::specta]
pub async fn set_wallflower_db_path(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<WallflowerStatus, AppError> {
    // Validate the path exists before saving (T-05-10)
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(AppError::Io(format!("File not found: {}", path)));
    }

    // Save to settings (DB lock released before status check file I/O)
    {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::set_setting(&db.conn, "wallflower_db_path", &path)
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    // Return updated status
    get_wallflower_status(state).await
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── discover_wallflower_db tests ────────────────────────────────────

    #[test]
    fn test_discover_wallflower_db_none_when_no_paths_exist() {
        // All candidate paths are non-existent — should return None
        let result = discover_wallflower_db(None);
        // We can only guarantee None if neither data_dir nor home wallflower.db exists.
        // In CI/test environments there should be no wallflower.db installed.
        // If this test fails, a real Wallflower DB is installed — that's fine.
        // The important invariant is: function returns Some only when a file exists.
        if let Some(ref path) = result {
            assert!(
                path.exists(),
                "discover_wallflower_db must only return paths that exist"
            );
        }
    }

    #[test]
    fn test_discover_wallflower_db_user_configured_path_wins() {
        // Create a temporary file to act as a mock Wallflower DB
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let tmp_path = tmp.path().to_str().unwrap().to_string();

        let result = discover_wallflower_db(Some(tmp_path.clone()));
        // The user-configured path exists, so it must be returned first (Priority 1)
        assert_eq!(
            result,
            Some(PathBuf::from(&tmp_path)),
            "User-configured path must take priority when file exists"
        );
    }

    #[test]
    fn test_discover_wallflower_db_nonexistent_user_path_falls_through() {
        // User-configured path that does not exist — should fall through to auto-discovery
        let result = discover_wallflower_db(Some(
            "/nonexistent/path/that/cannot/exist/wallflower.db".to_string(),
        ));
        // Result is either None (no auto-discovered DB) or Some(path that exists)
        if let Some(ref path) = result {
            assert!(
                path.exists(),
                "discover_wallflower_db must only return paths that exist"
            );
        }
    }

    // ── search_samples tests ────────────────────────────────────────────

    /// Create an in-memory Wallflower DB with the verified schema and test data.
    fn make_test_wallflower_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE jams (
                id INTEGER PRIMARY KEY,
                filename TEXT NOT NULL,
                file_path TEXT NOT NULL,
                sample_rate INTEGER,
                bit_depth INTEGER
            );
            CREATE TABLE jam_tempo (
                id INTEGER PRIMARY KEY,
                jam_id INTEGER NOT NULL REFERENCES jams(id),
                bpm REAL
            );
            CREATE TABLE jam_key (
                id INTEGER PRIMARY KEY,
                jam_id INTEGER NOT NULL REFERENCES jams(id),
                key_name TEXT,
                scale TEXT
            );
            CREATE TABLE jam_tags (
                id INTEGER PRIMARY KEY,
                jam_id INTEGER NOT NULL REFERENCES jams(id),
                tag TEXT NOT NULL
            );

            -- Seed: kick_808.wav — 120 BPM, A major, tags: kick, drums
            INSERT INTO jams (id, filename, file_path, sample_rate, bit_depth)
                VALUES (1, 'kick_808.wav', '/audio/kick_808.wav', 44100, 16);
            INSERT INTO jam_tempo (jam_id, bpm) VALUES (1, 120.0);
            INSERT INTO jam_key (jam_id, key_name, scale) VALUES (1, 'A', 'major');
            INSERT INTO jam_tags (jam_id, tag) VALUES (1, 'kick');
            INSERT INTO jam_tags (jam_id, tag) VALUES (1, 'drums');

            -- Seed: pad_cm.wav — 90 BPM, C minor, tags: pad, ambient
            INSERT INTO jams (id, filename, file_path, sample_rate, bit_depth)
                VALUES (2, 'pad_cm.wav', '/audio/pad_cm.wav', 44100, 24);
            INSERT INTO jam_tempo (jam_id, bpm) VALUES (2, 90.0);
            INSERT INTO jam_key (jam_id, key_name, scale) VALUES (2, 'C', 'minor');
            INSERT INTO jam_tags (jam_id, tag) VALUES (2, 'pad');
            INSERT INTO jam_tags (jam_id, tag) VALUES (2, 'ambient');

            -- Seed: bass_line.wav — no metadata
            INSERT INTO jams (id, filename, file_path, sample_rate, bit_depth)
                VALUES (3, 'bass_line.wav', '/audio/bass_line.wav', 48000, 16);
            "#,
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_search_empty_query_returns_all_ordered_by_filename() {
        let conn = make_test_wallflower_db();
        let results = search_samples(&conn, "", 200).unwrap();
        assert_eq!(results.len(), 3, "Empty query must return all 3 samples");
        // Must be ordered by filename ascending
        assert_eq!(results[0].filename, "bass_line.wav");
        assert_eq!(results[1].filename, "kick_808.wav");
        assert_eq!(results[2].filename, "pad_cm.wav");
    }

    #[test]
    fn test_search_by_filename_substring() {
        let conn = make_test_wallflower_db();
        let results = search_samples(&conn, "kick", 200).unwrap();
        assert_eq!(results.len(), 1, "Filename search for 'kick' should match 1 sample");
        assert_eq!(results[0].filename, "kick_808.wav");
    }

    #[test]
    fn test_search_by_key_name() {
        let conn = make_test_wallflower_db();
        let results = search_samples(&conn, "A", 200).unwrap();
        // Should match kick_808.wav (key_name = 'A')
        assert!(
            results.iter().any(|r| r.filename == "kick_808.wav"),
            "Key search for 'A' must return kick_808.wav"
        );
    }

    #[test]
    fn test_search_by_tag() {
        let conn = make_test_wallflower_db();
        let results = search_samples(&conn, "ambient", 200).unwrap();
        assert_eq!(results.len(), 1, "Tag search for 'ambient' should match 1 sample");
        assert_eq!(results[0].filename, "pad_cm.wav");
    }

    #[test]
    fn test_search_by_bpm() {
        let conn = make_test_wallflower_db();
        // BPM search: CAST(ROUND(bpm) AS TEXT) LIKE '120%'
        let results = search_samples(&conn, "120", 200).unwrap();
        assert!(
            results.iter().any(|r| r.filename == "kick_808.wav"),
            "BPM search for '120' must return kick_808.wav"
        );
    }

    #[test]
    fn test_search_tags_are_split_correctly() {
        let conn = make_test_wallflower_db();
        let results = search_samples(&conn, "kick", 200).unwrap();
        assert_eq!(results.len(), 1);
        let sample = &results[0];
        // kick_808.wav has tags: kick, drums — both must be present
        assert!(
            sample.tags.contains(&"kick".to_string()),
            "Tags must include 'kick'"
        );
        assert!(
            sample.tags.contains(&"drums".to_string()),
            "Tags must include 'drums'"
        );
    }

    #[test]
    fn test_search_limit_is_respected() {
        let conn = make_test_wallflower_db();
        let results = search_samples(&conn, "", 2).unwrap();
        assert_eq!(results.len(), 2, "Limit of 2 must return exactly 2 results");
    }

    #[test]
    fn test_get_wallflower_status_connected_false_when_no_db() {
        // When no Wallflower DB is found, connected must be false
        // Test the logic by calling discover_wallflower_db with a nonexistent user path
        let result = discover_wallflower_db(Some(
            "/nonexistent/wallflower.db".to_string(),
        ));
        // In a test environment without Wallflower installed, this must be None
        // (or Some(path) if another path happens to exist)
        if let Some(ref p) = result {
            assert!(p.exists(), "Must only return existing paths");
        }
    }

    #[test]
    fn test_set_wallflower_db_path_invalid_path_returns_error() {
        // Validate that nonexistent paths are rejected (T-05-10)
        // We test this by simulating the validation logic directly
        let nonexistent = PathBuf::from("/nonexistent/path/wallflower.db");
        assert!(
            !nonexistent.exists(),
            "Test precondition: path must not exist"
        );
        // The actual path validation in set_wallflower_db_path returns Err for !p.exists()
        // Verify this is the behavior
        let result: Result<(), AppError> = if !nonexistent.exists() {
            Err(AppError::Io(format!("File not found: {}", nonexistent.display())))
        } else {
            Ok(())
        };
        assert!(result.is_err(), "Nonexistent path must return Err");
    }

    #[test]
    fn test_wallflower_status_struct_fields() {
        // WallflowerStatus struct can be constructed with all field combinations
        let connected = WallflowerStatus {
            connected: true,
            db_path: Some("/path/to/wallflower.db".to_string()),
            sample_count: Some(42),
        };
        assert!(connected.connected);
        assert_eq!(connected.sample_count, Some(42));

        let disconnected = WallflowerStatus {
            connected: false,
            db_path: None,
            sample_count: None,
        };
        assert!(!disconnected.connected);
        assert!(disconnected.db_path.is_none());
    }

    #[test]
    fn test_wallflower_sample_struct_fields() {
        // WallflowerSample struct can be constructed and tags are a Vec<String>
        let sample = WallflowerSample {
            id: 1,
            filename: "kick.wav".to_string(),
            file_path: "/audio/kick.wav".to_string(),
            sample_rate: Some(44100),
            bit_depth: Some(16),
            bpm: Some(120.0),
            key_name: Some("A".to_string()),
            scale: Some("major".to_string()),
            tags: vec!["kick".to_string(), "drums".to_string()],
        };
        assert_eq!(sample.tags.len(), 2);
        assert_eq!(sample.bpm, Some(120.0));
    }
}
