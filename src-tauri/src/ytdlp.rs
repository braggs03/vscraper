use regex::Regex;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, Runtime, State};
use tauri_plugin_log::log::{debug, error, info};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::app_state::AppState;
use crate::emissions::Emission;
use vscraper_lib::handle_emit_result;

#[derive(Clone, Serialize)]
struct DownloadProgress {
    url: String,
    percent: String,
    size_downloaded: String,
    speed: String,
    eta: String,
}

#[derive(Clone, Deserialize)]
struct DownloadOptions {
    container: String,
    audio_quality: String,
    video_quality: String,
    url: String,
    // YTDLP Options
}

#[tauri::command]
pub async fn download_best_quality<R: Runtime>(
    app_handle: tauri::AppHandle<R>,
    url: String,
) -> tauri::Result<()> {
    let best_format = "bestvideo+bestaudio";
    download_from_custom_format(app_handle, url, best_format.to_string()).await
}

#[tauri::command]
pub async fn download_from_custom_format<R: Runtime>(
    app_handle: tauri::AppHandle<R>,
    url: String,
    format: String,
) -> tauri::Result<()> {
    tauri::async_runtime::spawn(async move {
        let state: State<'_, Arc<Mutex<AppState>>> = app_handle.state();
        let config = state.lock().unwrap().get_config();
        let ytdlp_path = config.get_ytdlp_path();
        let ffmpeg_path = config.get_ffmpeg_path();
        let (tx, rx) = mpsc::channel(); //
        state.lock().unwrap().get_downloads().lock().unwrap().insert(url.clone(), tx);

        debug!("checking url availability for: {}", url);
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
                if !exit_status.success() {
                    // TODO: Parse stderr to provide exact error caused by yt-dlp.

                    // Return generic error in place of other errors
                } else {
                    let success_emit = app_handle.emit(
                        Emission::YtdlpUrlSuccess.as_string(),
                        true,
                    );

                    handle_emit_result(success_emit, Emission::YtdlpUrlSuccess.as_string());
                }
            }
            Err(err) => match err.kind() {
                err => error!("executing command: {}", err),
            },
        }

        debug!("downloading from url");
        let mut child = Command::new(&ytdlp_path)
            .arg("--newline")
            .arg("--limit-rate")
            .arg("100K")
            .arg("--format")
            .arg(format)
            .arg("--ffmpeg-location")
            .arg(&ffmpeg_path)
            .arg(url.clone())
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        debug!("spawned ytdlp download from url: {}, with pid: {}", url, child.id().map_or("unknown".to_string(), |code| code.to_string()));

        let stderr = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stderr).lines();

        let regex = Regex::new(r"\[download\]\s+(\d+(?:\.\d+)?)%\s+of\s+(\d+(?:\.\d+)?[GMK]iB)\s+at\s+(\d+\.\d+(?:[GMK]i)?B\/s)\s+ETA\s+(\d+:\d+)").unwrap();
        while let Ok(Some(line)) = reader.next_line().await {
            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    let pid = child.id().map_or("unknown".to_string(), |code| code.to_string());
                    debug!("received kill signal for url: {}, pid: {}", url, pid);
                    match child.kill().await {
                        Ok(_) => {
                            info!("successfully killed child for url: {}, pid: {}", url, pid);
                            match child.wait().await {
                                Ok(exit_status) => {
                                    debug!("killed zombie child for url: {}, pid: {}, exit code: {}", url, pid, exit_status);
                                },
                                Err(err) => {
                                    error!("failed to kill zombie child for url: {}, pid: {}, err: {}", url, pid, err);
                                },
                            }
                        },
                        Err(err) => error!("failed to kill child for url: {}, pid: {} err: {}", url, pid, err),
                    }
                    break
                },
                Err(TryRecvError::Empty) => {},
            }
            if line.contains("download") && regex.is_match(&line) {
                if let Some(captures) = regex.captures(&line) {
                    let url = url.clone();
                    let percent = String::from(&captures[1]);
                    let size_downloaded = String::from(&captures[2]);
                    let speed = String::from(&captures[3]);
                    let eta = String::from(&captures[4]);

                    let update_emit = app_handle.emit(
                        Emission::YtdlpDownloadUpdate.as_string(),
                        DownloadProgress {
                            url,
                            percent,
                            size_downloaded,
                            speed,
                            eta,
                        },
                    );

                    handle_emit_result(update_emit, Emission::YtdlpDownloadUpdate.as_string());
                }
            }
        }
    }).await?;

    Ok(())
}

#[tauri::command]
pub fn cancel_download(app_handle: tauri::AppHandle, url: String) {
    let state: State<'_, Arc<Mutex<AppState>>> = app_handle.state();
    let safe_downloads = state.lock().unwrap().get_downloads();
    let downloads = safe_downloads.lock().unwrap();
    let tx = downloads.get(&url);
    match tx {
        Some(tx) => {
            let success = tx.send(()).is_ok();
            let emit_result = app_handle.emit(Emission::YtdlpCancelDownload.as_string(), success);
            handle_emit_result(emit_result, Emission::YtdlpCancelDownload.as_string());
        }
        None => {
            error!("no download with url: {}", url);
        }
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
        let result = download_from_custom_format(app.app_handle().clone(), url, format).await;
        assert_eq!(result.is_ok(), false);
    });
}
