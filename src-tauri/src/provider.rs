use serde::{Deserialize, Serialize};

use crate::{history, ollama, secrets, settings};

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

pub struct OpenAICompatibleProvider;

impl ChatProvider for OpenAICompatibleProvider {
    fn id(&self) -> &'static str {
        "openai-compatible"
    }

    fn chat(&self, prompt: &str, _model: &str) -> Result<String, String> {
        let app_settings = settings::load().unwrap_or_default();
        let api_key = secrets::get_provider_api_key("openai-compatible")?;
        chat_via_openai_api(
            &app_settings.openai_base_url,
            &app_settings.openai_model,
            &api_key,
            None,
            prompt,
        )
    }
}

pub struct OpenRouterProvider;

impl ChatProvider for OpenRouterProvider {
    fn id(&self) -> &'static str {
        "openrouter"
    }

    fn chat(&self, prompt: &str, _model: &str) -> Result<String, String> {
        let app_settings = settings::load().unwrap_or_default();
        let api_key = secrets::get_provider_api_key("openrouter")?;
        chat_via_openai_api(
            &app_settings.openrouter_base_url,
            &app_settings.openrouter_model,
            &api_key,
            Some(("https://github.com/meinzeug/twokey", "TwoKey Linux AI Assistant")),
            prompt,
        )
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

#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageResponse,
}

#[derive(Deserialize)]
struct OpenAIMessageResponse {
    content: String,
}

pub fn chat(prompt: &str) -> Result<String, String> {
    let app_settings = settings::load().unwrap_or_default();
    let provider_id = app_settings.preferred_chat_provider.clone();

    let result = match provider_id.as_str() {
        "openai-compatible" => {
            let provider = OpenAICompatibleProvider;
            provider.chat(prompt, &app_settings.openai_model)
        }
        "openrouter" => {
            let provider = OpenRouterProvider;
            provider.chat(prompt, &app_settings.openrouter_model)
        }
        _ => {
            let provider = OllamaProvider;
            provider.chat(prompt, &app_settings.ollama_model)
        }
    };

    let _ = history::record(history::AuditEvent {
        kind: "chat".to_string(),
        mode: Some("conversation".to_string()),
        provider: Some(provider_id),
        input_text: Some(prompt.to_string()),
        output_text: result.clone().ok(),
        metadata_json: None,
        success: result.is_ok(),
    });

    result
}

pub fn list() -> Vec<ProviderInfo> {
    let openai_has_key = secrets::provider_secret_status("openai-compatible").configured;
    let openrouter_has_key = secrets::provider_secret_status("openrouter").configured;

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
            enabled: openai_has_key,
            supports_chat: true,
            supports_stt: true,
            supports_tts: true,
            supports_vision: true,
            note: if openai_has_key {
                "Aktiv. API-Key ist sicher im lokalen Keyring gespeichert.".to_string()
            } else {
                "Inaktiv. API-Key im Settings-Fenster hinterlegen.".to_string()
            },
        },
        ProviderInfo {
            id: "openrouter".to_string(),
            label: "OpenRouter-kompatibel".to_string(),
            kind: "online".to_string(),
            enabled: openrouter_has_key,
            supports_chat: true,
            supports_stt: false,
            supports_tts: false,
            supports_vision: true,
            note: if openrouter_has_key {
                "Aktiv. API-Key ist sicher im lokalen Keyring gespeichert.".to_string()
            } else {
                "Inaktiv. API-Key im Settings-Fenster hinterlegen.".to_string()
            },
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

fn chat_via_openai_api(
    base_url: &str,
    model: &str,
    api_key: &str,
    app_headers: Option<(&str, &str)>,
    prompt: &str,
) -> Result<String, String> {
    let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = OpenAIChatRequest {
        model: model.to_string(),
        messages: vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: "Du bist TwoKey, ein knapper Linux-Desktop-Assistent.".to_string(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
        temperature: 0.2,
    };

    let mut request = reqwest::blocking::Client::new()
        .post(endpoint)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json");

    if let Some((referer, title)) = app_headers {
        request = request.header("HTTP-Referer", referer).header("X-Title", title);
    }

    let response = request
        .json(&body)
        .send()
        .map_err(|error| format!("Online-Provider konnte nicht erreicht werden: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("Online-Provider antwortete mit HTTP {}", response.status()));
    }

    let payload: OpenAIChatResponse = response
        .json()
        .map_err(|error| format!("Provider-Antwort konnte nicht gelesen werden: {error}"))?;

    let content = payload
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content.trim().to_string())
        .unwrap_or_default();

    if content.is_empty() {
        return Err("Provider lieferte eine leere Antwort".to_string());
    }

    Ok(content)
}
