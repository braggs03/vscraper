use std::{fs, sync::{Arc, Mutex}};

use serde_json::Value;
use tauri::State;
use toml::{Table, de::Error};

use crate::config::{self, Config};

pub struct AppState {
    config: Config,
}

impl AppState {
    pub async fn init() -> Result<AppState, Box<dyn std::error::Error>> {

        let config = Self::handle_config();

        Ok(AppState {
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
            Ok(c) =>{ 
                let mut values = c.parse::<Table>().unwrap();
                let default_values = config::DEFAULT_CONFIG.parse::<Table>().unwrap();

                default_values.iter().for_each(|(key, value)| {
                    if !values.contains_key(key) {
                        values.insert(key.to_owned(), value.to_owned());
                    }
                });

                toml::from_str(&values.to_string())

            },
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