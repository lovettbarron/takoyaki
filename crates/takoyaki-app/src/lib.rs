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

/// Messages sent to the dedicated audio thread.
pub enum AudioCommand {
    Play(PathBuf),
    Stop,
}

/// Application-wide state managed by Tauri.
pub struct AppState {
    pub db: Mutex<db::Database>,
    pub device: Mutex<DeviceState>,
    pub cancel_backup: Arc<AtomicBool>,
    pub audio_tx: std::sync::mpsc::Sender<AudioCommand>,
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
        commands::samples::compute_sample_dry_run,
        commands::samples::assign_sample,
        commands::samples::play_sample,
        commands::samples::stop_sample,
        commands::wallflower::get_wallflower_status,
        commands::wallflower::search_wallflower_samples,
        commands::wallflower::set_wallflower_db_path,
    ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/bindings.ts",
        )
        .expect("Failed to export TypeScript bindings");

    let (audio_tx, audio_rx) = std::sync::mpsc::channel::<AudioCommand>();

    // Dedicated audio thread — owns the OutputStream (which is !Send)
    std::thread::spawn(move || {
        let (_stream, stream_handle) = match rodio::OutputStream::try_default() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to open audio output: {}", e);
                return;
            }
        };
        let mut sink: Option<rodio::Sink> = None;

        for cmd in audio_rx {
            match cmd {
                AudioCommand::Play(path) => {
                    // Stop previous playback
                    if let Some(ref s) = sink {
                        s.stop();
                    }

                    let file = match std::fs::File::open(&path) {
                        Ok(f) => f,
                        Err(e) => {
                            tracing::error!("Failed to open audio file: {}", e);
                            continue;
                        }
                    };
                    let source = match rodio::Decoder::new(std::io::BufReader::new(file)) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("Failed to decode audio: {}", e);
                            continue;
                        }
                    };
                    let new_sink = match rodio::Sink::try_new(&stream_handle) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("Failed to create audio sink: {}", e);
                            continue;
                        }
                    };
                    new_sink.append(source);
                    new_sink.play();
                    sink = Some(new_sink);
                }
                AudioCommand::Stop => {
                    if let Some(ref s) = sink {
                        s.stop();
                    }
                    sink = None;
                }
            }
        }
    });

    let app_state = AppState {
        db: Mutex::new(db::Database::open_in_memory().expect("Failed to open database")),
        device: Mutex::new(DeviceState {
            mount_point: None,
            confirmed: false,
        }),
        cancel_backup: Arc::new(AtomicBool::new(false)),
        audio_tx,
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
