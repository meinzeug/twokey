use std::{
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
pub struct AudioRecorder {
    active: Option<ActiveRecording>,
}

struct ActiveRecording {
    child: Child,
    path: PathBuf,
}

impl AudioRecorder {
    pub fn start(&mut self) -> Result<PathBuf, String> {
        if self.active.is_some() {
            return Err("Audioaufnahme läuft bereits".to_string());
        }

        let path = next_recording_path()?;
        let Some(mut command) = recorder_command(&path) else {
            return Err("Kein Recorder gefunden. Installiere pw-record, parec oder arecord.".to_string());
        };

        let child = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("Audioaufnahme konnte nicht gestartet werden: {error}"))?;

        self.active = Some(ActiveRecording {
            child,
            path: path.clone(),
        });

        Ok(path)
    }

    pub fn stop(&mut self) -> Result<Option<PathBuf>, String> {
        let Some(mut active) = self.active.take() else {
            return Ok(None);
        };

        let _ = active.child.kill();
        let _ = active.child.wait();

        Ok(Some(active.path))
    }

    pub fn cancel(&mut self) -> Result<(), String> {
        if let Some(path) = self.stop()? {
            let _ = fs::remove_file(path);
        }

        Ok(())
    }
}

fn recorder_command(path: &PathBuf) -> Option<Command> {
    if command_exists("pw-record") {
        let mut command = Command::new("pw-record");
        command
            .arg("--rate")
            .arg("16000")
            .arg("--channels")
            .arg("1")
            .arg("--format")
            .arg("s16")
            .arg(path);
        return Some(command);
    }

    if command_exists("parec") {
        let mut command = Command::new("parec");
        command
            .arg("--file-format=wav")
            .arg("--rate=16000")
            .arg("--channels=1")
            .arg(path);
        return Some(command);
    }

    if command_exists("arecord") {
        let mut command = Command::new("arecord");
        command
            .arg("-f")
            .arg("S16_LE")
            .arg("-r")
            .arg("16000")
            .arg("-c")
            .arg("1")
            .arg(path);
        return Some(command);
    }

    None
}

fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn next_recording_path() -> Result<PathBuf, String> {
    let mut dir = xdg_cache_dir();
    dir.push("twokey-ai");
    dir.push("recordings");
    fs::create_dir_all(&dir).map_err(|error| format!("Cache-Verzeichnis konnte nicht erstellt werden: {error}"))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("Systemzeit konnte nicht gelesen werden: {error}"))?
        .as_millis();

    dir.push(format!("twokey-{timestamp}.wav"));
    Ok(dir)
}

fn xdg_cache_dir() -> PathBuf {
    if let Ok(value) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(value);
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".cache")
}
