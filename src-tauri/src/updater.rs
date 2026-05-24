use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};

const LATEST_RELEASE_URL: &str = "https://api.github.com/repos/meinzeug/twokey/releases/latest";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub release_url: Option<String>,
    pub message: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub fn check() -> Result<UpdateStatus, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let response = reqwest::blocking::Client::new()
        .get(LATEST_RELEASE_URL)
        .header("User-Agent", "twokey-ai")
        .send()
        .map_err(|error| format!("Update-Pruefung konnte GitHub nicht erreichen: {error}"))?;

    if response.status().as_u16() == 404 {
        return Ok(UpdateStatus {
            current_version,
            latest_version: None,
            update_available: false,
            release_url: None,
            message: "Noch keine GitHub Releases gefunden.".to_string(),
        });
    }

    if !response.status().is_success() {
        return Err(format!("GitHub Releases antwortete mit HTTP {}", response.status()));
    }

    let release: GithubRelease = response
        .json()
        .map_err(|error| format!("GitHub Release konnte nicht gelesen werden: {error}"))?;
    let latest_clean = release.tag_name.trim_start_matches('v').to_string();
    let update_available = latest_clean != current_version;

    Ok(UpdateStatus {
        current_version,
        latest_version: Some(release.tag_name),
        update_available,
        release_url: Some(release.html_url),
        message: if update_available {
            "Neue Version verfuegbar. Download und Start ist moeglich.".to_string()
        } else {
            "Du nutzt die aktuelle Release-Version.".to_string()
        },
    })
}

pub fn install_latest_appimage() -> Result<String, String> {
    let release = fetch_latest_release()?;
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name.to_ascii_lowercase().ends_with(".appimage"))
        .ok_or_else(|| "Keine AppImage-Datei im neuesten Release gefunden".to_string())?;

    let response = reqwest::blocking::Client::new()
        .get(&asset.browser_download_url)
        .header("User-Agent", "twokey-ai")
        .send()
        .map_err(|error| format!("Update-Datei konnte nicht heruntergeladen werden: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("Download antwortete mit HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .map_err(|error| format!("Update-Datei konnte nicht gelesen werden: {error}"))?;

    let target = update_binary_path()?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Update-Verzeichnis konnte nicht erstellt werden: {error}"))?;
    }

    fs::write(&target, &bytes).map_err(|error| format!("Update-Datei konnte nicht gespeichert werden: {error}"))?;
    let mut perms = fs::metadata(&target)
        .map_err(|error| format!("Dateiberechtigungen konnten nicht gelesen werden: {error}"))?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&target, perms).map_err(|error| format!("Datei konnte nicht ausführbar gemacht werden: {error}"))?;

    Command::new(&target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("Aktualisierte App konnte nicht gestartet werden: {error}"))?;

    Ok(format!("Update {} heruntergeladen und gestartet.", release.tag_name))
}

fn fetch_latest_release() -> Result<GithubRelease, String> {
    let response = reqwest::blocking::Client::new()
        .get(LATEST_RELEASE_URL)
        .header("User-Agent", "twokey-ai")
        .send()
        .map_err(|error| format!("Update-Pruefung konnte GitHub nicht erreichen: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("GitHub Releases antwortete mit HTTP {}", response.status()));
    }

    response
        .json()
        .map_err(|error| format!("GitHub Release konnte nicht gelesen werden: {error}"))
}

fn update_binary_path() -> Result<PathBuf, String> {
    let base = if let Ok(value) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(value)
    } else {
        let home = std::env::var("HOME").map_err(|_| "HOME ist nicht gesetzt".to_string())?;
        PathBuf::from(home).join(".local").join("share")
    };

    Ok(base.join("twokey").join("bin").join("twokey-ai.AppImage"))
}
