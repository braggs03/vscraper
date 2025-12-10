use std::{
    collections::HashMap,
    fs, sync::{Arc},
};
use tauri::{Manager, Runtime, State, async_runtime::Sender};
use tokio::sync::Mutex;

use crate::config::{self, Config};

pub struct AppState {
    config: Config,
    current_downloads: HashMap<String, Sender<()>>,
}

impl AppState {
    pub async fn init<R: Runtime>(
        app_handle: tauri::AppHandle<R>,
    ) -> Result<AppState, Box<dyn std::error::Error>> {
        Ok(AppState {
            config: Self::handle_config(app_handle),
            current_downloads: HashMap::new(),
        })
    }
    
    pub fn get_config(&self) -> Config {
        self.config.clone()
    }

    pub fn set_skip_homepage(&mut self, new_preference: bool) {
        self.config.set_skip_homepage(new_preference);
    }

    pub fn add_download(&mut self, url: String, sender: Sender<()>) -> bool {
        match self.current_downloads.contains_key(&url) {
            true => false,
            false => {
                self.current_downloads.insert(url, sender);
                true
            },
        }
    }

    pub fn get_download(&self, url: &str) -> Option<&Sender<()>> {
        self.current_downloads.get(url)
    }

    fn handle_config<R: Runtime>(app_handle: tauri::AppHandle<R>) -> Config {
        let dir = app_handle.path().app_config_dir();
        let user_config = match dir {
            Ok(dir) => match fs::create_dir_all(&dir) {
                Ok(_) => {
                    let file = dir.join(config::CONFIG_FILENAME);
                    let file_data = fs::read(file);
                    match file_data {
                        Ok(file_data) => serde_json::from_slice(&file_data).unwrap(),
                        Err(err) => match err.kind() {
                            std::io::ErrorKind::NotFound => Config::default(),
                            err => todo!("unknown potential errors: {}", err),
                        },
                    }
                }
                Err(err) => todo!("unable to get create directory: {}", err),
            },
            Err(err) => {
                // Handle config directory error - should be panic?
                todo!("unable to get config directory: {}", err)
                // panic!("unable to get config directory: {}", err)
            }
        };
        user_config
    }
}

#[tauri::command]
pub async fn get_config(state: State<'_, Arc<Mutex<AppState>>>) -> tauri::Result<Config> {
    let state = state.lock().await;
    Ok(state.get_config())
}

#[tauri::command]
pub async fn update_skip_homepage(state: State<'_, Arc<Mutex<AppState>>>, updated_preference: bool) -> tauri::Result<()> {
    Ok(state.lock().await.set_skip_homepage(updated_preference))
}
