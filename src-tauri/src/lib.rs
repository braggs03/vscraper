use core::panic;
use std::{
    fs, path::PathBuf, sync::{Arc, Mutex}
};
use tauri::{Emitter, Manager, State, WindowEvent};
use serde::{Deserialize, Serialize};
use tauri_plugin_log::log;
use turso::Database;
use yt_dlp::fetcher::deps::LibraryInstaller;

use crate::app_state::AppState;

mod app_state;
mod config;

#[tauri::command]
fn install_yt_dlp_ffmpeg(app_handle: tauri::AppHandle) -> Result<(), ()> {
    std::thread::spawn(move || {
        let destination = PathBuf::from("libs");
        let installer = LibraryInstaller::new(destination.clone());
        tauri::async_runtime::block_on(async {

            // FFMPEG Download and Installation
            log::info!("Starting FFMPEG Installation.");
            let ffmpeg_install_result = installer.install_ffmpeg(None).await;
            handle_install_result(&app_handle, "ffmpeg", ffmpeg_install_result);

            // YT-DLP Download and Installation
            log::info!("Starting YT-DLP Installation.");
            let yt_dlp_install_result = installer.install_youtube(None).await;
            handle_install_result(&app_handle, "yt-dlp", yt_dlp_install_result);
        });
    });

    Ok(())
}

fn handle_install_result(app_handle: &tauri::AppHandle, kind: &str, install_result: yt_dlp::error::Result<PathBuf>) {
    match &install_result {
        Ok(_) => { 
            log::info!("Finished Installing {}.", kind); 
        },
        Err(err) => { 
            log::error!("Failed to Install {}: {}.", kind, err); 
        },
    }

    match app_handle.emit(&format!("{}_install", kind), install_result.is_ok()) {
        Ok(_) => log::debug!("Emitted Event to Frontend: {}_install", kind),
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

            // Clone for the event handler
            let state_clone = state.clone();

            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    // Prevent premature application close.
                    api.prevent_close();

                    // Save in memory config to file.
                    let unlocked_state =  state_clone.lock().unwrap();
                    let config = unlocked_state.get_config();
                    match toml::to_string(&config) {
                        Ok(config_as_str) => {
                            match fs::write(config::CONFIG_FILENAME, config_as_str) {
                                Ok(_) => {
                                    log::debug!("Saved {} to file.", config::CONFIG_FILENAME);
                                },
                                Err(err) => {
                                    
                                },
                            }
                        },
                        Err(err) => {

                        },
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
            install_yt_dlp_ffmpeg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
