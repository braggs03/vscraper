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
            handle_install_result(&app_handle, &destination, "ffmpeg", ffmpeg_install_result);

            // YT-DLP Download and Installation
            log::info!("Starting YT-DLP Installation.");
            let yt_dlp_install_result = installer.install_youtube(None).await;
            handle_install_result(&app_handle, &destination, "yt-dlp", yt_dlp_install_result);
        });
    });

    Ok(())
}

fn handle_install_result(app_handle: &tauri::AppHandle, destination: &PathBuf, kind: &str, result: yt_dlp::error::Result<PathBuf>) {
    match result {
        Ok(_) => { 
            log::info!("Finished Downloading and Installing {}.", kind); 
            match app_handle.emit(&format!("{}_install", kind), true) {
                Ok(_) => log::debug!("Successfully Emitted Event to Frontend."),
                Err(err) => log::error!("Failed to Emit Event to Frontend: {}", err),
            }
        },
        Err(err) => { 
            log::error!("Failed to Download and/or install {}: {}.", kind, err); 
            log::error!("Attempting to Clean Install Path For {}.", kind);
            match app_handle.emit(&format!("{}_install", kind), false) {
                Ok(_) => log::debug!("Successfully Emitted Event to Frontend."),
                Err(err) => log::error!("Failed to Emit Event to Frontend: {}", err),
            }
            cleared_failed_download(destination, kind);
        },
    }
}

fn cleared_failed_download(libs_path: &PathBuf, kind: &str) {
    let paths = fs::read_dir(libs_path);
    match paths {
        Ok(paths) => {
            for dir in paths {
                match dir {
                    Ok(dir) => {
                        match dir.file_name().to_str() {
                            Some(file_name) => {
                                if file_name.contains(kind) {
                                    let path = dir.path();
                                    let result = if path.is_dir() {
                                        fs::remove_dir_all(&path)
                                    } else {
                                        fs::remove_file(&path)
                                    };

                                    match result {
                                        Ok(_) => log::trace!("Successfully Removed: {}", file_name),
                                        Err(_) => log::debug!("Failed to Remove: {}", file_name),
                                    }
                                }
                            },
                            None => { log::error!("File Name Error.", ); },
                        }
                    },
                    Err(err) => {
                        log::error!("Error Getting Directory/File: {}", err);
                    },
                }
            }
        },
        Err(err) => { log::error!("Failed to Get libs Path Directory Path: {}", err); },
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
                    let config = state_clone.lock().unwrap();
                    let config_str = toml::to_string(&config.get_config()).unwrap();
                    fs::write("user_config.toml", config_str).unwrap();
                    log::debug!("Successfully saved user_config.toml to file.");

                    // Allow closure of application.
                    std::process::exit(0);
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            // Config Handlers
            config::get_config,
            app_state::update_config,

            // YT-DLP Handlers 
            install_yt_dlp_ffmpeg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
