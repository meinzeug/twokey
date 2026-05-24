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
            "Neue Version verfuegbar. Download bleibt manuell.".to_string()
        } else {
            "Du nutzt die aktuelle Release-Version.".to_string()
        },
    })
}
