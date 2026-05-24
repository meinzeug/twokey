use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainDryRun {
    pub toolchain_id: String,
    pub toolchain_name: String,
    pub executable: bool,
    pub steps: Vec<String>,
    pub warnings: Vec<String>,
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

    run_chain_steps(&chain)?;

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

pub fn run_by_id(id: &str) -> Result<String, String> {
    let toolchains = load_toolchains()?;
    let chain = toolchains
        .into_iter()
        .find(|chain| chain.id == id)
        .ok_or_else(|| format!("Toolchain '{}' wurde nicht gefunden", id))?;

    run_chain_steps(&chain)?;
    let message = format!("Toolchain ausgefuehrt: {}", chain.name);

    let _ = history::record(history::AuditEvent {
        kind: "toolchain".to_string(),
        mode: Some("manual".to_string()),
        provider: None,
        input_text: Some(id.to_string()),
        output_text: Some(message.clone()),
        metadata_json: Some(format!("{{\"toolchainId\":\"{}\",\"source\":\"manual\"}}", chain.id)),
        success: true,
    });

    Ok(message)
}

pub fn dry_run_by_id(id: &str) -> Result<ToolchainDryRun, String> {
    let toolchains = load_toolchains()?;
    let chain = toolchains
        .into_iter()
        .find(|chain| chain.id == id)
        .ok_or_else(|| format!("Toolchain '{}' wurde nicht gefunden", id))?;

    let mut steps = Vec::new();
    let mut warnings = Vec::new();

    for (index, step) in chain.steps.iter().enumerate() {
        let step_no = index + 1;
        steps.push(format!("{step_no}. {} -> {}", step.kind, step.value));
        if let Some(warning) = step_warning(step) {
            warnings.push(format!("Step {step_no}: {warning}"));
        }
    }

    Ok(ToolchainDryRun {
        toolchain_id: chain.id,
        toolchain_name: chain.name,
        executable: warnings.is_empty(),
        steps,
        warnings,
    })
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

pub fn save_toolchains(toolchains: &[Toolchain]) -> Result<(), String> {
    validate_toolchains(toolchains)?;
    let path = toolchains_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Toolchain-Verzeichnis konnte nicht erstellt werden: {error}"))?;
    }

    let content = serde_json::to_string_pretty(toolchains)
        .map_err(|error| format!("Toolchains konnten nicht serialisiert werden: {error}"))?;
    fs::write(path, content).map_err(|error| format!("Toolchains konnten nicht geschrieben werden: {error}"))
}

fn validate_toolchains(toolchains: &[Toolchain]) -> Result<(), String> {
    for chain in toolchains {
        if chain.id.trim().is_empty() {
            return Err("Jede Toolchain benoetigt eine ID".to_string());
        }
        if chain.name.trim().is_empty() {
            return Err(format!("Toolchain '{}' hat keinen Namen", chain.id));
        }
        if chain.trigger.trim().is_empty() {
            return Err(format!("Toolchain '{}' hat keinen Trigger", chain.id));
        }
        if chain.steps.is_empty() {
            return Err(format!("Toolchain '{}' hat keine Schritte", chain.id));
        }

        for step in &chain.steps {
            let kind = step.kind.trim();
            if !matches!(kind, "open_url" | "open_app" | "shell" | "wait_ms" | "check_command") {
                return Err(format!(
                    "Toolchain '{}' enthaelt ungueltigen Step-Typ '{}'",
                    chain.id, step.kind
                ));
            }
            if step.value.trim().is_empty() {
                return Err(format!(
                    "Toolchain '{}' enthaelt leeren Step-Wert fuer '{}'",
                    chain.id, step.kind
                ));
            }

            if kind == "wait_ms" {
                step.value
                    .trim()
                    .parse::<u64>()
                    .map_err(|_| format!("Toolchain '{}' hat ungueltiges wait_ms '{}'; erwartet Millisekunden", chain.id, step.value))?;
            }

            if let Some(warning) = step_warning(step) {
                return Err(format!("Toolchain '{}' wurde blockiert: {warning}", chain.id));
            }
        }
    }

    Ok(())
}

fn run_chain_steps(chain: &Toolchain) -> Result<(), String> {
    for step in &chain.steps {
        run_step(step)?;
    }

    Ok(())
}

fn run_step(step: &ToolchainStep) -> Result<(), String> {
    if let Some(warning) = step_warning(step) {
        return Err(format!("Step wurde aus Sicherheitsgruenden blockiert: {warning}"));
    }

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
        "wait_ms" => {
            let millis = step
                .value
                .trim()
                .parse::<u64>()
                .map_err(|_| format!("wait_ms erwartet Millisekunden als Zahl, bekam '{}'", step.value))?;
            thread::sleep(Duration::from_millis(millis.min(60_000)));
            Ok(())
        }
        "check_command" => {
            let command = step.value.trim();
            let status = Command::new("sh")
                .arg("-c")
                .arg(format!("command -v {command} >/dev/null 2>&1"))
                .status()
                .map_err(|error| format!("check_command konnte nicht gestartet werden: {error}"))?;

            if status.success() {
                Ok(())
            } else {
                Err(format!("check_command fehlgeschlagen: '{command}' ist nicht verfuegbar"))
            }
        }
        _ => Err(format!("Unbekannter Toolchain-Step: {}", step.kind)),
    }
}

fn step_warning(step: &ToolchainStep) -> Option<String> {
    if step.kind != "shell" {
        return None;
    }

    let lower = step.value.to_ascii_lowercase();
    let dangerous_patterns = [
        "rm -rf /",
        "mkfs",
        "dd if=",
        "shutdown",
        "reboot",
        ":(){",
    ];

    for pattern in dangerous_patterns {
        if lower.contains(pattern) {
            return Some(format!("gefaehrliches shell-Muster erkannt ('{pattern}')"));
        }
    }

    None
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
