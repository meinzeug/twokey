use std::{fs, path::Path, process::Command};

use reqwest::blocking::multipart;
use serde::Deserialize;

use crate::{history, secrets, settings};

pub struct Transcript {
    pub text: String,
    pub provider: String,
}

#[derive(Deserialize)]
struct OpenAITranscriptionResponse {
    text: String,
}

pub fn transcribe(audio_path: &Path) -> Result<Transcript, String> {
    let app_settings = settings::load().unwrap_or_default();
    let result = match app_settings.stt_provider.as_str() {
        "local-whisper" => transcribe_with_local_whisper(audio_path, &app_settings.default_language),
        "external-command" => {
            let command_template = std::env::var("TWOKEY_STT_COMMAND")
                .map_err(|_| "TWOKEY_STT_COMMAND ist nicht gesetzt".to_string());
            command_template.and_then(|template| transcribe_with_command(audio_path, &template))
        }
        "openai-compatible" => transcribe_with_openai_compatible(audio_path),
        _ => {
            if let Ok(command_template) = std::env::var("TWOKEY_STT_COMMAND") {
                transcribe_with_command(audio_path, &command_template)
            } else {
                mock_transcribe(audio_path)
            }
        }
    };

    let _ = history::record(history::AuditEvent {
        kind: "stt".to_string(),
        mode: None,
        provider: result.as_ref().ok().map(|transcript| transcript.provider.clone()),
        input_text: Some(audio_path.to_string_lossy().to_string()),
        output_text: result.as_ref().ok().map(|transcript| transcript.text.clone()),
        metadata_json: None,
        success: result.is_ok(),
    });

    result
}

fn transcribe_with_local_whisper(audio_path: &Path, language: &str) -> Result<Transcript, String> {
    let commands: [Vec<String>; 2] = [
        vec![
            "whisper-cli".to_string(),
            "-f".to_string(),
            audio_path.to_string_lossy().to_string(),
            "-l".to_string(),
            language.to_string(),
            "-nt".to_string(),
        ],
        vec![
            "whisper".to_string(),
            audio_path.to_string_lossy().to_string(),
            "--language".to_string(),
            language.to_string(),
            "--model".to_string(),
            "base".to_string(),
        ],
    ];

    for command in commands {
        let program = &command[0];
        if !command_exists(program) {
            continue;
        }

        let output = Command::new(program)
            .args(&command[1..])
            .output()
            .map_err(|error| format!("{program} konnte nicht gestartet werden: {error}"))?;

        if !output.status.success() {
            continue;
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            continue;
        }

        return Ok(Transcript {
            text,
            provider: "local-whisper".to_string(),
        });
    }

    Err("Kein lokales Whisper-CLI gefunden oder es lieferte kein Transkript".to_string())
}

fn transcribe_with_openai_compatible(audio_path: &Path) -> Result<Transcript, String> {
    let app_settings = settings::load().unwrap_or_default();
    let api_key = secrets::get_provider_api_key("openai-compatible")?;
    let endpoint = format!("{}/audio/transcriptions", app_settings.openai_base_url.trim_end_matches('/'));
    let audio_bytes = fs::read(audio_path).map_err(|error| format!("Audiodatei konnte nicht gelesen werden: {error}"))?;

    let file_part = multipart::Part::bytes(audio_bytes)
        .file_name("recording.wav")
        .mime_str("audio/wav")
        .map_err(|error| format!("Audio-MIME konnte nicht gesetzt werden: {error}"))?;

    let form = multipart::Form::new().text("model", "whisper-1").part("file", file_part);

    let response = reqwest::blocking::Client::new()
        .post(endpoint)
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .map_err(|error| format!("OpenAI-kompatibles STT konnte nicht erreicht werden: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("OpenAI-kompatibles STT antwortete mit HTTP {}", response.status()));
    }

    let payload: OpenAITranscriptionResponse = response
        .json()
        .map_err(|error| format!("STT-Antwort konnte nicht gelesen werden: {error}"))?;
    let text = payload.text.trim().to_string();
    if text.is_empty() {
        return Err("OpenAI-kompatibles STT lieferte kein Transkript".to_string());
    }

    Ok(Transcript {
        text,
        provider: "openai-compatible".to_string(),
    })
}

fn transcribe_with_command(audio_path: &Path, command_template: &str) -> Result<Transcript, String> {
    if !command_template.contains("{audio}") {
        return Err("TWOKEY_STT_COMMAND muss den Platzhalter {audio} enthalten".to_string());
    }

    let audio_arg = shell_escape(audio_path.to_string_lossy().as_ref());
    let command = command_template.replace("{audio}", &audio_arg);
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|error| format!("STT-Befehl konnte nicht gestartet werden: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("STT-Befehl fehlgeschlagen: {}", stderr.trim()));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return Err("STT-Befehl lieferte kein Transkript".to_string());
    }

    Ok(Transcript {
        text,
        provider: "external-command".to_string(),
    })
}

fn mock_transcribe(audio_path: &Path) -> Result<Transcript, String> {
    let bytes = fs::metadata(audio_path)
        .map_err(|error| format!("Audiodatei konnte nicht gelesen werden: {error}"))?
        .len();

    Ok(Transcript {
        text: format!(
            "Mock-Transkript fuer {} Audio. Echtes STT kann mit TWOKEY_STT_COMMAND aktiviert werden.",
            human_bytes(bytes)
        ),
        provider: "mock".to_string(),
    })
}

fn human_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let kib = bytes as f64 / 1024.0;
    if kib < 1024.0 {
        return format!("{kib:.1} KiB");
    }

    format!("{:.1} MiB", kib / 1024.0)
}

fn shell_escape(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
