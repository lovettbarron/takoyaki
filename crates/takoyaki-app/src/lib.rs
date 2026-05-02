pub mod atomic;
mod commands;
pub mod db;
pub mod device;
mod error;
pub mod health;
pub mod management;

pub use error::{AppError, Result};

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use tauri::Manager;
use tauri_specta::collect_commands;

/// The state of the connected OT device.
pub struct DeviceState {
    pub mount_point: Option<PathBuf>,
    pub confirmed: bool,
}

/// Application-wide state managed by Tauri.
pub struct AppState {
    pub db: Mutex<db::Database>,
    pub device: Mutex<DeviceState>,
    pub cancel_backup: Arc<AtomicBool>,
}

pub fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::projects::list_projects,
        commands::projects::get_project_detail,
        commands::projects::get_project_banks,
        commands::projects::index_ot_projects,
        commands::samples::get_project_samples,
        commands::device::get_device_status,
        commands::device::confirm_device,
        commands::device::dismiss_device,
        commands::health::run_health_check,
        commands::backup::backup_project,
        commands::backup::restore_snapshot,
        commands::backup::compute_dry_run,
        commands::backup::list_backups,
        commands::backup::cancel_backup,
        commands::management::compute_management_dry_run,
        commands::management::duplicate_project,
        commands::management::rename_project,
        commands::management::export_project,
        commands::management::copy_bank,
    ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/bindings.ts",
        )
        .expect("Failed to export TypeScript bindings");

    let app_state = AppState {
        db: Mutex::new(db::Database::open_in_memory().expect("Failed to open database")),
        device: Mutex::new(DeviceState {
            mount_point: None,
            confirmed: false,
        }),
        cancel_backup: Arc::new(AtomicBool::new(false)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            device::start_polling(app.handle().clone());

            // D-12: Clean up incomplete backups from prior interrupted sessions
            let cleanup_state = app.state::<AppState>();
            if let Ok(mut db) = cleanup_state.db.lock() {
                if let Ok(incomplete_paths) = db::backups::cleanup_incomplete_backups(&mut db.conn) {
                    for path in incomplete_paths {
                        let _ = std::fs::remove_dir_all(&path);
                        tracing::info!("Cleaned up incomplete backup: {}", path);
                    }
                }
            }

            Ok(())
        })
        .manage(app_state)
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
