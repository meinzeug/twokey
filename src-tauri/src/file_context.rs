use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContext {
    pub path: String,
    pub name: String,
    pub kind: String,
    pub summary: String,
    pub extracted_text: Option<String>,
    pub cache_path: Option<String>,
}

pub fn pick_and_load() -> Result<FileContext, String> {
    let path = pick_file()?;
    load_path(&path)
}

fn pick_file() -> Result<PathBuf, String> {
    if command_exists("kdialog") {
        let output = Command::new("kdialog")
            .arg("--getopenfilename")
            .output()
            .map_err(|error| format!("kdialog konnte nicht gestartet werden: {error}"))?;
        return path_from_output(output.stdout);
    }

    if command_exists("zenity") {
        let output = Command::new("zenity")
            .arg("--file-selection")
            .arg("--title=Dateikontext hinzufuegen")
            .output()
            .map_err(|error| format!("zenity konnte nicht gestartet werden: {error}"))?;
        return path_from_output(output.stdout);
    }

    Err("Kein Dateidialog gefunden. Installiere kdialog oder zenity.".to_string())
}

fn load_path(path: &Path) -> Result<FileContext, String> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("Datei")
        .to_string();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match extension.as_str() {
        "txt" | "md" | "markdown" | "json" | "csv" => load_text_file(path, name),
        "pdf" => load_pdf_file(path, name),
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" => load_image_file(path, name),
        _ => Err(format!("Dateityp .{extension} wird in Phase 9 noch nicht unterstuetzt")),
    }
}

fn load_text_file(path: &Path, name: String) -> Result<FileContext, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("Textdatei konnte nicht gelesen werden: {error}"))?;
    let cache_path = write_cache(&name, &text)?;
    Ok(FileContext {
        path: path.to_string_lossy().to_string(),
        name,
        kind: "text".to_string(),
        summary: format!("Textkontext geladen, {} Zeichen", text.chars().count()),
        extracted_text: Some(limit_text(text)),
        cache_path: Some(cache_path.to_string_lossy().to_string()),
    })
}

fn load_pdf_file(path: &Path, name: String) -> Result<FileContext, String> {
    if !command_exists("pdftotext") {
        return Err("pdftotext ist nicht installiert. Installiere poppler-utils.".to_string());
    }

    let output = Command::new("pdftotext")
        .arg(path)
        .arg("-")
        .output()
        .map_err(|error| format!("PDF konnte nicht extrahiert werden: {error}"))?;

    if !output.status.success() {
        return Err("pdftotext konnte dieses PDF nicht lesen".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let cache_path = write_cache(&name, &text)?;
    Ok(FileContext {
        path: path.to_string_lossy().to_string(),
        name,
        kind: "pdf".to_string(),
        summary: format!("PDF-Kontext geladen, {} Zeichen extrahiert", text.chars().count()),
        extracted_text: Some(limit_text(text)),
        cache_path: Some(cache_path.to_string_lossy().to_string()),
    })
}

fn load_image_file(path: &Path, name: String) -> Result<FileContext, String> {
    let mime = Command::new("file")
        .arg("--brief")
        .arg("--mime-type")
        .arg(path)
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "image/unknown".to_string())
        .trim()
        .to_string();

    Ok(FileContext {
        path: path.to_string_lossy().to_string(),
        name,
        kind: "image".to_string(),
        summary: format!("Bildkontext vorgemerkt ({mime}). Vision-Analyse folgt mit Vision-Provider."),
        extracted_text: None,
        cache_path: None,
    })
}

fn write_cache(name: &str, text: &str) -> Result<PathBuf, String> {
    let mut dir = xdg_cache_dir();
    dir.push("twokey-ai");
    dir.push("file-contexts");
    fs::create_dir_all(&dir).map_err(|error| format!("Cache-Verzeichnis konnte nicht erstellt werden: {error}"))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("Systemzeit konnte nicht gelesen werden: {error}"))?
        .as_millis();
    let safe_name = name.replace('/', "_");
    dir.push(format!("{timestamp}-{safe_name}.txt"));
    fs::write(&dir, text).map_err(|error| format!("Kontextcache konnte nicht geschrieben werden: {error}"))?;
    Ok(dir)
}

fn limit_text(text: String) -> String {
    const MAX_CHARS: usize = 12_000;
    if text.chars().count() <= MAX_CHARS {
        return text;
    }

    text.chars().take(MAX_CHARS).collect::<String>()
}

fn path_from_output(output: Vec<u8>) -> Result<PathBuf, String> {
    let value = String::from_utf8_lossy(&output).trim().to_string();
    if value.is_empty() {
        return Err("Keine Datei ausgewaehlt".to_string());
    }

    Ok(PathBuf::from(value))
}

fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn xdg_cache_dir() -> PathBuf {
    if let Ok(value) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(value);
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".cache")
}
