pub mod clipboard;
pub mod focus;
pub mod paste;
pub mod platform;

use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::events::{emit_error, emit_injection_result};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InjectionResult {
    Pasted,
    ClipboardWithToast,
    Failed { reason: String },
}

pub fn inject_text(app: &AppHandle, text: &str) -> InjectionResult {
    if text.trim().is_empty() {
        return InjectionResult::Failed {
            reason: "Empty transcript".to_string(),
        };
    }

    focus::restore_target_window();
    thread::sleep(Duration::from_millis(20));

    if let Err(e) = clipboard::write_clipboard(text) {
        emit_error(app, "clipboard_failed", &e);
        focus::clear_target_window();
        return InjectionResult::Failed {
            reason: format!("Clipboard write failed: {e}"),
        };
    }

    thread::sleep(Duration::from_millis(15));

    if !platform::injection_reliable() {
        show_paste_toast(app);
        emit_injection_result(app, "clipboard", true);
        focus::clear_target_window();
        return InjectionResult::ClipboardWithToast;
    }

    let result = match paste::simulate_paste() {
        Ok(()) => {
            tracing::info!("Pasted transcript ({} chars)", text.len());
            emit_injection_result(app, "paste", true);
            InjectionResult::Pasted
        }
        Err(e) => {
            tracing::warn!("Paste simulation failed: {e}");
            show_paste_toast(app);
            emit_injection_result(app, "clipboard_fallback", true);
            InjectionResult::ClipboardWithToast
        }
    };

    focus::clear_target_window();
    result
}

fn show_paste_toast(app: &AppHandle) {
    let modifier = platform::paste_modifier_label();
    let body = format!("Transcript copied — press {modifier}+V to paste.");
    let _ = app
        .notification()
        .builder()
        .title("Polyflo")
        .body(&body)
        .show();
}
