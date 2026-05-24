use std::sync::{Arc, Mutex};

use audio::AudioRecorder;
use tauri::{AppHandle, Manager};

mod audio;
mod autostart;
mod desktop;
mod file_context;
mod history;
mod hotkeys;
mod ollama;
mod provider;
mod secrets;
mod settings;
mod stt;
mod tray;
mod tts;
mod updater;

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
    provider::chat(&prompt)
}

#[tauri::command]
fn ask_assistant(prompt: String) -> Result<String, String> {
    provider::chat(&prompt)
}

#[tauri::command]
fn speak_text(text: String) -> Result<String, String> {
    tts::speak_text(&text)
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

#[tauri::command]
fn set_autostart(enabled: bool) -> Result<(), String> {
    autostart::set_enabled(enabled)
}

#[tauri::command]
fn list_providers() -> Vec<provider::ProviderInfo> {
    provider::list()
}

#[tauri::command]
fn history_recent(limit: Option<u32>) -> Result<Vec<history::HistoryEntry>, String> {
    history::list_recent(limit.unwrap_or(50))
}

#[tauri::command]
fn set_provider_api_key(provider_id: String, api_key: String) -> Result<(), String> {
    secrets::set_provider_api_key(&provider_id, &api_key)
}

#[tauri::command]
fn clear_provider_api_key(provider_id: String) -> Result<(), String> {
    secrets::clear_provider_api_key(&provider_id)
}

#[tauri::command]
fn provider_api_key_status(provider_id: String) -> secrets::SecretStatus {
    secrets::provider_secret_status(&provider_id)
}

#[tauri::command]
fn add_file_context() -> Result<file_context::FileContext, String> {
    file_context::pick_and_load()
}

#[tauri::command]
fn check_for_updates() -> Result<updater::UpdateStatus, String> {
    updater::check()
}

pub fn run() {
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(AudioRecorder::default())))
        .setup(|app| {
            let app_settings = settings::load().unwrap_or_default();
            let _ = history::init();
            if app_settings.tray_enabled {
                let _ = tray::setup(app);
            }
            hotkeys::start_hotkey_service(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_file_context,
            ask_assistant,
            ask_ollama,
            clear_provider_api_key,
            check_for_updates,
            get_desktop_capabilities,
            get_desktop_session_type,
            get_settings,
            history_recent,
            insert_text,
            list_providers,
            open_settings_window,
            provider_api_key_status,
            read_selected_text,
            replace_selected_text,
            save_settings,
            set_provider_api_key,
            speak_text,
            set_autostart
        ])
        .run(tauri::generate_context!())
        .expect("failed to run TwoKey Linux AI Assistant");
}
