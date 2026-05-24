const SERVICE_NAME: &str = "dev.twokey.ai";

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretStatus {
    pub provider_id: String,
    pub configured: bool,
}

pub fn set_provider_api_key(provider_id: &str, api_key: &str) -> Result<(), String> {
    if api_key.trim().is_empty() {
        return Err("API-Key darf nicht leer sein".to_string());
    }

    let entry = keyring::Entry::new(SERVICE_NAME, provider_id)
        .map_err(|error| format!("Keyring-Eintrag konnte nicht erstellt werden: {error}"))?;
    entry
        .set_password(api_key)
        .map_err(|error| format!("API-Key konnte nicht im Keyring gespeichert werden: {error}"))
}

pub fn clear_provider_api_key(provider_id: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE_NAME, provider_id)
        .map_err(|error| format!("Keyring-Eintrag konnte nicht erstellt werden: {error}"))?;

    match entry.delete_credential() {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(format!("API-Key konnte nicht entfernt werden: {error}")),
    }
}

pub fn get_provider_api_key(provider_id: &str) -> Result<String, String> {
    let entry = keyring::Entry::new(SERVICE_NAME, provider_id)
        .map_err(|error| format!("Keyring-Eintrag konnte nicht erstellt werden: {error}"))?;

    entry
        .get_password()
        .map_err(|error| format!("Kein API-Key fuer {provider_id} im Keyring gefunden: {error}"))
}

pub fn provider_secret_status(provider_id: &str) -> SecretStatus {
    let configured = get_provider_api_key(provider_id).is_ok();
    SecretStatus {
        provider_id: provider_id.to_string(),
        configured,
    }
}
