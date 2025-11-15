use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;

pub const CONFIG_FILENAME: &str = "user_config.toml";
pub const DEFAULT_CONFIG: &str = 
"
skip_homepage = false
";

#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    skip_homepage: bool,
}