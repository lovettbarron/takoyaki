mod atomic;
mod commands;
pub mod db;
pub mod device;
mod error;

pub use error::{AppError, Result};

use std::path::PathBuf;
use std::sync::Mutex;
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
    };

    tauri::Builder::default()
        .setup(|app| {
            device::start_polling(app.handle().clone());
            Ok(())
        })
        .manage(app_state)
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
