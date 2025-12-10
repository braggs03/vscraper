use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::{Mutex, mpsc};
use std::future::Future;
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;
use tauri::{Manager, Runtime, State};
use tauri_plugin_log::log::{debug, error, info, trace};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::app_state::AppState;
use crate::emissions::Emission;
use crate::emit_and_handle_result;

const YTDLP_DOWNLOAD_UPDATE_REGEX: &str = r"\[download\]\s+(\d+(?:\.\d+)?)%\s+of\s+~?\s+?(\d+(?:\.\d+)?[GMK]iB)\s+at\s+(\d+\.\d+(?:[GMK]i)?B\/s)\s+ETA\s+((\d+:\d+)|(?:Unknown))";

#[derive(Clone, Serialize)]
struct DownloadProgress {
    url: String,
    percent: String,
    size_downloaded: String,
    speed: String,
    eta: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DownloadOptions {
    #[serde(default = "default_container")]
    container: String,
    #[serde(default = "default_name_format")]
    name_format: String,
    url: String,
    #[serde(default = "default_quality")]
    quality: String,
    // YTDLP Options
}

fn default_container() -> String {
    String::from("mp4")
}

fn default_name_format() -> String {
    String::from("%(title)s.%(ext)s")
}

fn default_quality() -> String {
    String::from("best")
}

#[tauri::command]
pub async fn download_from_options<R: Runtime>(
    app_handle: tauri::AppHandle<R>,
    options: DownloadOptions,
) -> tauri::Result<()> {
    tauri::async_runtime::spawn(async move {
        let state: State<'_, Arc<Mutex<AppState>>> = app_handle.state();
        let config = state.lock().await.get_config();
        let ytdlp_path = config.get_ytdlp_path();
        let ffmpeg_path = config.get_ffmpeg_path();
        let (tx, mut rx) = mpsc::channel(100); // Used to communicate kill and pause.
        state.lock().await.add_download(options.url.clone(), tx);

        debug!("checking url availability for: {}", options.url);
        match check_url_availability(&ytdlp_path, &ffmpeg_path, &options).await {
            Ok(exit_status) => {
                if exit_status.success() {
                    // TODO: Parse stderr to provide exact error caused by yt-dlp.

                    // Return generic error in place of other errors
                } else {

                }
                emit_and_handle_result(&app_handle, Emission::YtdlpUrlUpdate, exit_status.success());
            }
            Err(err) => match err.kind() {
                err => error!("executing command: {}", err),
            },
        }

        let download_path = app_handle.path().download_dir().unwrap().join(&options.name_format);

        debug!("downloading from url");
        let mut child = Command::new(&ytdlp_path)
            .arg("--newline")
            .arg("--ffmpeg-location")
            .arg(&ffmpeg_path)
            .arg("-f")
            .arg(options.quality)
            .arg("-o")
            .arg(download_path)
            .arg(options.url.clone())
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        debug!("spawned ytdlp download from url: {}, with pid: {}", options.url, child.id().map_or("unknown".to_string(), |code| code.to_string()));

        let stderr = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stderr).lines();

        let regex = Regex::new(YTDLP_DOWNLOAD_UPDATE_REGEX).unwrap();
        while let Ok(Some(line)) = reader.next_line().await {
            trace!("ytdlp: {}", line);
            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    let pid = child.id().map_or("unknown".to_string(), |code| code.to_string());
                    debug!("received kill signal for url: {}, pid: {}", options.url, pid);
                    match child.kill().await {
                        Ok(_) => {
                            info!("successfully killed child for url: {}, pid: {}", options.url, pid);
                            match child.wait().await {
                                Ok(exit_status) => {
                                    debug!("killed zombie child for url: {}, pid: {}, exit code: {}", options.url, pid, exit_status);
                                },
                                Err(err) => {
                                    error!("failed to kill zombie child for url: {}, pid: {}, err: {}", options.url, pid, err);
                                },
                            }
                        },
                        Err(err) => error!("failed to kill child for url: {}, pid: {} err: {}", options.url, pid, err),
                    }
                    break
                },
                Err(TryRecvError::Empty) => {},
            }
            if regex.is_match(&line) {
                if let Some(captures) = regex.captures(&line) {
                    let url = options.url.clone();
                    let percent = String::from(&captures[1]);
                    let size_downloaded = String::from(&captures[2]);
                    let speed = String::from(&captures[3]);
                    let eta = String::from(&captures[4]);

                    emit_and_handle_result(
                        &app_handle, 
                        Emission::YtdlpDownloadUpdate, 
                        DownloadProgress {
                            url,
                            percent,
                            size_downloaded,
                            speed,
                            eta,
                        }
                    );
                }
            }
        }

        match child.wait().await {
            Ok(status) => {
                emit_and_handle_result(
                    &app_handle, 
                    Emission::YtdlpDownloadFinish, 
                    status.success()
                );
            },
            Err(err) => error!("download with url: {}, failed with err: {}", options.url, err),
        }
    }).await?;

    Ok(())
}

#[tauri::command]
pub async fn cancel_download(app_handle: tauri::AppHandle, url: String) {
    let state: State<'_, Arc<Mutex<AppState>>> = app_handle.state();
    let app_state = state.lock().await;
    let tx = app_state.get_download(&url);
    match tx {
        Some(tx) => {
            emit_and_handle_result(
                &app_handle, 
                Emission::YtdlpCancelDownload, 
                tx.send(()).await.is_ok()
            );
        }
        None => {
            error!("no download with url: {}", url);
        }
    }
}

fn check_url_availability(
    ytdlp_path: &PathBuf, 
    ffmpeg_path: &PathBuf, 
    options: &DownloadOptions
) -> impl Future<Output = Result<ExitStatus, std::io::Error>> {
    Command::new(&ytdlp_path)
        .arg("--ffmpeg-location")
        .arg(&ffmpeg_path)
        .arg("--simulate")
        .arg(&options.url)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .status()
}

#[tauri::command]
pub async fn download_best_quality<R: Runtime>(
    app_handle: tauri::AppHandle<R>,
    options: DownloadOptions,
) -> tauri::Result<()> {
    //
    download_from_options(
        app_handle, 
        DownloadOptions {
            quality: String::from("bestvideo"),
            ..options
        }
    ).await
}        

// #[test]
// fn test_ytdlp_custom_format_invalid_url() {
//     tauri::async_runtime::block_on(async {
//         use tauri::Manager;
//         let url = String::from(
//             "htt://www.youtube.com/watch?v=dQw4w9WgXcQ&list=RDdQw4w9WgXcQ&start_radio=1",
//         );
//         let app = tauri::test::mock_app();
//         let result = download_from_options(
//             app.app_handle().clone(),
//             DownloadOptions { 
//                 url,
//                 container: default_container(),
//                 name_format: default_name_format(),
//                 quality: default_quality(),
//             }
//         ).await;
//         assert_eq!(result.is_ok(), false);
//     });
// }
