const SERVICE: &str = "polyflo";
const USER: &str = "sarvam";

pub fn get_api_key() -> Result<Option<String>, String> {
    if let Ok(key) = std::env::var("SARVAM_API_KEY") {
        if !key.is_empty() {
            return Ok(Some(key));
        }
    }

    match keyring::Entry::new(SERVICE, USER) {
        Ok(entry) => match entry.get_password() {
            Ok(key) if !key.is_empty() => Ok(Some(key)),
            Ok(_) => Ok(None),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

pub fn set_api_key(key: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, USER).map_err(|e| e.to_string())?;
    entry.set_password(key).map_err(|e| e.to_string())
}

pub fn clear_api_key() -> Result<(), String> {
    match keyring::Entry::new(SERVICE, USER) {
        Ok(entry) => match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

pub fn has_api_key() -> bool {
    get_api_key()
        .ok()
        .flatten()
        .map(|k| !k.is_empty())
        .unwrap_or(false)
}
