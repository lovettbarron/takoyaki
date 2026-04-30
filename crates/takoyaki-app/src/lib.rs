mod commands;
pub mod db;
mod error;

pub use error::{AppError, Result};

use commands::projects::AppState;
use std::sync::Mutex;
use tauri_specta::collect_commands;

pub fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::projects::list_projects,
        commands::projects::get_project_detail,
        commands::projects::get_project_banks,
        commands::projects::index_ot_projects,
        commands::samples::get_project_samples,
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
        volume_path: Mutex::new(None),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
