use tauri::{AppHandle, Manager};

#[tauri::command]
fn get_desktop_session_type() -> String {
    std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".to_string())
}

#[tauri::command]
fn open_settings_window(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("settings")
        .ok_or_else(|| "Settings window is not configured".to_string())?;

    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_desktop_session_type,
            open_settings_window
        ])
        .run(tauri::generate_context!())
        .expect("failed to run TwoKey Linux AI Assistant");
}
