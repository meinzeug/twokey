use std::{
    io::Write,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use crate::session;

pub fn insert_text(text: &str) -> Result<(), String> {
    let session_type = session_type();
    if session_type == "wayland" {
        return insert_text_wayland(text);
    }

    ensure_x11_automation("Texteinfuegung")?;
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

pub fn read_selected_text() -> Result<String, String> {
    let session_type = session_type();
    if session_type == "wayland" {
        return read_selected_text_wayland();
    }

    ensure_x11_automation("Textauswahl lesen")?;
    let clipboard = ClipboardTool::detect()?;
    let previous_clipboard = clipboard.read().ok();

    let copy_result = Command::new("xdotool")
        .arg("key")
        .arg("ctrl+c")
        .status()
        .map_err(|error| format!("xdotool konnte nicht gestartet werden: {error}"))
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(format!("xdotool beendete sich mit Status {status}"))
            }
        });

    thread::sleep(Duration::from_millis(140));
    let selected_text = copy_result.and_then(|_| clipboard.read());

    if let Some(previous) = previous_clipboard {
        let _ = clipboard.write(&previous);
    }

    let selected_text = selected_text?.trim().to_string();
    if selected_text.is_empty() {
        return Err("Keine markierte Textauswahl gefunden".to_string());
    }

    Ok(selected_text)
}

pub fn replace_selected_text(text: &str) -> Result<(), String> {
    insert_text(text)
}

fn ensure_x11_automation(action: &str) -> Result<(), String> {
    let session_type = session_type();
    if session_type != "x11" {
        return Err(format!(
            "{action} ist aktuell nur unter X11 implementiert. Wayland folgt mit expliziten Portalen/Fallbacks."
        ));
    }

    if !command_exists("xdotool") {
        return Err("xdotool ist fuer X11-Desktop-Automation nicht installiert".to_string());
    }

    Ok(())
}

fn insert_text_wayland(text: &str) -> Result<(), String> {
    if command_exists("wtype") {
        let status = Command::new("wtype")
            .arg(text)
            .status()
            .map_err(|error| format!("wtype konnte nicht gestartet werden: {error}"))?;

        if status.success() {
            return Ok(());
        }

        return Err(format!("wtype beendete sich mit Status {status}"));
    }

    if command_exists("ydotool") {
        let status = Command::new("ydotool")
            .arg("type")
            .arg(text)
            .status()
            .map_err(|error| format!("ydotool konnte nicht gestartet werden: {error}"))?;

        if status.success() {
            return Ok(());
        }

        return Err(format!("ydotool beendete sich mit Status {status}"));
    }

    Err(
        "Wayland-Einfuegung benoetigt wtype oder ydotool. Installiere eines der Tools fuer sicheres Tippen ohne globale Hotkey-Umgehung."
            .to_string(),
    )
}

fn read_selected_text_wayland() -> Result<String, String> {
    if command_exists("wtype") && command_exists("wl-paste") {
        let status = Command::new("wtype")
            .args(["-M", "ctrl", "c", "-m", "ctrl"])
            .status()
            .map_err(|error| format!("wtype Copy-Shortcut konnte nicht gestartet werden: {error}"))?;

        if !status.success() {
            return Err(format!("wtype Copy-Shortcut beendete sich mit Status {status}"));
        }

        thread::sleep(Duration::from_millis(140));

        let output = Command::new("wl-paste")
            .arg("--no-newline")
            .output()
            .map_err(|error| format!("wl-paste konnte nicht gestartet werden: {error}"))?;

        if !output.status.success() {
            return Err("wl-paste konnte die Auswahl nicht lesen".to_string());
        }

        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if selected.is_empty() {
            return Err("Keine markierte Textauswahl gefunden".to_string());
        }

        return Ok(selected);
    }

    let detected = session::detect();
    let compositor = session::compositor_label(&detected.compositor);
    Err(format!(
        "Wayland-Textauswahl lesen erfordert wtype + wl-paste (Compositor: {compositor})."
    ))
}

fn session_type() -> String {
    std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".to_string())
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
