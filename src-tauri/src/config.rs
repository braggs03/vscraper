use serde::{Deserialize, Serialize};


pub const CONFIG_FILENAME: &str = "settings.json";
pub const DEFAULT_CONFIG: &str = 
"
{
    \"skip_homepage\": false,
    \"ffmpeg_path\": \"./libs/ffmpeg\",
    \"ytdlp_path\": \".libs/yt-dlp\"
}
";

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Config {
    
    #[serde(default = "default_ffmpeg_path")]
    ffmpeg_path: String,

    #[serde(default)]
    skip_homepage: bool,

    #[serde(default = "default_ytdlp_path")]
    ytdlp_path: String,
}

fn default_ffmpeg_path() -> String {
    String::from("./libs/ffmpeg")
}

fn default_ytdlp_path() -> String {
    String::from(".libs/yt-dlp")
}