use std::sync::{Arc, Mutex};

use audio::AudioRecorder;
use serde::Serialize;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, Size, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

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
mod toolchains;
mod tray;
mod tts;
mod updater;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UiHotkeyEvent {
    kind: &'static str,
    status: &'static str,
    message: String,
    audio_path: Option<String>,
    transcript: Option<String>,
    provider: Option<String>,
}

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
    let window = ensure_settings_window(&app)?;

    window.unminimize().map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_always_on_top(true).map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    window.set_always_on_top(false).map_err(|error| error.to_string())?;
    Ok(())
}

fn ensure_settings_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("settings") {
        return Ok(window);
    }

    WebviewWindowBuilder::new(
        app,
        "settings",
        WebviewUrl::App("index.html?view=settings".into()),
    )
    .title("TwoKey Einstellungen")
    .inner_size(920.0, 680.0)
    .min_inner_size(760.0, 560.0)
    .center()
    .visible(false)
    .resizable(true)
    .build()
    .map_err(|error| format!("Settings window could not be created: {error}"))
}

#[tauri::command]
fn set_overlay_window_expanded(app: AppHandle, expanded: bool) -> Result<(), String> {
    let window = app
        .get_webview_window("overlay")
        .ok_or_else(|| "Overlay window is not configured".to_string())?;

    let size = if expanded {
        Size::Logical(LogicalSize::new(440.0, 620.0))
    } else {
        Size::Logical(LogicalSize::new(340.0, 72.0))
    };

    window.set_size(size).map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
fn ask_ollama(prompt: String) -> Result<String, String> {
    provider::chat(&prompt)
}

#[tauri::command]
fn ask_assistant(prompt: String, file_context: Option<provider::ChatFileContext>) -> Result<String, String> {
    provider::chat_with_context(&prompt, file_context)
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
async fn save_settings(settings: settings::AppSettings) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        if settings.stt_provider == "local-whisper" {
            stt::ensure_local_whisper_installed()?;
        }

        settings::save(&settings)
    })
    .await
    .map_err(|error| format!("Settings-Task ist fehlgeschlagen: {error}"))?
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
fn get_local_whisper_diagnostics() -> stt::LocalWhisperDiagnostics {
    stt::local_whisper_diagnostics()
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

#[tauri::command]
fn install_latest_update() -> Result<String, String> {
    updater::install_latest_appimage()
}

#[tauri::command]
fn submit_feedback(text: String) -> Result<String, String> {
    history::record(history::AuditEvent {
        kind: "feedback".to_string(),
        mode: Some("feedback".to_string()),
        provider: None,
        input_text: Some(text),
        output_text: None,
        metadata_json: None,
        success: true,
    })?;

    Ok("Danke fuer dein Feedback. Es wurde lokal gespeichert und kann in der Historie eingesehen werden.".to_string())
}

#[tauri::command]
fn run_toolchain_from_text(text: String) -> Result<Option<String>, String> {
    toolchains::run_for_transcript(&text)
}

#[tauri::command]
fn start_manual_capture(app: AppHandle) -> Result<(), String> {
    let recorder = app.state::<Arc<Mutex<AudioRecorder>>>().inner().clone();
    let path = recorder
        .lock()
        .map_err(|_| "Recorder-State ist gesperrt".to_string())?
        .start()?;

    let _ = app.emit(
        "twokey://hotkey-event",
        UiHotkeyEvent {
            kind: "recording-started",
            status: "listening",
            message: "Höre zu... Aufnahme laeuft.".to_string(),
            audio_path: Some(path.to_string_lossy().to_string()),
            transcript: None,
            provider: None,
        },
    );

    Ok(())
}

#[tauri::command]
fn stop_manual_capture(app: AppHandle) -> Result<(), String> {
    let recorder = app.state::<Arc<Mutex<AudioRecorder>>>().inner().clone();
    let stopped_path = recorder
        .lock()
        .map_err(|_| "Recorder-State ist gesperrt".to_string())?
        .stop()?;

    if let Some(path) = stopped_path {
        let _ = app.emit(
            "twokey://hotkey-event",
            UiHotkeyEvent {
                kind: "recording-stopped",
                status: "transcribing",
                message: "Audioaufnahme gespeichert. Transkribiere...".to_string(),
                audio_path: Some(path.to_string_lossy().to_string()),
                transcript: None,
                provider: None,
            },
        );

        let app_handle = app.clone();
        std::thread::spawn(move || match stt::transcribe(&path) {
            Ok(transcript) => {
                let _ = app_handle.emit(
                    "twokey://hotkey-event",
                    UiHotkeyEvent {
                        kind: "transcript-ready",
                        status: "ready",
                        message: "Transkription abgeschlossen. Starte KI-Verarbeitung.".to_string(),
                        audio_path: Some(path.to_string_lossy().to_string()),
                        transcript: Some(transcript.text),
                        provider: Some(transcript.provider),
                    },
                );
            }
            Err(error) => {
                let _ = app_handle.emit(
                    "twokey://hotkey-event",
                    UiHotkeyEvent {
                        kind: "error",
                        status: "error",
                        message: error,
                        audio_path: Some(path.to_string_lossy().to_string()),
                        transcript: None,
                        provider: None,
                    },
                );
            }
        });
    }

    Ok(())
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
            install_latest_update,
            get_desktop_capabilities,
            get_desktop_session_type,
            get_settings,
            history_recent,
            insert_text,
            list_providers,
            get_local_whisper_diagnostics,
            open_settings_window,
            provider_api_key_status,
            read_selected_text,
            replace_selected_text,
            save_settings,
            run_toolchain_from_text,
            start_manual_capture,
            stop_manual_capture,
            set_overlay_window_expanded,
            set_provider_api_key,
            speak_text,
            submit_feedback,
            set_autostart
        ])
        .run(tauri::generate_context!())
        .expect("failed to run TwoKey Linux AI Assistant");
}
