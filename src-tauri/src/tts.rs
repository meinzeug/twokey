use std::process::Command;

use crate::{history, settings};

pub fn speak_text(text: &str) -> Result<String, String> {
    let app_settings = settings::load().unwrap_or_default();
    if !app_settings.tts_enabled {
        return Err("TTS ist in den Settings deaktiviert".to_string());
    }

    let backend = if command_exists("spd-say") {
        let rate = ((app_settings.tts_speed - 1.0) * 60.0).round() as i32;
        run_command("spd-say", &["-r", &rate.to_string(), text])?;
        "spd-say"
    } else if command_exists("espeak-ng") {
        let speed_wpm = (app_settings.tts_speed * 170.0).round().clamp(80.0, 360.0) as i32;
        run_command("espeak-ng", &["-s", &speed_wpm.to_string(), "-v", &app_settings.tts_voice, text])?;
        "espeak-ng"
    } else if command_exists("espeak") {
        let speed_wpm = (app_settings.tts_speed * 170.0).round().clamp(80.0, 360.0) as i32;
        run_command("espeak", &["-s", &speed_wpm.to_string(), "-v", &app_settings.tts_voice, text])?;
        "espeak"
    } else {
        return Err("Kein TTS-Backend gefunden. Installiere spd-say, espeak-ng oder espeak.".to_string());
    };

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

fn run_command(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
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
