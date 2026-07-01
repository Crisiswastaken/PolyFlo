use tauri::{AppHandle, State};

use crate::audio::capture;
use crate::config::{AppSettings, PlatformInfo};
use crate::hotkey;
use crate::inject::platform;
use crate::secrets;
use crate::history::HistoryEntry;
use crate::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.config.lock().unwrap().get()
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    let hotkey_changed = {
        let config = state.config.lock().unwrap();
        config.get().hotkey != settings.hotkey
    };

    {
        let mut config = state.config.lock().unwrap();
        config.update(settings.clone())?;
    }

    if hotkey_changed {
        hotkey::register_hotkey_internal(&app, &settings.hotkey)?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_api_key_set() -> bool {
    secrets::has_api_key()
}

#[tauri::command]
pub fn get_api_key() -> Result<Option<String>, String> {
    secrets::get_api_key()
}

#[tauri::command]
pub fn get_clipboard_history(state: State<'_, AppState>) -> Vec<HistoryEntry> {
    state.history.lock().unwrap().list()
}

#[tauri::command]
pub fn set_api_key(key: String) -> Result<(), String> {
    secrets::set_api_key(key.trim())
}

#[tauri::command]
pub fn clear_api_key() -> Result<(), String> {
    secrets::clear_api_key()
}

#[tauri::command]
pub fn test_mic() -> Result<bool, String> {
    capture::test_mic(500)
}

#[tauri::command]
pub fn pause_hotkey(app: AppHandle) -> Result<(), String> {
    hotkey::unregister_hotkey(&app)
}

#[tauri::command]
pub fn resume_hotkey(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let settings = state.config.lock().unwrap().get();
    hotkey::register_hotkey_internal(&app, &settings.hotkey)
}

#[tauri::command]
pub fn get_hotkey_status(app: AppHandle, state: State<'_, AppState>) -> serde_json::Value {
    let settings = state.config.lock().unwrap().get();
    serde_json::json!({
        "registered": hotkey::get_hotkey_registered(&app),
        "hotkey": settings.hotkey
    })
}

#[tauri::command]
pub fn get_platform_info() -> PlatformInfo {
    platform::get_platform_info()
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn check_accessibility_permission() -> bool {
    platform::injection_reliable()
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn request_accessibility_permission() -> Result<(), String> {
    use std::process::Command;
    Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
