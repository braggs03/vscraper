use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;

#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    skip_homepage: bool,
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