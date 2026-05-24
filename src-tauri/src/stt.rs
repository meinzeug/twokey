use std::{fs, path::Path, process::Command};

pub struct Transcript {
    pub text: String,
    pub provider: String,
}

pub fn transcribe(audio_path: &Path) -> Result<Transcript, String> {
    if let Ok(command_template) = std::env::var("TWOKEY_STT_COMMAND") {
        return transcribe_with_command(audio_path, &command_template);
    }

    mock_transcribe(audio_path)
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
