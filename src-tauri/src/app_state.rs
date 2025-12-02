use std::{
    fs,
    sync::{Arc, Mutex},
};
use tauri::{Manager, Runtime, State};

use crate::config::{self, Config};

pub struct AppState {
    config: Config,
}

impl AppState {
    pub async fn init<R: Runtime>(app_handle: tauri::AppHandle<R>) -> Result<AppState, Box<dyn std::error::Error>> {
        let config = Self::handle_config(app_handle);

        Ok(AppState { config: config })
    }

    pub fn get_config(&self) -> Config {
        self.config.clone()
    }

    pub fn set_config(&mut self, new_config: Config) {
        self.config = new_config;
    }

    fn handle_config<R: Runtime>(
        app_handle: tauri::AppHandle<R>
    ) -> Config {
        let dir = app_handle.path().app_config_dir();
        
        let user_config = match dir {
            Ok(dir) => {
                fs::create_dir_all(&dir).unwrap();
                let file = dir.join(config::CONFIG_FILENAME);

                let file_data = fs::read(file);

                match file_data {
                    Ok(file_data) => {
                        let config: Config = serde_json::from_slice(&file_data).unwrap();

                        config
                    },
                    Err(err) => {
                        match err.kind() {
                            std::io::ErrorKind::NotFound => {
                                match serde_json::from_str(config::DEFAULT_CONFIG) {
                                    Ok(config) => {
                                        let config: Config = config;

                                        config
                                    },
                                    Err(err) => todo!("{}", err),
                                }
                            },
                            _ => todo!(),
                        }
                    },
                }
            },
            Err(err) => {
                match err {
                    // Handle config directory error - should be panic?
                    _ => todo!()
                }
            },
        };

        user_config
    }
}

#[tauri::command]
pub fn get_config(state: State<'_, Arc<Mutex<AppState>>>) -> Option<Config> {
    let state = state.lock();

    let state = match state {
        Ok(state) => state,
        Err(_) => return None,
    };

    Some(state.get_config())
}

#[tauri::command]
pub fn update_config(state: State<'_, Arc<Mutex<AppState>>>, updated_config: Config) -> bool {
    let state = state.lock();

    let mut state = match state {
        Ok(state) => state,
        Err(_) => return false,
    };

    state.set_config(updated_config);

    true
}
