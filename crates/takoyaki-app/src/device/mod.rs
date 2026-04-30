use sysinfo::Disks;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tracing::info;

const OT_SIGNATURE_DIRS: &[&str] = &["AUDIO", "SETS"];
const POLL_INTERVAL_SECS: u64 = 2;

/// Check if a given path has the OT directory signature.
/// An OT volume has both /AUDIO and /SETS directories at its root.
pub fn is_ot_volume(mount_point: &std::path::Path) -> bool {
    OT_SIGNATURE_DIRS.iter().all(|d| mount_point.join(d).is_dir())
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
/// Emits "ot-device-changed" event to frontend when state changes.
/// Payload is Option<String> — Some(mount_path) when connected, None when disconnected.
pub async fn poll_loop(app: AppHandle) {
    info!(
        "Starting OT volume detection polling ({}s interval)",
        POLL_INTERVAL_SECS
    );
    let mut last_state: Option<PathBuf> = None;
    loop {
        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;

        let current = detect_ot_volume();
        let changed = last_state != current;

        if changed {
            match &current {
                Some(path) => info!("OT volume detected: {}", path.display()),
                None => info!("OT volume disconnected"),
            }
            let payload = current.as_ref().map(|p| p.to_string_lossy().to_string());
            let _ = app.emit("ot-device-changed", &payload);
            last_state = current;
        }
    }
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
    fn test_is_ot_volume_positive() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("AUDIO")).unwrap();
        std::fs::create_dir_all(tmp.path().join("SETS")).unwrap();
        assert!(is_ot_volume(tmp.path()));
    }

    #[test]
    fn test_is_ot_volume_negative_missing_audio() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("SETS")).unwrap();
        assert!(!is_ot_volume(tmp.path()));
    }

    #[test]
    fn test_is_ot_volume_negative_missing_sets() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("AUDIO")).unwrap();
        assert!(!is_ot_volume(tmp.path()));
    }

    #[test]
    fn test_is_ot_volume_negative_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(!is_ot_volume(tmp.path()));
    }

    #[test]
    fn test_detect_does_not_panic() {
        // detect_ot_volume should never panic regardless of system state
        let _ = detect_ot_volume();
    }
}
