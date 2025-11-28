use std::process::{Command, Stdio};
use tauri::Runtime;
use tauri_plugin_log::log::{debug, error};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    BadURL,
    InvalidFormat,
}

#[tauri::command]
pub fn download_best_quality<R: Runtime>(app_handle: tauri::AppHandle<R>, url: String) -> bool {

    let best_format = "bestvideo+bestaudio";

    let result = download_from_custom_format(app_handle, url, best_format.to_string());

    result.is_ok()
}

#[tauri::command]
fn download_from_custom_format<R: Runtime>(app_handle: tauri::AppHandle<R>, url: String, format: String) -> Result<()> {

    debug!("checking url availibility");

    let command_exit = Command::new("./yt-dlp")
        .arg("--progress")
        .arg("--newline")
        .arg(&url)
        .stderr(Stdio::piped())
        .status();

    match command_exit {
        Ok(exit_status) => {
            if !exit_status.success() {
                // TODO: Parse stderr to provide exact error caused by yt-dlp.
                
                // Return generic error in place of other errors
                return Err(Error::BadURL)
            }
        },
        Err(err) => {
            match err.kind() {
                err => error!("Error: {}", err),
            }
        },
    }

    debug!("downloading from url");

    let mut command = Command::new("./libs/yt-dlp");
    command.arg("--format").arg(format).arg(&url);

    match command.output() {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                // Download failed
                error!("Download failed: {}", String::from_utf8_lossy(&output.stderr));
                return Err(Error::InvalidFormat);
            }
        }
        Err(e) => {
            eprintln!("Error running command: {:?}", e);
            return Err(Error::InvalidFormat);
        }
    }
}


#[cfg(test)]


#[test]
fn ytdlp_custom_format_invalid_url() {
    use tauri::Manager;

    let url = String::from("htt://www.youtube.com/watch?v=dQw4w9WgXcQ&list=RDdQw4w9WgXcQ&start_radio=1");
    let format = String::from("best[height<=720]");

    let app = tauri::test::mock_app();

    let result = download_from_custom_format(app.app_handle().clone(), url, format);

    assert_eq!(result.err().unwrap(), Error::BadURL);
}