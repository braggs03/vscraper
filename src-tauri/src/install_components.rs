use tauri::Emitter;
use ubi::UbiBuilder;

use crate::handle_emit_result;

#[tauri::command]
pub fn install_ytdlp(app_handle: tauri::AppHandle) -> bool {
    let result = install_lib(app_handle, "yt-dlp/yt-dlp", "./libs", Some("yt-dlp_install"));

    result
}

#[tauri::command]
pub fn install_ffmpeg(app_handle: tauri::AppHandle) -> bool {
    let result = install_lib(app_handle, "eugeneware/ffmpeg-static", "./libs", Some("ffmpeg_install"));

    result
}

#[tauri::command]
pub fn install_ffmpeg_ytdlp(app_handle: tauri::AppHandle) {
    install_ffmpeg(app_handle.clone());
    install_ytdlp(app_handle);
}


fn install_lib(app_handle: tauri::AppHandle, git_format: &str, install_path: &str, emit_event: Option<&str>) -> bool {
    let success = tauri::async_runtime::block_on(async {
        let ubi = UbiBuilder::new()
            .project(git_format)
            .extract_all()
            .install_dir(install_path)
            .build();
        let result = match ubi {
            Ok(mut ubi) => ubi.install_binary().await,
            Err(err) => Err(err),
        };
        
        if let Some(emit_event) = emit_event {
            let emit_status = app_handle.emit(emit_event, result.is_ok());
            handle_emit_result(emit_status, emit_event);
        }

        result
    });

   success.is_ok()
}
