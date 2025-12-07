use tauri_plugin_log::log::{error, trace};

pub fn handle_emit_result(result: Result<(), tauri::Error>, kind: &str) {
    match result {
        Ok(_) => trace!("emitted event to frontend: {}", kind),
        Err(err) => error!("failed to emit event to frontend: {}", err),
    }
}