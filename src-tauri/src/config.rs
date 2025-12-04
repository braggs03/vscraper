use serde::{Deserialize, Serialize};

pub const CONFIG_FILENAME: &str = "settings.json";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Config {
    #[serde(default = "default_ffmpeg_path")]
    pub ffmpeg_path: String,

    #[serde(default)]
    skip_homepage: bool,

    #[serde(default = "default_ytdlp_path")]
    ytdlp_path: String,
}

fn default_ffmpeg_path() -> String {
    String::from("./libs/ffmpeg")
}

fn default_ytdlp_path() -> String {
    String::from("./libs/yt-dlp")
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ffmpeg_path: "./libs/ffmpeg".to_string(),
            skip_homepage: false,
            ytdlp_path: "./libs/yt-dlp".to_string(),
        }
    }
}

impl Config {
    pub fn get_ytdlp_path(&self) -> String {
        self.ytdlp_path.clone()
    }
}

#[test]
fn test_default_config() {
    let config = Config {
        ffmpeg_path: "./libs/ffmpeg".to_string(),
        skip_homepage: false,
        ytdlp_path: ".libs/yt-dlp".to_string(),
    };

    assert_eq!(config, Config::default());

    let serde_conf: Config = serde_json::from_str("{}").unwrap();

    assert_eq!(serde_conf, Config::default());
}
