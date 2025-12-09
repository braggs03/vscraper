use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::components;

pub const CONFIG_FILENAME: &str = "settings.json";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Config {
    #[serde(default = "default_binary_path")]
    binary_install_path: PathBuf,
    
    #[serde(default = "default_ffmpeg_path")]
    pub ffmpeg_path: PathBuf,

    #[serde(default)]
    skip_homepage: bool,

    #[serde(default = "default_ytdlp_path")]
    ytdlp_path: PathBuf,
}

fn default_binary_path() -> PathBuf {
    PathBuf::from("./libs/")
}

fn default_ffmpeg_path() -> PathBuf {
    default_binary_path().join(components::FFMPEG_EXECUTABLE)
}

fn default_ytdlp_path() -> PathBuf {
    default_binary_path().join(components::YTDLP_EXECUTABLE)
}

impl Default for Config {
    fn default() -> Self {
        Config {
            binary_install_path: default_binary_path(),
            ffmpeg_path: default_ffmpeg_path(),
            skip_homepage: false,
            ytdlp_path: default_ytdlp_path(),
        }
    }
}

impl Config {
    pub fn get_binary_path(&self) -> PathBuf {
        self.binary_install_path.clone()
    }

    pub fn get_ytdlp_path(&self) -> PathBuf {
        self.ytdlp_path.clone()
    }

    pub fn get_ffmpeg_path(&self) -> PathBuf {
        self.ffmpeg_path.clone()
    }

    pub fn get_skip_homepage(&self) -> bool {
        self.skip_homepage
    }

    pub fn set_skip_homepage(&mut self, new_preference: bool) {
        self.skip_homepage = new_preference;
    }
}

#[test]
fn test_default_config() {
    let serde_conf: Config = serde_json::from_str("{}").unwrap();
    assert_eq!(serde_conf, Config::default());
}
