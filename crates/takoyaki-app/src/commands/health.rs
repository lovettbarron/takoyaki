//! Tauri command for running the background health check (DETC-01, DETC-02, DETC-03).
//!
//! Threat model:
//! - T-02-05: Path traversal prevention in health::resolve_ot_path (canonicalize check).
//! - T-02-06: Frontend supplies project_id (opaque UUID), not a raw path. Backend resolves
//!   card_path from SQLite DB via db::projects::get_card_path — user never controls raw paths.

use crate::db;
use crate::error::AppError;
use crate::AppState;
use tauri::Emitter;

/// Trigger the health check for a project.
///
/// Returns `Ok(())` immediately after spawning the background task — never blocks
/// on file I/O (RESEARCH.md Pitfall 2: "Health Check Blocking the UI Thread").
///
/// Results are emitted as a "health-complete" event carrying a `HealthCheckComplete`
/// payload when the background scan finishes. The frontend listens for this event
/// and stores results in react-query cache.
///
/// Threat model T-02-06: `project_id` is resolved to `card_path` via SQLite — the
/// frontend never supplies a raw file path.
#[tauri::command]
#[specta::specta]
pub async fn run_health_check(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> Result<(), AppError> {
    // 1. Grab project path and volume path from state, then DROP the locks.
    //    All file I/O happens outside the lock (RESEARCH.md SQLite Lock Pattern).
    let (project_path, volume_path) = {
        let card_path = {
            let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
            db::projects::get_card_path(&db.conn, &project_id)
                .map_err(|e| AppError::Database(e.to_string()))?
        };
        // DB lock is dropped here.

        let vol = {
            let device = state.device.lock().map_err(|e| AppError::Lock(e.to_string()))?;
            device
                .mount_point
                .clone()
                .ok_or_else(|| AppError::Io("No OT volume mounted".to_string()))?
        };
        // Device lock is dropped here.

        (card_path, vol)
    };

    // 2. Spawn background task — return Ok(()) immediately so the UI is never blocked.
    //    The spawned task owns all data it needs (project_path, volume_path, project_id, app).
    tauri::async_runtime::spawn(async move {
        // Build stub slot list from project_path.
        // FIXME: Phase 1 OT project.work parser not yet implemented.
        // When available, replace this stub with:
        //   let data = std::fs::read(project_path.join("project.work"))?;
        //   let project = ot_parser::ProjectFile::from_bytes(&data)?;
        //   build real SlotCheckInput list from project.flex_sample_slots / static_sample_slots
        //   and cross-reference track references from all 16 bank files.
        let slot_inputs: Vec<crate::health::SlotCheckInput> = Vec::new();

        let issues = crate::health::perform_health_check(
            &project_path,
            &volume_path,
            &slot_inputs,
        )
        .await;

        // Format scanned_at timestamp without pulling in chrono dependency.
        // Uses std::time::SystemTime formatted as ISO 8601.
        let scanned_at = {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            format_iso8601(secs)
        };

        let result = crate::health::HealthCheckComplete {
            project_id,
            issues,
            scanned_at,
        };

        app.emit("health-complete", result).unwrap_or_else(|e| {
            tracing::error!("health-complete emit failed: {e}");
        });
    });

    Ok(())
}

/// Format a Unix timestamp as an ISO 8601 datetime string.
///
/// Avoids pulling in `chrono` for a simple timestamp conversion.
/// Format: "YYYY-MM-DDTHH:MM:SSZ"
fn format_iso8601(secs: u64) -> String {
    let days_total = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let mut year = 1970u64;
    let mut remaining_days = days_total;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let months = if is_leap_year(year) {
        [31u64, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u64;
    for &days_in_month in &months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }

    let day = remaining_days + 1;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
