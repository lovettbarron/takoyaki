//! Read-only connection to Wallflower's SQLite database.
//!
//! Threat model T-01-09: Uses SQLITE_OPEN_READ_ONLY flag — any write attempt
//! on this connection will fail at the driver level, not just by convention.

use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use crate::error::AppError;

/// Open Wallflower's SQLite database in read-only mode.
///
/// Uses `SQLITE_OPEN_READ_ONLY` flag — any write attempt will fail at the
/// driver level with `SQLITE_READONLY`. The file must already exist.
///
/// Returns `Err(AppError::Io)` if the path does not exist, since SQLite in
/// read-only mode cannot create a new file.
pub fn open_wallflower_db(path: &Path) -> Result<Connection, AppError> {
    if !path.exists() {
        return Err(AppError::Io(format!(
            "Wallflower database not found at: {}",
            path.display()
        )));
    }
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    Ok(conn)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallflower_db_nonexistent_returns_err() {
        let result = open_wallflower_db(Path::new("/nonexistent/path/wallflower.db"));
        assert!(result.is_err(), "Non-existent path must return Err");
    }

    #[test]
    fn test_wallflower_db_opens_readonly() {
        // Create a temporary valid SQLite file
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            conn.execute_batch("CREATE TABLE test (id INTEGER PRIMARY KEY);")
                .unwrap();
        }
        // Open read-only — must succeed
        let conn = open_wallflower_db(&path).unwrap();
        // Write attempt must fail at driver level
        let result = conn.execute_batch("CREATE TABLE test2 (id INTEGER PRIMARY KEY);");
        assert!(
            result.is_err(),
            "Write to SQLITE_OPEN_READ_ONLY connection must return an error"
        );
    }
}
