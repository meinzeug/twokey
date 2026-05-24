use std::{fs, path::PathBuf};

pub fn set_enabled(enabled: bool) -> Result<(), String> {
    let path = autostart_path()?;
    if enabled {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("Autostart-Verzeichnis konnte nicht erstellt werden: {error}"))?;
        }

        let exe = std::env::current_exe().map_err(|error| format!("Aktuelle App konnte nicht ermittelt werden: {error}"))?;
        let desktop = format!(
            "[Desktop Entry]\nType=Application\nName=TwoKey Linux AI Assistant\nExec={}\nTerminal=false\nX-GNOME-Autostart-enabled=true\nCategories=Utility;\n",
            exe.to_string_lossy()
        );
        fs::write(path, desktop).map_err(|error| format!("Autostart-Datei konnte nicht geschrieben werden: {error}"))?;
    } else if path.exists() {
        fs::remove_file(path).map_err(|error| format!("Autostart-Datei konnte nicht entfernt werden: {error}"))?;
    }

    Ok(())
}

fn autostart_path() -> Result<PathBuf, String> {
    let base = if let Ok(value) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(value)
    } else {
        let home = std::env::var("HOME").map_err(|_| "HOME ist nicht gesetzt".to_string())?;
        PathBuf::from(home).join(".config")
    };

    Ok(base.join("autostart").join("twokey-ai.desktop"))
}
