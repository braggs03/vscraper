use std::{fs, sync::{Arc, Mutex}};

use tauri::State;
use turso::Database;

use crate::config::{self, Config};

pub struct AppState {
    db: Database,
    config: Config,
}

impl AppState {
    pub async fn init() -> Result<AppState, Box<dyn std::error::Error>> {
        let db = turso::Builder::new_local("sqlite.db").build().await?;

        let config = Self::handle_config();

        Ok(AppState {
            db: db,
            config: config,
        })
    }

    pub fn get_config(&self) -> Config {
        self.config.clone()
    }

    pub fn set_config(&mut self, new_config: Config) {
        self.config = new_config;
    } 

    fn handle_config() -> Config {
        let user_config = match fs::read_to_string(config::CONFIG_FILENAME) {
            Ok(c) => toml::from_str(&c),
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => {
                    toml::from_str(config::DEFAULT_CONFIG)
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