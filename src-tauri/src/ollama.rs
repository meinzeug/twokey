use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use base64::Engine;
use serde::{Deserialize, Serialize};

const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_CHAT_TIMEOUT_SECS: u64 = 60;
const DEFAULT_VISION_TIMEOUT_SECS: u64 = 75;

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
    let response = send_chat_request_with_retry(&url, &request, DEFAULT_CHAT_TIMEOUT_SECS, model)?;

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

pub fn is_ready() -> bool {
    let base_url = std::env::var("TWOKEY_OLLAMA_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string());
    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));

    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .ok()
        .and_then(|client| client.get(url).send().ok())
        .map(|response| response.status().is_success())
        .unwrap_or(false)
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
    let response = send_chat_request_with_retry(&url, &request, DEFAULT_VISION_TIMEOUT_SECS, model)?;

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

fn send_chat_request_with_retry(
    url: &str,
    request: &ChatRequest,
    timeout_secs: u64,
    model: &str,
) -> Result<reqwest::blocking::Response, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|error| format!("Ollama-Client konnte nicht erstellt werden: {error}"))?;

    match client.post(url).json(request).send() {
        Ok(response) => Ok(response),
        Err(first_error) => {
            if try_start_ollama_service() {
                std::thread::sleep(Duration::from_millis(900));
                return client.post(url).json(request).send().map_err(|retry_error| {
                    format!(
                        "Ollama ist nicht erreichbar. Pruefe `systemctl status ollama` und ob Modell `{model}` installiert ist. Details: {retry_error}"
                    )
                });
            }

            Err(format!(
                "Ollama ist nicht erreichbar. Pruefe `systemctl status ollama` und ob Modell `{model}` installiert ist. Details: {first_error}"
            ))
        }
    }
}

fn try_start_ollama_service() -> bool {
    let has_ollama = Command::new("sh")
        .arg("-c")
        .arg("command -v ollama >/dev/null 2>&1")
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if !has_ollama {
        return false;
    }

    Command::new("ollama")
        .arg("serve")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}
