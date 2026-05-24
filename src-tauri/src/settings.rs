use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub autostart: bool,
    pub overlay_position: String,
    pub overlay_size: String,
    pub theme: String,
    pub accent_color: String,
    pub main_hotkey: String,
    pub double_tap_ms: u32,
    pub escape_cancel: bool,
    pub stt_provider: String,
    pub default_language: String,
    pub whisper_model: String,
    pub whisper_beam_size: u8,
    pub edit_auto_apply: bool,
    pub punctuation_cleanup: bool,
    pub tts_enabled: bool,
    pub tts_voice: String,
    pub tts_speed: f32,
    pub ollama_model: String,
    pub preferred_chat_provider: String,
    pub openai_base_url: String,
    pub openai_model: String,
    pub openrouter_base_url: String,
    pub openrouter_model: String,
    pub prefer_local: bool,
    pub save_history: bool,
    pub log_api_requests: bool,
    pub tray_enabled: bool,
    pub update_channel: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            autostart: false,
            overlay_position: "top-left".to_string(),
            overlay_size: "compact".to_string(),
            theme: "dark".to_string(),
            accent_color: "#75e0c3".to_string(),
            main_hotkey: "Ctrl+Space".to_string(),
            double_tap_ms: 420,
            escape_cancel: true,
            stt_provider: "mock".to_string(),
            default_language: "de".to_string(),
            whisper_model: "base".to_string(),
            whisper_beam_size: 5,
            edit_auto_apply: true,
            punctuation_cleanup: false,
            tts_enabled: false,
            tts_voice: "piper-default".to_string(),
            tts_speed: 1.0,
            ollama_model: "qwen2.5:3b".to_string(),
            preferred_chat_provider: "ollama".to_string(),
            openai_base_url: "https://api.openai.com/v1".to_string(),
            openai_model: "gpt-4o-mini".to_string(),
            openrouter_base_url: "https://openrouter.ai/api/v1".to_string(),
            openrouter_model: "openai/gpt-4o-mini".to_string(),
            prefer_local: true,
            save_history: true,
            log_api_requests: false,
            tray_enabled: true,
            update_channel: "stable".to_string(),
        }
    }
}

pub fn load() -> Result<AppSettings, String> {
    let path = settings_path()?;
    if !path.exists() {
        let settings = AppSettings::default();
        save(&settings)?;
        return Ok(settings);
    }

    let content = fs::read_to_string(&path).map_err(|error| format!("Settings konnten nicht gelesen werden: {error}"))?;
    serde_json::from_str(&content).map_err(|error| format!("Settings JSON ist ungueltig: {error}"))
}

pub fn save(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("Config-Verzeichnis konnte nicht erstellt werden: {error}"))?;
    }

    let content = serde_json::to_string_pretty(settings).map_err(|error| format!("Settings konnten nicht serialisiert werden: {error}"))?;
    fs::write(path, content).map_err(|error| format!("Settings konnten nicht gespeichert werden: {error}"))
}

fn settings_path() -> Result<PathBuf, String> {
    let base = if let Ok(value) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(value)
    } else {
        let home = std::env::var("HOME").map_err(|_| "HOME ist nicht gesetzt".to_string())?;
        PathBuf::from(home).join(".config")
    };

    Ok(base.join("twokey-ai").join("settings.json"))
}
