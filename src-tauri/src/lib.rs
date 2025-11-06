// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use core::panic;
use std::{
    error::Error, fs, path::PathBuf, sync::{Arc, Mutex}
};
use tauri::{Manager, State, WindowEvent, async_runtime};

use serde::{Deserialize, Serialize};
use tauri_plugin_log::log;
use turso::Database;
use yt_dlp::{Youtube, fetcher::deps::LibraryInstaller};

const DEFAULT_CONFIG: &str = 
"
skip_homepage = false
";

struct AppState {
    db: Database,
    config: Config,
}

#[derive(Clone, Deserialize, Serialize)]
struct Config {
    skip_homepage: bool,
}

impl AppState {
    async fn init() -> Result<AppState, Box<dyn std::error::Error>> {
        let db = turso::Builder::new_local("sqlite.db").build().await?;

        let config = Self::handle_config();

        Ok(AppState {
            db: db,
            config: config,
        })
    }

    fn handle_config() -> Config {
        let filename = "user_config.toml";

        let user_config = match fs::read_to_string(filename) {
            Ok(c) => toml::from_str(&c),
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => {
                    toml::from_str(DEFAULT_CONFIG)
                }
                _ => {
                    panic!("Config Error: {}", error);
                }
            },
        }.expect("Config Error, could not deserialized from toml.");

        user_config
    }
}

#[tauri::command]
fn get_config(state: State<'_, Arc<Mutex<AppState>>>) -> Option<Config> {
    let state = state.lock();

    let state = match state {
        Ok(state) => state,
        Err(_) => return None,
    };

    Some(state.config.clone())
}

#[tauri::command]
fn install_yt_dlp_ffmpeg() -> Result<(), ()> {
    std::thread::spawn(|| {
        let destination = PathBuf::from("libs");
        let installer = LibraryInstaller::new(destination.clone());
        tauri::async_runtime::block_on(async {
            log::info!("Starting FFMPEG Installation.");

            let ffmpeg_install_result = installer.install_ffmpeg(None).await;

            match ffmpeg_install_result {
                Ok(_) => { log::info!("Finished Downloading and Installing FFMPEG."); },
                Err(err) => { 
                    log::error!("Failed to Download and/or install FFMPEG: {}.", err); 
                    log::error!("Attempting to Clean Install Path For FFMPEG.");
                    cleared_failed_download(&destination, "ffmpeg");
                },
            }

            log::info!("Starting YT-DLP Installation.");

            let yt_dlp_install_result = installer.install_youtube(None).await;

            match yt_dlp_install_result {
                Ok(_) => { log::info!("Finished Downloading and Installing YT-DLP."); },
                Err(err) => { 
                    log::error!("Failed to Download and/or install YT-DLP: {}.", err); 
                    log::error!("Attempting to Clean Install Path For YT-DLP.");
                    cleared_failed_download(&destination, "yt-dlp");
                },
            }
        });
    });

    Ok(())
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

#[tauri::command]
fn set_homepage_preference(state: State<'_, Arc<Mutex<AppState>>>, preference: bool) -> bool {
    let state = state.lock();

    let mut state = match state {
        Ok(state) => state,
        Err(_) => return false,
    };

    state.config.skip_homepage = preference;

    true
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
            app.manage(state.clone()); // ✅ Register the state here

            // Clone for the event handler
            let state_clone = state.clone();

            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    // Prevent premature application close.
                    api.prevent_close();

                    // Save in memory config to file.
                    let config = state_clone.lock().unwrap();
                    let config_str = toml::to_string(&config.config).unwrap();
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
            get_config,
            set_homepage_preference,
            install_yt_dlp_ffmpeg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
