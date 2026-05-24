use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{history, hotkeys, provider, settings, stt, tts, toolchains};

pub fn export_debug_report(user_notes: &str) -> Result<String, String> {
    let report = build_debug_report(user_notes)?;
    let path = pick_save_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("Debug-Verzeichnis konnte nicht erstellt werden: {error}"))?;
    }

    fs::write(&path, report).map_err(|error| format!("Debug-Datei konnte nicht geschrieben werden: {error}"))?;

    Ok(path.to_string_lossy().to_string())
}

fn build_debug_report(user_notes: &str) -> Result<String, String> {
    let app_settings = settings::load().unwrap_or_default();
    let capabilities = hotkeys::capabilities();
    let whisper = stt::local_whisper_diagnostics();
    let tts = tts::diagnostics();
    let providers = provider::list();
    let history_entries = history::list_recent(20).unwrap_or_default();

    let report = serde_json::json!({
        "generatedAt": now_iso_like(),
        "appVersion": env!("CARGO_PKG_VERSION"),
        "system": system_snapshot(),
        "environment": environment_snapshot(),
        "installedCommands": installed_commands_snapshot(),
        "twoKey": {
            "settings": app_settings,
            "desktopCapabilities": capabilities,
            "whisperDiagnostics": whisper,
            "ttsDiagnostics": tts,
            "providers": providers,
            "toolchains": toolchains::load_toolchains().unwrap_or_default(),
            "recentHistory": history_entries,
        },
        "userNotes": user_notes,
    });

    Ok(format!("{}\n", serde_json::to_string_pretty(&report).map_err(|error| format!("Debug-Bericht konnte nicht serialisiert werden: {error}"))?))
}

fn system_snapshot() -> serde_json::Value {
    serde_json::json!({
        "hostname": command_output("hostname", &[]).unwrap_or_else(|| "unknown".to_string()),
        "kernel": command_output("uname", &["-a"]).unwrap_or_else(|| "unknown".to_string()),
        "osRelease": fs::read_to_string("/etc/os-release").unwrap_or_default(),
    })
}

fn environment_snapshot() -> serde_json::Value {
    serde_json::json!({
        "sessionType": std::env::var("XDG_SESSION_TYPE").unwrap_or_default(),
        "desktop": std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
        "session": std::env::var("DESKTOP_SESSION").unwrap_or_default(),
        "display": std::env::var("DISPLAY").unwrap_or_default(),
        "waylandDisplay": std::env::var("WAYLAND_DISPLAY").unwrap_or_default(),
        "user": std::env::var("USER").unwrap_or_default(),
    })
}

fn installed_commands_snapshot() -> Vec<serde_json::Value> {
    let commands = [
        ("ollama", &["--version"][..]),
        ("ffmpeg", &["-version"][..]),
        ("whisper", &["--help"][..]),
        ("whisper-cli", &["--help"][..]),
        ("espeak-ng", &["--version"][..]),
        ("espeak", &["--version"][..]),
        ("sudo", &["-V"][..]),
        ("pkexec", &["--version"][..]),
        ("kdialog", &["--version"][..]),
        ("zenity", &["--version"][..]),
        ("pw-record", &["--version"][..]),
        ("arecord", &["--version"][..]),
        ("python3", &["--version"][..]),
    ];

    commands
        .iter()
        .map(|(name, args)| {
            serde_json::json!({
                "name": name,
                "installed": command_exists(name),
                "version": command_output(name, args).unwrap_or_else(|| "unknown".to_string()),
            })
        })
        .collect()
}

fn pick_save_path() -> Result<PathBuf, String> {
    let default_name = format!("twokey-debug-{}.json", timestamp_compact());

    if command_exists("kdialog") {
        let output = Command::new("kdialog")
            .arg("--getsavefilename")
            .arg(default_name)
            .output()
            .map_err(|error| format!("kdialog konnte nicht gestartet werden: {error}"))?;
        return path_from_output(output.stdout);
    }

    if command_exists("zenity") {
        let output = Command::new("zenity")
            .arg("--file-selection")
            .arg("--save")
            .arg("--confirm-overwrite")
            .arg("--filename")
            .arg(&default_name)
            .arg("--title=Debug-Datei speichern")
            .output()
            .map_err(|error| format!("zenity konnte nicht gestartet werden: {error}"))?;
        return path_from_output(output.stdout);
    }

    Err("Kein Dateidialog gefunden. Installiere kdialog oder zenity.".to_string())
}

fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                Some(if !stdout.is_empty() { stdout } else { stderr })
            } else {
                None
            }
        })
        .filter(|value| !value.is_empty())
}

fn path_from_output(output: Vec<u8>) -> Result<PathBuf, String> {
    let value = String::from_utf8_lossy(&output).trim().to_string();
    if value.is_empty() {
        return Err("Keine Datei ausgewaehlt".to_string());
    }

    Ok(PathBuf::from(value))
}

fn timestamp_compact() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn now_iso_like() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    seconds.to_string()
}