use std::sync::Arc;

use tauri::{App, AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::events::emit_hotkey_status;
use crate::session::SessionState;
use crate::AppState;

pub fn register_default_hotkey(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let settings = app.state::<AppState>().config.lock().unwrap().get();
    register_hotkey_internal(app.handle(), &settings.hotkey).map_err(Into::into)
}

pub fn unregister_hotkey(app: &AppHandle) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;
    if let Some(state) = app.try_state::<AppState>() {
        *state.hotkey_registered.lock().unwrap() = false;
    }
    Ok(())
}

pub fn register_hotkey_internal(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    let shortcut: Shortcut = hotkey.parse().map_err(|e| format!("Invalid hotkey: {e}"))?;

    let _ = app.global_shortcut().unregister_all();

    match app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            handle_hotkey_event(&app, event.state).await;
        });
    }) {
        Ok(()) => {
            if let Some(state) = app.try_state::<AppState>() {
                *state.hotkey_registered.lock().unwrap() = true;
            }
            emit_hotkey_status(app, true, hotkey, None);
            Ok(())
        }
        Err(e) => {
            if let Some(state) = app.try_state::<AppState>() {
                *state.hotkey_registered.lock().unwrap() = false;
            }
            let msg = format!("Failed to register hotkey: {e}");
            emit_hotkey_status(app, false, hotkey, Some(&msg));
            Err(msg)
        }
    }
}

async fn handle_hotkey_event(app: &AppHandle, state: ShortcutState) {
    let Some(app_state) = app.try_state::<AppState>() else {
        return;
    };

    let settings = app_state.config.lock().unwrap().get();
    let session: Arc<tokio::sync::Mutex<crate::session::SessionController>> =
        app_state.session.clone();

    match state {
        ShortcutState::Pressed => {
            let mut guard = session.lock().await;
            if *guard.state() == SessionState::Idle {
                if let Err(e) = guard.start(settings).await {
                    crate::events::emit_error(app, "session_start_failed", &e);
                }
            }
        }
        ShortcutState::Released => {
            let mut guard = session.lock().await;
            match guard.state() {
                SessionState::Listening => {
                    if let Err(e) = guard.finalize().await {
                        crate::events::emit_error(app, "session_finalize_failed", &e);
                        guard.cancel().await;
                    }
                }
                SessionState::Processing | SessionState::Injecting => {
                    // Ignore key release while a session is finishing.
                }
                SessionState::Idle => {}
            }
        }
    }
}

pub fn get_hotkey_registered(app: &AppHandle) -> bool {
    app.try_state::<AppState>()
        .map(|s| *s.hotkey_registered.lock().unwrap())
        .unwrap_or(false)
}
