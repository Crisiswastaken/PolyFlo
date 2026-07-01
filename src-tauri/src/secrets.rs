use std::sync::Mutex;

use keyring::Entry;

const SERVICE: &str = "polyflo";
const LEGACY_SERVICE: &str = "voice-dictation";
const USER: &str = "sarvam";
const TARGET: &str = "com.polyflo.app/sarvam-api-key";

static KEYRING_LOCK: Mutex<()> = Mutex::new(());

fn open_entry(service: &str) -> Result<Entry, String> {
    Entry::new_with_target(TARGET, service, USER).map_err(|e| e.to_string())
}

fn read_keyring(service: &str) -> Result<Option<String>, String> {
    let _guard = KEYRING_LOCK
        .lock()
        .map_err(|e| format!("Keyring lock poisoned: {e}"))?;
    let entry = open_entry(service)?;
    match entry.get_password() {
        Ok(key) if !key.is_empty() => Ok(Some(key)),
        Ok(_) => Ok(None),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn get_api_key() -> Result<Option<String>, String> {
    if let Ok(key) = std::env::var("SARVAM_API_KEY") {
        if !key.is_empty() {
            return Ok(Some(key));
        }
    }

    if let Some(key) = read_keyring(SERVICE)? {
        return Ok(Some(key));
    }

    // Migrate keys saved under the pre-Polyflo service name.
    if let Some(key) = read_keyring(LEGACY_SERVICE)? {
        set_api_key(&key)?;
        if let Ok(entry) = open_entry(LEGACY_SERVICE) {
            let _ = entry.delete_credential();
        }
        return Ok(Some(key));
    }

    Ok(None)
}

pub fn set_api_key(key: &str) -> Result<(), String> {
    let _guard = KEYRING_LOCK
        .lock()
        .map_err(|e| format!("Keyring lock poisoned: {e}"))?;
    let entry = open_entry(SERVICE)?;
    entry.set_password(key).map_err(|e| e.to_string())?;

    // Verify persistence on the same credential (Windows can be timing-sensitive).
    let stored = entry.get_password().map_err(|e| e.to_string())?;
    if stored != key {
        return Err("API key verification failed after save".to_string());
    }
    Ok(())
}

pub fn clear_api_key() -> Result<(), String> {
    let _guard = KEYRING_LOCK
        .lock()
        .map_err(|e| format!("Keyring lock poisoned: {e}"))?;
    match open_entry(SERVICE) {
        Ok(entry) => match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e),
    }
}

pub fn has_api_key() -> bool {
    get_api_key()
        .ok()
        .flatten()
        .map(|k| !k.is_empty())
        .unwrap_or(false)
}
