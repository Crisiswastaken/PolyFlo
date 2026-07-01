pub mod audio;
pub mod commands;
pub mod config;
pub mod events;
pub mod history;
pub mod hotkey;
pub mod inject;
pub mod secrets;
pub mod session;
pub mod stt;
pub mod translate;
pub mod tray;

use std::sync::{Arc, Mutex};

use tauri::Manager;
use tracing_subscriber::EnvFilter;

use crate::config::ConfigStore;
use crate::history::HistoryStore;
use crate::session::SessionController;

pub struct AppState {
    pub config: Mutex<ConfigStore>,
    pub history: Mutex<HistoryStore>,
    pub session: Arc<tokio::sync::Mutex<SessionController>>,
    pub hotkey_registered: Mutex<bool>,
}

fn load_env() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest.join("../.env"),
        manifest.join(".env"),
    ];
    for path in candidates {
        if path.exists() {
            let _ = dotenvy::from_path(path);
        }
    }
    let _ = dotenvy::dotenv();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    load_env();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                app.handle()
                    .plugin(tauri_plugin_macos_permissions::init())?;
            }

            let config = ConfigStore::load(app.handle())?;
            let history = HistoryStore::load(app.handle())?;
            let session = Arc::new(tokio::sync::Mutex::new(SessionController::new(
                app.handle().clone(),
            )));

            app.manage(AppState {
                config: Mutex::new(config),
                history: Mutex::new(history),
                session,
                hotkey_registered: Mutex::new(false),
            });

            crate::tray::setup_tray(app)?;
            crate::hotkey::register_default_hotkey(app)?;

            if let Err(e) = crate::session::init_overlay(app.handle()) {
                tracing::warn!("Failed to show idle overlay: {e}");
            }

            if !crate::secrets::has_api_key() {
                if let Some(window) = app.get_webview_window("settings") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::get_api_key_set,
            commands::get_api_key,
            commands::get_clipboard_history,
            commands::set_api_key,
            commands::clear_api_key,
            commands::test_mic,
            commands::get_hotkey_status,
            commands::pause_hotkey,
            commands::resume_hotkey,
            commands::get_platform_info,
            #[cfg(target_os = "macos")]
            commands::check_accessibility_permission,
            #[cfg(target_os = "macos")]
            commands::request_accessibility_permission,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
