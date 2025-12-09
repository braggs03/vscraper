use std::sync::{Arc, Mutex};
use tauri::State;
use tauri_plugin_log::log::error;
use ubi::UbiBuilder;

use crate::{app_state::AppState, emissions::Emission, emit_and_handle_result};

pub const FFMPEG_EXECUTABLE: &str = "ffmpeg";
const FFMPEG_GITHUB: &str = "eugeneware/ffmpeg-static";
pub const YTDLP_EXECUTABLE: &str = "yt-dlp";
const YTDLP_GITHUB: &str = "yt-dlp/yt-dlp";

#[tauri::command]
pub async fn install_ytdlp(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> tauri::Result<()> {
    let install_path = state.lock().unwrap().get_config().get_binary_path();
    install_lib(
        app_handle,
        String::from(YTDLP_GITHUB),
        String::from(YTDLP_EXECUTABLE),
        install_path,
        Emission::YtdlpInstall,
    );

    Ok(())
}

#[tauri::command]
pub async fn install_ffmpeg(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> tauri::Result<()> {
    let install_path = state.lock().unwrap().get_config().get_binary_path();
    install_lib(
        app_handle,
        String::from(FFMPEG_GITHUB),
        String::from(FFMPEG_EXECUTABLE),
        install_path,
        Emission::FfmpegInstall,
    );

    Ok(())
}

fn install_lib(
    app_handle: tauri::AppHandle,
    git_format: String,
    executable_name: String,
    install_path: std::path::PathBuf,
    emission: Emission,
) {
    std::thread::spawn(move || {
        let ubi = UbiBuilder::new()
            .project(&git_format)
            .install_dir(install_path)
            .rename_exe_to(&executable_name)
            .build();

        match ubi {
            Ok(mut ubi) => {
                tauri::async_runtime::block_on(async {
                    let install_result = ubi.install_binary().await;
                    emit_and_handle_result(&app_handle, emission, install_result.is_ok());
                });
            }
            Err(err) => error!("building ubi installer for {}: {}", git_format, err),
        }
    });
}

#[tauri::command]
pub async fn install_ffmpeg_ytdlp(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> tauri::Result<()> {
    install_ffmpeg(app_handle.clone(), state.clone()).await?;
    install_ytdlp(app_handle, state).await?;

    Ok(())
}
