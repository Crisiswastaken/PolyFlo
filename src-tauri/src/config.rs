use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DictationMode {
    Native,
    English,
}

impl Default for DictationMode {
    fn default() -> Self {
        Self::Native
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub hotkey: String,
    pub dictation_mode: DictationMode,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: default_hotkey(),
            dictation_mode: DictationMode::default(),
        }
    }
}

pub fn default_hotkey() -> String {
    if cfg!(target_os = "macos") {
        "Cmd+Shift+Space".to_string()
    } else {
        "Ctrl+Shift+Space".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    pub os: String,
    pub injection_reliable: bool,
    pub paste_modifier: String,
}

pub struct ConfigStore {
    settings: AppSettings,
    store_path: std::path::PathBuf,
}

impl ConfigStore {
    pub fn load(app: &tauri::AppHandle) -> Result<Self, String> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let store_path = dir.join("settings.json");

        let settings = if store_path.exists() {
            let data = std::fs::read_to_string(&store_path).map_err(|e| e.to_string())?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            AppSettings::default()
        };

        Ok(Self {
            settings,
            store_path,
        })
    }

    pub fn get(&self) -> AppSettings {
        self.settings.clone()
    }

    pub fn update(&mut self, settings: AppSettings) -> Result<(), String> {
        self.settings = settings;
        self.save()
    }

    pub fn save(&self) -> Result<(), String> {
        let data = serde_json::to_string_pretty(&self.settings).map_err(|e| e.to_string())?;
        std::fs::write(&self.store_path, data).map_err(|e| e.to_string())
    }
}
