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
        // Read project.work to build real slot inputs (Phase 7: replaces empty stub)
        let raw = std::fs::read(format!("{}/project.work", project_path))
            .or_else(|_| std::fs::read(format!("{}/project.strd", project_path)))
            .unwrap_or_default();

        let parsed = crate::commands::samples::parse_project_work(&raw);

        let mut slot_inputs: Vec<crate::health::SlotCheckInput> = Vec::new();

        for (idx, path_opt) in parsed.flex_slots.iter().enumerate() {
            slot_inputs.push(crate::health::SlotCheckInput {
                slot_type: "flex".to_string(),
                slot_index: idx as u8,
                occupied: path_opt.is_some(),
                raw_path: path_opt.clone(),
                track_references: vec![], // Bank body opaque -- DETC-03 limitation
            });
        }
        for (idx, path_opt) in parsed.static_slots.iter().enumerate() {
            slot_inputs.push(crate::health::SlotCheckInput {
                slot_type: "static".to_string(),
                slot_index: idx as u8,
                occupied: path_opt.is_some(),
                raw_path: path_opt.clone(),
                track_references: vec![], // Bank body opaque -- DETC-03 limitation
            });
        }

        tracing::info!(
            "run_health_check: built {} slot inputs ({} occupied)",
            slot_inputs.len(),
            slot_inputs.iter().filter(|s| s.occupied).count(),
        );

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
