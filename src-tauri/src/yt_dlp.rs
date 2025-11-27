use std::process::{Command, Stdio};
use tauri::Runtime;
use tauri_plugin_log::log::error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    BadURL,
    InvalidFormat,

}

#[tauri::command]
fn download_from_custom_format<R: Runtime>(app_handle: tauri::AppHandle<R>, url: String, format: String) -> Result<()> {

    let mut command_exit = Command::new("./yt-dlp")
        .arg("--progress")
        .arg("--newline")
        .arg("https://www.youtube.com/wh?v=9zKz-2PZU")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // yt-dlp prints progress on stderr
        .status();

    match command_exit {
        Ok(exit_status) => {
            match exit_status.success() {
                true => {


                },
                false => todo!(),
            }
        },
        Err(err) => {
            match err.kind() {
                _ => todo!()
            }
        },
    }

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