use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};

use crate::history;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Toolchain {
    pub id: String,
    pub name: String,
    pub trigger: String,
    pub steps: Vec<ToolchainStep>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainStep {
    pub kind: String,
    pub value: String,
}

pub fn run_for_transcript(transcript: &str) -> Result<Option<String>, String> {
    let normalized = transcript.trim().to_lowercase();
    if normalized.is_empty() {
        return Ok(None);
    }

    let toolchains = load_toolchains()?;
    let Some(chain) = toolchains.into_iter().find(|chain| normalized.contains(&chain.trigger.to_lowercase())) else {
        return Ok(None);
    };

    for step in &chain.steps {
        run_step(step)?;
    }

    let message = format!("Toolchain ausgefuehrt: {}", chain.name);
    let _ = history::record(history::AuditEvent {
        kind: "toolchain".to_string(),
        mode: Some("conversation".to_string()),
        provider: None,
        input_text: Some(transcript.to_string()),
        output_text: Some(message.clone()),
        metadata_json: Some(format!("{{\"toolchainId\":\"{}\"}}", chain.id)),
        success: true,
    });

    Ok(Some(message))
}

pub fn load_toolchains() -> Result<Vec<Toolchain>, String> {
    let path = toolchains_path()?;
    if !path.exists() {
        let defaults = default_toolchains();
        let content = serde_json::to_string_pretty(&defaults)
            .map_err(|error| format!("Default-Toolchains konnten nicht serialisiert werden: {error}"))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Toolchain-Verzeichnis konnte nicht erstellt werden: {error}"))?;
        }
        fs::write(&path, content).map_err(|error| format!("Toolchains konnten nicht geschrieben werden: {error}"))?;
        return Ok(defaults);
    }

    let content = fs::read_to_string(&path).map_err(|error| format!("Toolchains konnten nicht gelesen werden: {error}"))?;
    serde_json::from_str::<Vec<Toolchain>>(&content)
        .map_err(|error| format!("Toolchains JSON ist ungueltig: {error}"))
}

fn run_step(step: &ToolchainStep) -> Result<(), String> {
    match step.kind.as_str() {
        "open_url" => {
            let status = Command::new("xdg-open")
                .arg(&step.value)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map_err(|error| format!("open_url konnte nicht gestartet werden: {error}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("open_url schlug fehl mit Status {status}"))
            }
        }
        "open_app" => {
            Command::new("sh")
                .arg("-c")
                .arg(format!("{} >/dev/null 2>&1 &", step.value))
                .status()
                .map_err(|error| format!("open_app konnte nicht gestartet werden: {error}"))?;
            Ok(())
        }
        "shell" => {
            let status = Command::new("sh")
                .arg("-c")
                .arg(&step.value)
                .status()
                .map_err(|error| format!("shell-step konnte nicht gestartet werden: {error}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("shell-step schlug fehl mit Status {status}"))
            }
        }
        _ => Err(format!("Unbekannter Toolchain-Step: {}", step.kind)),
    }
}

fn toolchains_path() -> Result<PathBuf, String> {
    let base = if let Ok(value) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(value)
    } else {
        let home = std::env::var("HOME").map_err(|_| "HOME ist nicht gesetzt".to_string())?;
        PathBuf::from(home).join(".config")
    };

    Ok(base.join("twokey-ai").join("toolchains.json"))
}

fn default_toolchains() -> Vec<Toolchain> {
    vec![
        Toolchain {
            id: "studio-setup".to_string(),
            name: "Studio Setup".to_string(),
            trigger: "ich bin jetzt im studio".to_string(),
            steps: vec![
                ToolchainStep {
                    kind: "open_app".to_string(),
                    value: "obs".to_string(),
                },
                ToolchainStep {
                    kind: "open_url".to_string(),
                    value: "https://calendar.google.com".to_string(),
                },
            ],
        },
        Toolchain {
            id: "bank-start".to_string(),
            name: "Bank App".to_string(),
            trigger: "oeffne meine bank app".to_string(),
            steps: vec![ToolchainStep {
                kind: "open_url".to_string(),
                value: "https://app.vivid.money".to_string(),
            }],
        },
    ]
}
