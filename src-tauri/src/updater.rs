use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::settings;

const LATEST_RELEASE_URL: &str = "https://api.github.com/repos/meinzeug/twokey/releases/latest";
const RELEASES_URL: &str = "https://api.github.com/repos/meinzeug/twokey/releases";

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
    prerelease: bool,
    draft: bool,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub fn check() -> Result<UpdateStatus, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let channel = settings::load().unwrap_or_default().update_channel;
    let release = match fetch_release_for_channel(&channel) {
        Ok(Some(release)) => release,
        Ok(None) => {
            return Ok(UpdateStatus {
                current_version,
                latest_version: None,
                update_available: false,
                release_url: None,
                message: format!("Kein Release fuer Kanal '{channel}' gefunden."),
            });
        }
        Err(error) => return Err(error),
    };

    let latest_clean = release.tag_name.trim_start_matches('v').to_string();
    let update_available = latest_clean != current_version;

    Ok(UpdateStatus {
        current_version,
        latest_version: Some(release.tag_name),
        update_available,
        release_url: Some(release.html_url),
        message: if update_available {
            format!("Neue Version im Kanal '{channel}' verfuegbar. Download und Start ist moeglich.")
        } else {
            format!("Du nutzt die aktuelle Version fuer Kanal '{channel}'.")
        },
    })
}

pub fn install_latest_appimage() -> Result<String, String> {
    let channel = settings::load().unwrap_or_default().update_channel;
    let release = fetch_release_for_channel(&channel)?
        .ok_or_else(|| format!("Kein Release fuer Kanal '{channel}' gefunden"))?;
    let asset = release
        .assets
        .iter()
        .find(|asset| {
            let name = asset.name.to_ascii_lowercase();
            name.ends_with(".appimage") && name.contains("amd64")
        })
        .or_else(|| {
            release
                .assets
                .iter()
                .find(|asset| asset.name.to_ascii_lowercase().ends_with(".appimage"))
        })
        .ok_or_else(|| "Keine AppImage-Datei im neuesten Release gefunden".to_string())?;

    let checksum_asset = release.assets.iter().find(|checksum| {
        let name = checksum.name.to_ascii_lowercase();
        name == format!("{}.sha256", asset.name.to_ascii_lowercase())
    });

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

    if let Some(checksum_asset) = checksum_asset {
        let checksum_response = reqwest::blocking::Client::new()
            .get(&checksum_asset.browser_download_url)
            .header("User-Agent", "twokey-ai")
            .send()
            .map_err(|error| format!("Checksum-Datei konnte nicht heruntergeladen werden: {error}"))?;
        if !checksum_response.status().is_success() {
            return Err(format!("Checksum-Download antwortete mit HTTP {}", checksum_response.status()));
        }

        let checksum_text = checksum_response
            .text()
            .map_err(|error| format!("Checksum-Datei konnte nicht gelesen werden: {error}"))?;
        let expected = parse_sha256_line(&checksum_text)
            .ok_or_else(|| "Checksum-Datei hat ein ungueltiges Format".to_string())?;
        let actual = hex_sha256(&bytes);
        if expected.to_ascii_lowercase() != actual {
            return Err("Checksum-Pruefung fehlgeschlagen. Update wurde abgebrochen.".to_string());
        }
    }

    let target = update_binary_path()?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Update-Verzeichnis konnte nicht erstellt werden: {error}"))?;
    }

    let temp_target = target.with_extension("AppImage.tmp");
    let backup_target = target.with_extension("AppImage.bak");

    fs::write(&temp_target, &bytes).map_err(|error| format!("Update-Datei konnte nicht gespeichert werden: {error}"))?;
    let mut perms = fs::metadata(&temp_target)
        .map_err(|error| format!("Dateiberechtigungen konnten nicht gelesen werden: {error}"))?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&temp_target, perms).map_err(|error| format!("Datei konnte nicht ausführbar gemacht werden: {error}"))?;

    if target.exists() {
        if backup_target.exists() {
            let _ = fs::remove_file(&backup_target);
        }
        fs::rename(&target, &backup_target)
            .map_err(|error| format!("Vorherige AppImage konnte nicht gesichert werden: {error}"))?;
    }

    if let Err(error) = fs::rename(&temp_target, &target) {
        if backup_target.exists() {
            let _ = fs::rename(&backup_target, &target);
        }
        return Err(format!("Update-Datei konnte nicht aktiviert werden: {error}"));
    }

    let started = Command::new(&target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| true)
        .unwrap_or(false);

    if !started {
        let _ = fs::remove_file(&target);
        if backup_target.exists() {
            let _ = fs::rename(&backup_target, &target);
        }
        return Err("Aktualisierte App konnte nicht gestartet werden. Vorherige Version wurde wiederhergestellt.".to_string());
    }

    if backup_target.exists() {
        let _ = fs::remove_file(&backup_target);
    }

    Ok(format!("Update {} (Kanal: {}) heruntergeladen und gestartet.", release.tag_name, channel))
}

fn fetch_release_for_channel(channel: &str) -> Result<Option<GithubRelease>, String> {
    if channel == "stable" {
        return fetch_latest_release().map(Some);
    }

    let response = reqwest::blocking::Client::new()
        .get(RELEASES_URL)
        .header("User-Agent", "twokey-ai")
        .send()
        .map_err(|error| format!("Update-Pruefung konnte GitHub nicht erreichen: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("GitHub Releases antwortete mit HTTP {}", response.status()));
    }

    let releases: Vec<GithubRelease> = response
        .json()
        .map_err(|error| format!("GitHub Releases konnten nicht gelesen werden: {error}"))?;

    let selected = match channel {
        "beta" => releases
            .into_iter()
            .find(|release| !release.draft && release.prerelease),
        "dev" => releases
            .into_iter()
            .find(|release| !release.draft),
        _ => releases
            .into_iter()
            .find(|release| !release.draft && !release.prerelease),
    };

    Ok(selected)
}

fn fetch_latest_release() -> Result<GithubRelease, String> {
    let response = reqwest::blocking::Client::new()
        .get(LATEST_RELEASE_URL)
        .header("User-Agent", "twokey-ai")
        .send()
        .map_err(|error| format!("Update-Pruefung konnte GitHub nicht erreichen: {error}"))?;

    if response.status().as_u16() == 404 {
        return Err("Noch keine GitHub Releases gefunden.".to_string());
    }

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

fn parse_sha256_line(text: &str) -> Option<String> {
    let line = text.lines().next()?.trim();
    let hash = line.split_whitespace().next()?.trim();
    if hash.len() == 64 && hash.chars().all(|char| char.is_ascii_hexdigit()) {
        return Some(hash.to_string());
    }
    None
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{:x}", digest)
}
