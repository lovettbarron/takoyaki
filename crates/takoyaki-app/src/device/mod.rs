use sysinfo::Disks;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tracing::info;

const POLL_INTERVAL_SECS: u64 = 2;

/// Check if a given path has the OT directory signature.
/// Real OT CF cards have varied layouts depending on firmware version and user config.
/// We detect by looking for known OT structural markers:
/// - A root-level Set folder containing project files (bank*.strd, project.strd)
/// - Or the PRESETS directory containing named Sets
/// - Or the classic AUDIO + SETS top-level layout
pub fn is_ot_volume(mount_point: &std::path::Path) -> bool {
    // Classic layout: top-level AUDIO + SETS
    if mount_point.join("AUDIO").is_dir() && mount_point.join("SETS").is_dir() {
        return true;
    }

    // Common layout: PRESETS directory at root containing Sets
    if mount_point.join("PRESETS").is_dir() {
        if has_ot_project_files(&mount_point.join("PRESETS")) {
            return true;
        }
    }

    // Any root-level directory containing OT project files (e.g. a Set folder)
    if let Ok(entries) = std::fs::read_dir(mount_point) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && has_ot_project_files(&path) {
                return true;
            }
        }
    }

    false
}

/// Check if a directory (or its immediate children) contains OT project files.
fn has_ot_project_files(dir: &std::path::Path) -> bool {
    // Direct project files in this dir
    if dir.join("project.strd").is_file() || dir.join("bank01.strd").is_file() {
        return true;
    }
    // One level deep (PRESETS/SetName/project.strd)
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.take(50).flatten() {
            let path = entry.path();
            if path.is_dir()
                && (path.join("project.strd").is_file() || path.join("bank01.strd").is_file())
            {
                return true;
            }
        }
    }
    false
}

/// Scan all removable disks for an OT volume.
/// Returns the mount point of the first OT volume found, or None.
///
/// Uses sysinfo Disks API to enumerate removable volumes.
/// Falls back to scanning /Volumes/* if sysinfo returns empty
/// (compatibility with macOS versions where is_removable may not work).
pub fn detect_ot_volume() -> Option<PathBuf> {
    let disks = Disks::new_with_refreshed_list();
    let disk_list: Vec<_> = disks.list().iter().collect();

    // First try: filter by is_removable
    for disk in &disk_list {
        if !disk.is_removable() {
            continue;
        }
        let mount = disk.mount_point();
        if is_ot_volume(mount) {
            return Some(mount.to_path_buf());
        }
    }

    // Fallback: scan all disk mount points (some macOS versions
    // don't report CF card readers as removable)
    for disk in &disk_list {
        let mount = disk.mount_point();
        // Skip system volumes
        if mount == std::path::Path::new("/")
            || mount.starts_with("/System")
            || mount.starts_with("/private")
        {
            continue;
        }
        if is_ot_volume(mount) {
            return Some(mount.to_path_buf());
        }
    }

    // Final fallback: direct /Volumes scan if sysinfo returned nothing useful
    if let Ok(entries) = std::fs::read_dir("/Volumes") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && is_ot_volume(&path) {
                return Some(path);
            }
        }
    }

    None
}

/// Background polling loop that checks for OT volume changes.
/// Updates AppState.device and emits "ot-device-changed" event to frontend when state changes.
/// Payload is Option<String> — Some(mount_path) when connected, None when disconnected.
pub async fn poll_loop(app: AppHandle) {
    info!(
        "Starting OT volume detection polling ({}s interval)",
        POLL_INTERVAL_SECS
    );
    let mut last_state: Option<PathBuf> = None;

    // Check immediately on startup (no initial delay)
    let initial = detect_ot_volume();
    if initial.is_some() {
        update_device_state(&app, &initial);
        let payload = initial.as_ref().map(|p| p.to_string_lossy().to_string());
        let _ = app.emit("ot-device-changed", &payload);
        last_state = initial;
    }

    loop {
        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;

        let current = detect_ot_volume();
        let changed = last_state != current;

        if changed {
            match &current {
                Some(path) => info!("OT volume detected: {}", path.display()),
                None => info!("OT volume disconnected"),
            }
            update_device_state(&app, &current);
            let payload = current.as_ref().map(|p| p.to_string_lossy().to_string());
            let _ = app.emit("ot-device-changed", &payload);
            last_state = current;
        }
    }
}

fn update_device_state(app: &AppHandle, volume: &Option<PathBuf>) {
    let state = app.state::<crate::AppState>();
    if let Ok(mut device) = state.device.lock() {
        device.mount_point = volume.clone();
        if volume.is_none() {
            device.confirmed = false;
        }
    };
}

/// Start the volume detection polling task.
/// Called from Tauri setup closure.
pub fn start_polling(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        poll_loop(app).await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_is_ot_volume_classic_layout() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("AUDIO")).unwrap();
        std::fs::create_dir_all(tmp.path().join("SETS")).unwrap();
        assert!(is_ot_volume(tmp.path()));
    }

    #[test]
    fn test_is_ot_volume_presets_layout() {
        let tmp = TempDir::new().unwrap();
        let set_dir = tmp.path().join("PRESETS").join("MySet");
        std::fs::create_dir_all(&set_dir).unwrap();
        std::fs::write(set_dir.join("project.strd"), b"fake").unwrap();
        assert!(is_ot_volume(tmp.path()));
    }

    #[test]
    fn test_is_ot_volume_root_set_folder() {
        let tmp = TempDir::new().unwrap();
        let set_dir = tmp.path().join("Sett");
        std::fs::create_dir_all(&set_dir).unwrap();
        std::fs::write(set_dir.join("bank01.strd"), b"fake").unwrap();
        assert!(is_ot_volume(tmp.path()));
    }

    #[test]
    fn test_is_ot_volume_negative_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(!is_ot_volume(tmp.path()));
    }

    #[test]
    fn test_is_ot_volume_negative_random_dirs() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("Documents")).unwrap();
        std::fs::create_dir_all(tmp.path().join("Music")).unwrap();
        assert!(!is_ot_volume(tmp.path()));
    }

    #[test]
    fn test_detect_does_not_panic() {
        let _ = detect_ot_volume();
    }
}
