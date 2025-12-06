use regex::Regex;
use serde::Serialize;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, Runtime, State};
use tauri_plugin_log::log::{debug, error, info};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::app_state::AppState;
use crate::emissions::Emission;
use crate::handle_emit_result;

#[derive(Clone, Serialize)]
struct DownloadProgress {
    url: String,
    percent: String,
    size_downloaded: String,
    speed: String,
    eta: String,
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
) -> tauri::Result<String> {
    tauri::async_runtime::spawn(async move {
        let state: State<'_, Arc<Mutex<AppState>>> = app_handle.state();
        
        let config = state.lock().unwrap().get_config();

        let ytdlp_path = config.get_ytdlp_path();
        let ffmpeg_path = config.get_ffmpeg_path();

        debug!("checking url availability");
        let command_exit = Command::new(&ytdlp_path)
            .arg("--simulate")
            .arg(&url)
            .arg("--ffmpeg-location")
            .arg(&ffmpeg_path)
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .status()
            .await;

        match command_exit {
            Ok(exit_status) => {
                if exit_status.success() {
                    // TODO: Parse stderr to provide exact error caused by yt-dlp.

                    // Return generic error in place of other errors
                } else {
                    let success_emit = app_handle.emit(
                        "ytdlp_url_success",
                        true,
                    );

                    handle_emit_result(success_emit, "ytdlp_url_success");
                }
            }
            Err(err) => match err.kind() {
                err => error!("executing command: {}", err),
            },
        }

        debug!("downloading from url");

        let child = Command::new(&ytdlp_path)
            .arg("--newline")
            .arg("--limit-rate")
            .arg("100K")
            .arg("--format")
            .arg(format)
            .arg("--ffmpeg-location")
            .arg(&ffmpeg_path)
            .arg(url.clone())
            .stderr(Stdio::null()) // <-- capture stderr
            .stdout(Stdio::piped()) // <-- ignore stdout
            .spawn()
            .unwrap();
            
        let safe_child = Arc::new(Mutex::new(child));

        state.lock().unwrap().get_downloads().lock().unwrap().insert(url.clone(), safe_child.clone());

        let stderr = safe_child.lock().unwrap().stdout.take().unwrap();

        let mut reader = BufReader::new(stderr).lines();

        while let Some(line) = reader.next_line().await.unwrap() {
            info!("Line: {}", line);
            let regex = Regex::new(r"\[download\]\s+(\d+(?:\.\d+)?)%\s+of\s+(\d+(?:\.\d+)?[GMK]iB)\s+at\s+(\d+\.\d+(?:[GMK]i)?B\/s)\s+ETA\s+(\d+:\d+)").unwrap();
            if line.contains("download") && regex.is_match(&line) {
                if let Some(captures) = regex.captures(&line) {
                    let url = url.clone();
                    let percent = String::from(&captures[1]);
                    let size_downloaded = String::from(&captures[2]);
                    let speed = String::from(&captures[3]);
                    let eta = String::from(&captures[4]);

                    let update_emit = app_handle.emit(
                        Emission::YtdlpDownloadUpdate.as_str(),
                        DownloadProgress {
                            url,
                            percent,
                            size_downloaded,
                            speed,
                            eta,
                        },
                    );

                    handle_emit_result(update_emit, Emission::YtdlpDownloadUpdate.as_str());
                }
            }
        }
    }).await?;

    Ok("success".to_string())
}

#[tauri::command]
pub fn cancel_download(
    app_handle: tauri::AppHandle,
    url: String,
) {
    let state: State<'_, Arc<Mutex<AppState>>> = app_handle.state();
    let safe_downloads = state.lock().unwrap().get_downloads();
    let downloads = safe_downloads.lock().unwrap();
    let download_child = downloads.get(&url);
    match download_child {
        Some(download_child) => {
            let mut download_child = download_child.lock().unwrap();
            tauri::async_runtime::block_on(async {
                info!("{:?}", download_child);
                match download_child.id() {
                    Some(num) => {
                        info!("PID: {}", num);
                        Command::new("kill").arg(num.to_string());
                    },
                    None => todo!(),
                }
            });
        },
        None => {
            error!("no download with url: {}", url);
        },
    }
}

#[test]
fn test_ytdlp_custom_format_invalid_url() {
    tauri::async_runtime::block_on(async {
        use tauri::Manager;
        let url = String::from(
            "htt://www.youtube.com/watch?v=dQw4w9WgXcQ&list=RDdQw4w9WgXcQ&start_radio=1",
        );
        let format = String::from("best[height<=720]");
        let app = tauri::test::mock_app();
        let result = download_from_custom_format(app.app_handle().clone(), url, format);
        assert_eq!(result.await.is_ok(), false);
    });
}
