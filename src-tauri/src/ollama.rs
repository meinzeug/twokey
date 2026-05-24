use std::path::Path;

use base64::Engine;
use serde::{Deserialize, Serialize};

const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    stream: bool,
    messages: Vec<ChatMessage>,
    options: ChatOptions,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
}

#[derive(Serialize)]
struct ChatOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

pub fn chat_with_model(prompt: &str, model: &str) -> Result<String, String> {
    let base_url = std::env::var("TWOKEY_OLLAMA_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string());

    let request = ChatRequest {
        model: model.to_string(),
        stream: false,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "Du bist TwoKey, ein knapper Linux-Desktop-Assistent. Antworte direkt, hilfreich und auf Deutsch, wenn der Nutzer Deutsch verwendet.".to_string(),
                images: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
                images: None,
            },
        ],
        options: ChatOptions {
            temperature: 0.3,
            num_predict: 384,
        },
    };

    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let response = reqwest::blocking::Client::new()
        .post(url)
        .json(&request)
        .send()
        .map_err(|error| {
            format!(
                "Ollama ist nicht erreichbar. Pruefe `systemctl status ollama` und ob Modell `{model}` installiert ist. Details: {error}"
            )
        })?;

    if !response.status().is_success() {
        return Err(format!("Ollama antwortete mit HTTP {}", response.status()));
    }

    let payload: ChatResponse = response
        .json()
        .map_err(|error| format!("Ollama-Antwort konnte nicht gelesen werden: {error}"))?;

    let content = payload.message.content.trim().to_string();
    if content.is_empty() {
        return Err("Ollama lieferte eine leere Antwort".to_string());
    }

    Ok(content)
}

pub fn chat_with_model_and_image(prompt: &str, model: &str, image_path: &Path) -> Result<String, String> {
    let base_url = std::env::var("TWOKEY_OLLAMA_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string());
    let image_bytes = std::fs::read(image_path).map_err(|error| format!("Bild konnte nicht gelesen werden: {error}"))?;
    let image_base64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);

    let request = ChatRequest {
        model: model.to_string(),
        stream: false,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "Du bist TwoKey, ein knapper Linux-Desktop-Assistent mit Vision-Unterstuetzung.".to_string(),
                images: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
                images: Some(vec![image_base64]),
            },
        ],
        options: ChatOptions {
            temperature: 0.3,
            num_predict: 384,
        },
    };

    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let response = reqwest::blocking::Client::new()
        .post(url)
        .json(&request)
        .send()
        .map_err(|error| {
            format!(
                "Ollama Vision ist nicht erreichbar. Pruefe `systemctl status ollama` und ob Modell `{model}` Vision unterstuetzt. Details: {error}"
            )
        })?;

    if !response.status().is_success() {
        return Err(format!("Ollama Vision antwortete mit HTTP {}", response.status()));
    }

    let payload: ChatResponse = response
        .json()
        .map_err(|error| format!("Ollama-Vision-Antwort konnte nicht gelesen werden: {error}"))?;

    let content = payload.message.content.trim().to_string();
    if content.is_empty() {
        return Err("Ollama Vision lieferte eine leere Antwort".to_string());
    }

    Ok(content)
}
