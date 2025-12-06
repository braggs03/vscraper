use std::{
    fs, sync::{Arc, Mutex},
};
use tauri::{Manager, State, WindowEvent};
use tauri_plugin_log::log::{self, error};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

use crate::app_state::AppState;

mod app_state;
mod config;
mod components;
mod emissions;
mod ytdlp;

fn handle_emit_result(result: Result<(), tauri::Error>, kind: &str) {
    match result {
        Ok(_) => log::debug!("emitted event to frontend: {}", kind),
        Err(err) => log::error!("failed to emit event to frontend: {}", err),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app
                .get_webview_window("main")
                .expect("Failed to get main window");

            let state = tauri::async_runtime::block_on(async {
                AppState::init(app.app_handle().clone())
                    .await
                    .expect("Failed to initialize app state")
            });

            let state = Arc::new(Mutex::new(state));
            app.manage(state.clone());

            let app_handle = app.app_handle().clone();

            window.on_window_event(move |event| {
                match event {
                    WindowEvent::CloseRequested { api, .. } => {
                        // Prevent premature application close.
                        api.prevent_close();

                        let close_state: State<'_, Arc<Mutex<AppState>>> = app_handle.state();

                        // Save in memory config to file.
                        let config_dir = app_handle.path().app_config_dir();
                        let config = close_state.lock().unwrap().get_config();

                        match config_dir {
                            Ok(config_dir) => match serde_json::to_string_pretty(&config) {
                                Ok(config_as_str) => {
                                    let config_file_location =
                                        config_dir.join(config::CONFIG_FILENAME);
                                    match fs::write(&config_file_location, &config_as_str) {
                                        Ok(_) => {
                                            log::debug!(
                                                "saved {} to file.",
                                                config::CONFIG_FILENAME
                                            );
                                        }
                                        Err(err) => match err.kind() {
                                            std::io::ErrorKind::NotFound => {
                                                error!("config dir not found: {}", err);
                                            }
                                            _ => error!("generic: {}", err),
                                        },
                                    }
                                }
                                Err(err) => {
                                    error!("error saving config to string: {}", err);
                                }
                            },
                            Err(err) => {
                                error!("retrieving config dir: {}", err);
                            }
                        }

                        if let Err(err) = app_handle.save_window_state(StateFlags::all()) {
                            error!("failed to save windows state: {}", err);
                        }

                        // Close application.
                        std::process::exit(0);
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("logs".to_string()),
                    },
                ))
                .build(),
        )
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            // App State and Config Handlers
            app_state::update_config,
            app_state::get_config,
            // YT-DLP Handlers
            components::install_ytdlp,
            components::install_ffmpeg_ytdlp,
            ytdlp::download_best_quality,
            ytdlp::download_from_custom_format,
            ytdlp::cancel_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
