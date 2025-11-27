use std::{
    fs,
    sync::{Arc, Mutex},
};
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_log::log;
use ubi::UbiBuilder;

use crate::app_state::AppState;

mod app_state;
mod config;
mod install_components;
mod yt_dlp;

fn handle_emit_result(result: Result<(), tauri::Error>, kind: &str) {
    match result  {
        Ok(_) => log::debug!("Emitted Event to Frontend: {}", kind),
        Err(err) => log::error!("Failed to Emit Event to Frontend: {}", err),
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
                AppState::init()
                    .await
                    .expect("Failed to initialize app state")
            });

            let state = Arc::new(Mutex::new(state));
            app.manage(state.clone());

            let state_clone = state.clone();

            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    // Prevent premature application close.
                    api.prevent_close();

                    // Save in memory config to file.
                    let unlocked_state = state_clone.lock().unwrap();
                    let config = unlocked_state.get_config();
                    match toml::to_string(&config) {
                        Ok(config_as_str) => {
                            match fs::write(config::CONFIG_FILENAME, config_as_str) {
                                Ok(_) => {
                                    log::debug!("Saved {} to file.", config::CONFIG_FILENAME);
                                }
                                Err(err) => todo!(),
                            }
                        }
                        Err(err) => todo!(),
                    }

                    // Close application.
                    std::process::exit(0);
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            // App State and Config Handlers
            app_state::update_config,
            app_state::get_config,
            // YT-DLP Handlers
            install_components::install_ffmpeg,
            install_components::install_ytdlp,
            install_components::install_ffmpeg_ytdlp,
            yt_dlp::download_best_quality,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
