use serde_json::{json, Value};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{Emitter, Manager, Runtime, State};
use tauri_plugin_log::log::{debug, error, info};

use crate::app_state::{self, AppState};
use crate::handle_emit_result;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    BadURL,
    InvalidFormat,
}

#[tauri::command]
pub async fn download_best_quality<R: Runtime>(
    app_handle: tauri::AppHandle<R>,
    url: String,
) -> tauri::Result<()> {
    tauri::async_runtime::spawn(async move {
        let best_format = "bestvideo+bestaudio";

        let result = download_from_custom_format(app_handle, url, best_format.to_string()).await;

        result.is_ok()
    });

    Ok(())
}

#[tauri::command]
pub async fn download_from_custom_format<R: Runtime>(
    app_handle: tauri::AppHandle<R>,
    url: String,
    format: String,
) -> tauri::Result<()> {
    tauri::async_runtime::spawn(async move {
        debug!("checking url availibility");

        let state: State<'_, Arc<Mutex<AppState>>> = app_handle.state();

        let ytdlp_path = state.lock().unwrap().get_config().get_ytdlp_path();

        info!("ytdlp path: {}", ytdlp_path);

        let command_exit = Command::new(&ytdlp_path)
            .arg("--simulate")
            .arg(&url)
            .stderr(Stdio::piped())
            .status();

        match command_exit {
            Ok(exit_status) => {
                if !exit_status.success() {
                    // TODO: Parse stderr to provide exact error caused by yt-dlp.

                    // Return generic error in place of other errors
                } else {
                    let success_emit = app_handle.emit(
                        "yt_dlp_test",
                        json!({
                            "data": "1"
                        }),
                    );

                    handle_emit_result(success_emit, "yt_dlp_test");
                }
            }
            Err(err) => match err.kind() {
                err => error!("executing command: {}", err),
            },
        }

        debug!("downloading from url");

        let mut command = Command::new(&ytdlp_path);
        command.arg("--format").arg(format).arg(&url);

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    // Download failed
                    error!(
                        "Download failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    return Err(Error::InvalidFormat);
                }
            }
            Err(e) => {
                eprintln!("Error running command: {:?}", e);
                return Err(Error::InvalidFormat);
            }
        }
    });

    Ok(())
}

#[test]
fn ytdlp_custom_format_invalid_url() {
    tauri::async_runtime::block_on(async {
        use tauri::Manager;
        let url = String::from(
            "htt://www.youtube.com/watch?v=dQw4w9WgXcQ&list=RDdQw4w9WgXcQ&start_radio=1",
        );
        let format = String::from("best[height<=720]");

        let app = tauri::test::mock_app();

        let result = download_from_custom_format(app.app_handle().clone(), url, format);

        assert_eq!(result.await.is_ok(), true);
    });
}
