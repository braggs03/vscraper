use core::panic;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{Emitter, Manager, State, WindowEvent};
use tauri_plugin_log::log;
use turso::Database;
use ubi::UbiBuilder;

use crate::app_state::AppState;

mod app_state;
mod config;
mod yt_dlp;

fn detect_arch() -> &'static str {
    // Map Rust target_arch to the naming convention used by johnvansickle.com
    // This site uses: arm64, amd64, i686, armhf, etc.
    match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "arm" => "armhf",
        "i686" => "i686",
        other => panic!("Unsupported arch: {other}"),
    }
}

#[tauri::command]
fn install_ytdlp(app_handle: tauri::AppHandle) -> bool {
    let success = tauri::async_runtime::block_on(async {
        let ubi = UbiBuilder::new()
            .project("yt-dlp/yt-dlp")
            .install_dir("./libs")
            .build();
        
        let result = match ubi {
            Ok(mut ubi) => ubi.install_binary().await,
            Err(err) => Err(err),
        };

        let emit_status = app_handle.emit(&format!("yt-dlp_install"), result.is_ok());

        handle_emit_result(emit_status, "yt-dlp_install");

        result
    });

    success.is_ok()
}

#[tauri::command]
fn install_ffmpeg(app_handle: tauri::AppHandle) -> bool {
    let arch = detect_arch();
    let url = format!("https://johnvansickle.com/ffmpeg/builds/ffmpeg-git-{arch}-static.tar.xz");

    let success = tauri::async_runtime::block_on(async {
        let ubi = UbiBuilder::new()
            .url(&url)
            .install_dir("./libs")
            .exe("ffmpeg")
            .build();

        let result = match ubi {
            Ok(mut ubi) => ubi.install_binary().await,
            Err(err) => Err(err),
        };

        let emit_status = app_handle.emit(&format!("ffmpeg_install"), result.is_ok());

        handle_emit_result(emit_status, "ffmpeg_install");

        result
    });

    success.is_ok()
}

#[tauri::command]
fn install_ffmpeg_ytdlp(app_handle: tauri::AppHandle) {
    install_ffmpeg(app_handle.clone());
    install_ytdlp(app_handle);
}

fn handle_emit_result(result: Result<(), tauri::Error>, kind: &str) {
    match result  {
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
                    let unlocked_state = state_clone.lock().unwrap();
                    let config = unlocked_state.get_config();
                    match toml::to_string(&config) {
                        Ok(config_as_str) => {
                            match fs::write(config::CONFIG_FILENAME, config_as_str) {
                                Ok(_) => {
                                    log::debug!("Saved {} to file.", config::CONFIG_FILENAME);
                                }
                                Err(err) => {}
                            }
                        }
                        Err(err) => {}
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
            install_ffmpeg,
            install_ytdlp,
            install_ffmpeg_ytdlp,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
