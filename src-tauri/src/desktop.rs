use std::{
    io::Write,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

pub fn insert_text(text: &str) -> Result<(), String> {
    let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".to_string());
    if session_type != "x11" {
        return Err("Texteinfügung ist aktuell nur unter X11 implementiert. Wayland folgt mit expliziten Portalen/Fallbacks.".to_string());
    }

    if !command_exists("xdotool") {
        return Err("xdotool ist fuer X11-Texteinfügung nicht installiert".to_string());
    }

    let clipboard = ClipboardTool::detect()?;
    let previous_clipboard = clipboard.read().ok();

    clipboard.write(text)?;
    thread::sleep(Duration::from_millis(80));

    let paste_result = Command::new("xdotool")
        .arg("key")
        .arg("ctrl+v")
        .status()
        .map_err(|error| format!("xdotool konnte nicht gestartet werden: {error}"))
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(format!("xdotool beendete sich mit Status {status}"))
            }
        });

    thread::sleep(Duration::from_millis(120));

    if let Some(previous) = previous_clipboard {
        let _ = clipboard.write(&previous);
    }

    paste_result
}

enum ClipboardTool {
    Xclip,
    Xsel,
}

impl ClipboardTool {
    fn detect() -> Result<Self, String> {
        if command_exists("xclip") {
            return Ok(Self::Xclip);
        }

        if command_exists("xsel") {
            return Ok(Self::Xsel);
        }

        Err("Installiere xclip oder xsel fuer X11-Clipboard-Automation".to_string())
    }

    fn read(&self) -> Result<String, String> {
        let output = match self {
            Self::Xclip => Command::new("xclip")
                .arg("-selection")
                .arg("clipboard")
                .arg("-o")
                .output(),
            Self::Xsel => Command::new("xsel").arg("--clipboard").arg("--output").output(),
        }
        .map_err(|error| format!("Clipboard konnte nicht gelesen werden: {error}"))?;

        if !output.status.success() {
            return Err("Clipboard ist leer oder konnte nicht gelesen werden".to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn write(&self, text: &str) -> Result<(), String> {
        let mut child = match self {
            Self::Xclip => Command::new("xclip")
                .arg("-selection")
                .arg("clipboard")
                .stdin(Stdio::piped())
                .spawn(),
            Self::Xsel => Command::new("xsel")
                .arg("--clipboard")
                .arg("--input")
                .stdin(Stdio::piped())
                .spawn(),
        }
        .map_err(|error| format!("Clipboard-Tool konnte nicht gestartet werden: {error}"))?;

        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "Clipboard-Tool stdin ist nicht verfuegbar".to_string())?;
        stdin
            .write_all(text.as_bytes())
            .map_err(|error| format!("Clipboard konnte nicht geschrieben werden: {error}"))?;

        let status = child
            .wait()
            .map_err(|error| format!("Clipboard-Tool konnte nicht beendet werden: {error}"))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("Clipboard-Tool beendete sich mit Status {status}"))
        }
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
