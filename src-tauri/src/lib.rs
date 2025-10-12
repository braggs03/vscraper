// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use core::panic;
use std::{
    fs,
    sync::{Arc, Mutex},
};
use tauri::{Manager, State, WindowEvent};

use serde::{Deserialize, Serialize};
use turso::Database;

const DEFAULT_CONFIG_FILE_LOCATION: &str = "default_config.toml";

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

        let user_config_str = match fs::read_to_string(filename) {
            Ok(c) => c,
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => {
                    let default_config = fs::read_to_string(DEFAULT_CONFIG_FILE_LOCATION);
                    match default_config {
                        Ok(default_config) => default_config,
                        Err(error) => {
                            panic!("Go fuck yourself, you messed with the default_config.toml. Heres the error: {}", error);
                        }
                    }
                }
                _ => {
                    panic!("File could not be read! Heres the error: {}", error);
                }
            },
        };

        let user_config: Config = match toml::from_str(&user_config_str) {
            Ok(d) => d,
            Err(_) => {
                todo!("Error handling on config failure.")
            }
        };

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

            // Do async state init in a block
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
