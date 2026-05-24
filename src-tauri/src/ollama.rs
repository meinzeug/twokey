use serde::{Deserialize, Serialize};

const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_MODEL: &str = "qwen2.5:3b";

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

pub fn chat(prompt: &str) -> Result<String, String> {
    let base_url = std::env::var("TWOKEY_OLLAMA_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string());
    let model = std::env::var("TWOKEY_OLLAMA_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());

    let request = ChatRequest {
        model: model.clone(),
        stream: false,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "Du bist TwoKey, ein knapper Linux-Desktop-Assistent. Antworte direkt, hilfreich und auf Deutsch, wenn der Nutzer Deutsch verwendet.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
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
