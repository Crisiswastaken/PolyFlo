use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::Manager;
use crate::config::DictationMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub text: String,
    pub timestamp: i64,
    pub mode: DictationMode,
}

pub struct HistoryStore {
    entries: Vec<HistoryEntry>,
    store_path: PathBuf,
    max_entries: usize,
}

impl HistoryStore {
    pub fn load(app: &tauri::AppHandle) -> Result<Self, String> {
        let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let store_path = dir.join("history.json");

        let entries = if store_path.exists() {
            let data = std::fs::read_to_string(&store_path).map_err(|e| e.to_string())?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(Self {
            entries,
            store_path,
            max_entries: 50,
        })
    }

    pub fn list(&self) -> Vec<HistoryEntry> {
        self.entries.clone()
    }

    pub fn add(&mut self, text: String, mode: DictationMode) -> HistoryEntry {
        let entry = HistoryEntry {
            id: new_entry_id(),
            text,
            timestamp: now_millis(),
            mode,
        };
        self.entries.insert(0, entry.clone());
        if self.entries.len() > self.max_entries {
            self.entries.truncate(self.max_entries);
        }
        let _ = self.save();
        entry
    }

    fn save(&self) -> Result<(), String> {
        let data = serde_json::to_string_pretty(&self.entries).map_err(|e| e.to_string())?;
        std::fs::write(&self.store_path, data).map_err(|e| e.to_string())
    }
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn new_entry_id() -> String {
    format!("{}", now_millis())
}
