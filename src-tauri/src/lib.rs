use std::sync::{Arc, Mutex};

use audio::AudioRecorder;
use tauri::{AppHandle, Manager};

mod audio;
mod desktop;
mod hotkeys;
mod ollama;
mod stt;

#[tauri::command]
fn get_desktop_session_type() -> String {
    std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".to_string())
}

#[tauri::command]
fn get_desktop_capabilities() -> hotkeys::DesktopCapabilities {
    hotkeys::capabilities()
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

#[tauri::command]
fn ask_ollama(prompt: String) -> Result<String, String> {
    ollama::chat(&prompt)
}

#[tauri::command]
fn insert_text(text: String) -> Result<(), String> {
    desktop::insert_text(&text)
}

pub fn run() {
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(AudioRecorder::default())))
        .setup(|app| {
            hotkeys::start_hotkey_service(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ask_ollama,
            get_desktop_capabilities,
            get_desktop_session_type,
            insert_text,
            open_settings_window
        ])
        .run(tauri::generate_context!())
        .expect("failed to run TwoKey Linux AI Assistant");
}
