use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};


pub const CONFIG_FILENAME: &str = "user_config.toml";
pub const DEFAULT_CONFIG: &str = 
"
skip_homepage = false
ffmpeg_path = \"./libs/ffmpeg\"
ytdlp_path = \".libs/yt-dlp\"
";

#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    ffmpeg_path: String,
    skip_homepage: bool,
    ytdlp_path: String,
}