use serde::Serialize;

use crate::{ollama, settings};

pub trait ChatProvider {
    fn id(&self) -> &'static str;
    fn chat(&self, prompt: &str, model: &str) -> Result<String, String>;
}

pub struct OllamaProvider;

impl ChatProvider for OllamaProvider {
    fn id(&self) -> &'static str {
        "ollama"
    }

    fn chat(&self, prompt: &str, model: &str) -> Result<String, String> {
        ollama::chat_with_model(prompt, model)
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub enabled: bool,
    pub supports_chat: bool,
    pub supports_stt: bool,
    pub supports_tts: bool,
    pub supports_vision: bool,
    pub note: String,
}

pub fn chat(prompt: &str) -> Result<String, String> {
    let settings = settings::load().unwrap_or_default();
    let provider = OllamaProvider;
    eprintln!("twokey provider: {} chat model {}", provider.id(), settings.ollama_model);
    provider.chat(prompt, &settings.ollama_model)
}

pub fn list() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            id: "ollama".to_string(),
            label: "Ollama lokal".to_string(),
            kind: "local".to_string(),
            enabled: true,
            supports_chat: true,
            supports_stt: false,
            supports_tts: false,
            supports_vision: false,
            note: "Lokaler Chat ueber http://127.0.0.1:11434".to_string(),
        },
        ProviderInfo {
            id: "openai-compatible".to_string(),
            label: "OpenAI-kompatibel".to_string(),
            kind: "online".to_string(),
            enabled: false,
            supports_chat: true,
            supports_stt: true,
            supports_tts: true,
            supports_vision: true,
            note: "Platzhalter. API-Key-Verwaltung folgt ohne Secrets im Code.".to_string(),
        },
        ProviderInfo {
            id: "openrouter".to_string(),
            label: "OpenRouter-kompatibel".to_string(),
            kind: "online".to_string(),
            enabled: false,
            supports_chat: true,
            supports_stt: false,
            supports_tts: false,
            supports_vision: true,
            note: "Platzhalter fuer spaetere Modell-Routing-Auswahl.".to_string(),
        },
        ProviderInfo {
            id: "mock-stt".to_string(),
            label: "Mock STT".to_string(),
            kind: "local-dev".to_string(),
            enabled: true,
            supports_chat: false,
            supports_stt: true,
            supports_tts: false,
            supports_vision: false,
            note: "Deterministischer Entwicklungs-Transkribierer.".to_string(),
        },
    ]
}
