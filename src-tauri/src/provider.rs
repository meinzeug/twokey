use std::path::Path;

use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{history, ollama, secrets, settings};

pub trait ChatProvider {
    fn chat(&self, prompt: &str, model: &str, context: Option<&ChatFileContext>) -> Result<String, String>;
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatFileContext {
    pub path: String,
    pub name: String,
    pub kind: String,
    pub summary: String,
    pub extracted_text: Option<String>,
}

pub struct OllamaProvider;

impl ChatProvider for OllamaProvider {
    fn chat(&self, prompt: &str, model: &str, context: Option<&ChatFileContext>) -> Result<String, String> {
        if let Some(file_context) = context {
            if file_context.kind == "image" {
                return ollama::chat_with_model_and_image(prompt, model, Path::new(&file_context.path));
            }
        }

        ollama::chat_with_model(prompt, model)
    }
}

pub struct OpenAICompatibleProvider;

impl ChatProvider for OpenAICompatibleProvider {
    fn chat(&self, prompt: &str, _model: &str, context: Option<&ChatFileContext>) -> Result<String, String> {
        let app_settings = settings::load().unwrap_or_default();
        let api_key = secrets::get_provider_api_key("openai-compatible")?;
        chat_via_openai_api(
            &app_settings.openai_base_url,
            &app_settings.openai_model,
            &api_key,
            None,
            prompt,
            context,
        )
    }
}

pub struct OpenRouterProvider;

impl ChatProvider for OpenRouterProvider {
    fn chat(&self, prompt: &str, _model: &str, context: Option<&ChatFileContext>) -> Result<String, String> {
        let app_settings = settings::load().unwrap_or_default();
        let api_key = secrets::get_provider_api_key("openrouter")?;
        chat_via_openai_api(
            &app_settings.openrouter_base_url,
            &app_settings.openrouter_model,
            &api_key,
            Some(("https://github.com/meinzeug/twokey", "TwoKey Linux AI Assistant")),
            prompt,
            context,
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
    chat_with_context(prompt, None)
}

pub fn chat_with_context(prompt: &str, context: Option<ChatFileContext>) -> Result<String, String> {
    let app_settings = settings::load().unwrap_or_default();
    let provider_id = choose_provider(&app_settings, context.as_ref());

    let result = match provider_id.as_str() {
        "openai-compatible" => {
            let provider = OpenAICompatibleProvider;
            provider.chat(prompt, &app_settings.openai_model, context.as_ref())
        }
        "openrouter" => {
            let provider = OpenRouterProvider;
            provider.chat(prompt, &app_settings.openrouter_model, context.as_ref())
        }
        _ => {
            let provider = OllamaProvider;
            provider.chat(prompt, &app_settings.ollama_model, context.as_ref())
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

fn choose_provider(app_settings: &settings::AppSettings, context: Option<&ChatFileContext>) -> String {
    if !app_settings.prefer_local {
        return app_settings.preferred_chat_provider.clone();
    }

    if context.is_some_and(|ctx| ctx.kind == "image") {
        if secrets::provider_secret_status("openai-compatible").configured {
            return "openai-compatible".to_string();
        }
        if secrets::provider_secret_status("openrouter").configured {
            return "openrouter".to_string();
        }
    }

    "ollama".to_string()
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
    context: Option<&ChatFileContext>,
) -> Result<String, String> {
    let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let user_message = if let Some(file_context) = context {
        if file_context.kind == "image" {
            let data_url = image_path_to_data_url(Path::new(&file_context.path))?;
            json!({
                "role": "user",
                "content": [
                    { "type": "text", "text": prompt },
                    { "type": "image_url", "image_url": { "url": data_url } }
                ]
            })
        } else {
            json!({ "role": "user", "content": prompt })
        }
    } else {
        json!({ "role": "user", "content": prompt })
    };

    let body = json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "Du bist TwoKey, ein knapper Linux-Desktop-Assistent."
            },
            user_message
        ],
        "temperature": 0.2
    });

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

fn image_path_to_data_url(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("Bild konnte nicht gelesen werden: {error}"))?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let mime = guess_image_mime(path);
    Ok(format!("data:{mime};base64,{encoded}"))
}

fn guess_image_mime(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        _ => "application/octet-stream",
    }
}
