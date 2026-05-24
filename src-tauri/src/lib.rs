use std::sync::{Arc, Mutex};

use audio::AudioRecorder;
use tauri::{AppHandle, Manager};

mod audio;
mod desktop;
mod hotkeys;
mod ollama;
mod settings;
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

#[tauri::command]
fn read_selected_text() -> Result<String, String> {
    desktop::read_selected_text()
}

#[tauri::command]
fn replace_selected_text(text: String) -> Result<(), String> {
    desktop::replace_selected_text(&text)
}

#[tauri::command]
fn get_settings() -> Result<settings::AppSettings, String> {
    settings::load()
}

#[tauri::command]
fn save_settings(settings: settings::AppSettings) -> Result<(), String> {
    settings::save(&settings)
}

pub fn run() {
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(AudioRecorder::default())))
        .setup(|app| {
            let _ = settings::load();
            hotkeys::start_hotkey_service(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ask_ollama,
            get_desktop_capabilities,
            get_desktop_session_type,
            get_settings,
            insert_text,
            open_settings_window,
            read_selected_text,
            replace_selected_text,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("failed to run TwoKey Linux AI Assistant");
}
