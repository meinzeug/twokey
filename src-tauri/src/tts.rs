use std::{io::Write, process::{Command, Stdio}};

use serde::Serialize;

use crate::{history, settings};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsDiagnostics {
    pub backend: String,
    pub available: bool,
    pub message: String,
}

pub fn speak_text(text: &str) -> Result<String, String> {
    let app_settings = settings::load().unwrap_or_default();
    if !app_settings.tts_enabled {
        return Err("TTS ist in den Settings deaktiviert".to_string());
    }

    let backend = select_backend(&app_settings.tts_backend)?;
    match backend.as_str() {
        "spd-say" => {
            let rate = ((app_settings.tts_speed - 1.0) * 60.0).round() as i32;
            run_command("spd-say", &["-r", &rate.to_string(), text])?;
        }
        "espeak-ng" => {
            let speed_wpm = (app_settings.tts_speed * 170.0).round().clamp(80.0, 360.0) as i32;
            run_command("espeak-ng", &["-s", &speed_wpm.to_string(), "-v", &app_settings.tts_voice, text])?;
        }
        "espeak" => {
            let speed_wpm = (app_settings.tts_speed * 170.0).round().clamp(80.0, 360.0) as i32;
            run_command("espeak", &["-s", &speed_wpm.to_string(), "-v", &app_settings.tts_voice, text])?;
        }
        _ => return Err(format!("Unbekanntes TTS-Backend: {backend}")),
    }

    let _ = history::record(history::AuditEvent {
        kind: "tts".to_string(),
        mode: Some("conversation".to_string()),
        provider: Some(backend.to_string()),
        input_text: Some(text.to_string()),
        output_text: None,
        metadata_json: None,
        success: true,
    });

    Ok(backend.to_string())
}

pub fn install_backend(backend: &str, sudo_password: Option<&str>) -> Result<String, String> {
    let package = backend_package_name(backend)?;

    if command_exists(backend) {
        return Ok(format!("TTS-Backend {backend} ist bereits installiert"));
    }

    if command_exists("apt-get") {
        run_privileged_command("apt-get", &["install", "-y", package], sudo_password)?;
    } else {
        return Err("Es konnte kein Paketmanager gefunden werden. Installiere das TTS-Backend manuell oder verwende ein System mit apt-get.".to_string());
    }

    if command_exists(backend) {
        Ok(format!("TTS-Backend {backend} wurde installiert"))
    } else {
        Err(format!("TTS-Backend {backend} konnte nicht verifiziert werden"))
    }
}

pub fn backend_status(backend: &str) -> TtsDiagnostics {
    if backend == "auto" {
        return diagnostics();
    }

    let available = command_exists(backend);
    TtsDiagnostics {
        backend: backend.to_string(),
        available,
        message: if available {
            format!("TTS-Backend {backend} ist installiert")
        } else {
            format!("TTS-Backend {backend} ist nicht installiert")
        },
    }
}

pub fn diagnostics() -> TtsDiagnostics {
    let app_settings = settings::load().unwrap_or_default();
    let selected = app_settings.tts_backend.clone();
    if selected != "auto" {
        let available = command_exists(&selected);
        return TtsDiagnostics {
            backend: selected.clone(),
            available,
            message: if available {
                format!("TTS bereit via {selected}")
            } else {
                format!("TTS-Backend {selected} ist nicht installiert")
            },
        };
    }

    if command_exists("spd-say") {
        return TtsDiagnostics {
            backend: "spd-say".to_string(),
            available: true,
            message: "TTS bereit via spd-say".to_string(),
        };
    }

    if command_exists("espeak-ng") {
        return TtsDiagnostics {
            backend: "espeak-ng".to_string(),
            available: true,
            message: "TTS bereit via espeak-ng".to_string(),
        };
    }

    if command_exists("espeak") {
        return TtsDiagnostics {
            backend: "espeak".to_string(),
            available: true,
            message: "TTS bereit via espeak".to_string(),
        };
    }

    TtsDiagnostics {
        backend: "none".to_string(),
        available: false,
        message: "Kein TTS-Backend gefunden. Installiere spd-say, espeak-ng oder espeak.".to_string(),
    }
}

fn select_backend(selected: &str) -> Result<String, String> {
    if selected == "auto" {
        if command_exists("spd-say") {
            return Ok("spd-say".to_string());
        }
        if command_exists("espeak-ng") {
            return Ok("espeak-ng".to_string());
        }
        if command_exists("espeak") {
            return Ok("espeak".to_string());
        }

        return Err("Kein TTS-Backend gefunden. Installiere spd-say, espeak-ng oder espeak.".to_string());
    }

    if command_exists(selected) {
        return Ok(selected.to_string());
    }

    Err(format!("Das gewaehlte TTS-Backend '{selected}' ist nicht installiert"))
}

fn backend_package_name(backend: &str) -> Result<&'static str, String> {
    match backend {
        "spd-say" => Ok("speech-dispatcher"),
        "espeak-ng" => Ok("espeak-ng"),
        "espeak" => Ok("espeak"),
        "auto" => Ok("espeak-ng"),
        _ => Err(format!("Unbekanntes TTS-Backend: {backend}")),
    }
}

fn run_privileged_command(program: &str, args: &[&str], sudo_password: Option<&str>) -> Result<(), String> {
    if is_root() {
        return run_command(program, args);
    }

    let password = sudo_password
        .map(str::trim)
        .filter(|password| !password.is_empty())
        .ok_or_else(|| "Für die Installation von TTS-Backends wird ein sudo-Passwort benötigt".to_string())?;

    if !command_exists("sudo") {
        return Err("sudo wurde nicht gefunden. Bitte installiere das TTS-Backend manuell oder starte TwoKey als root.".to_string());
    }

    let mut child = Command::new("sudo")
        .args(["-S", "-p", "", program])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("sudo konnte nicht gestartet werden: {error}"))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(format!("{password}\n").as_bytes())
            .map_err(|error| format!("sudo-Passwort konnte nicht uebergeben werden: {error}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("sudo-Installation konnte nicht abgeschlossen werden: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if !stderr.is_empty() { stderr } else if !stdout.is_empty() { stdout } else { format!("Exit-Code {}", output.status) };
        Err(detail)
    }
}

fn is_root() -> bool {
    Command::new("sh")
        .arg("-c")
        .arg("id -u")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim() == "0")
        .unwrap_or(false)
}

fn run_command(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .env_remove("PYTHONHOME")
        .env_remove("PYTHONPATH")
        .args(args)
        .status()
        .map_err(|error| format!("TTS-Befehl {program} konnte nicht gestartet werden: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("TTS-Befehl {program} beendete sich mit Status {status}"))
    }
}

fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
