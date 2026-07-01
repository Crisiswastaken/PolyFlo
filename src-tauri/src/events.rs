use tauri::{AppHandle, Emitter};

pub fn emit_dictation_state(app: &AppHandle, state: &str) {
    let _ = app.emit("dictation-state", state);
    let _ = crate::session::resize_overlay_for_state(app, state);
}

pub fn emit_audio_level(app: &AppHandle, level: f32) {
    let _ = app.emit("audio-level", level);
}

pub fn emit_injection_result(app: &AppHandle, method: &str, success: bool) {
    let _ = app.emit(
        "injection-result",
        serde_json::json!({ "method": method, "success": success }),
    );
}

pub fn emit_error(app: &AppHandle, code: &str, message: &str) {
    let _ = app.emit(
        "error",
        serde_json::json!({ "code": code, "message": message }),
    );
}

pub fn emit_clipboard_history_updated(app: &AppHandle, entry: &crate::history::HistoryEntry) {
    let _ = app.emit("clipboard-history-updated", entry);
}

pub fn emit_hotkey_status(app: &AppHandle, registered: bool, hotkey: &str, error: Option<&str>) {
    let _ = app.emit(
        "hotkey-status",
        serde_json::json!({
            "registered": registered,
            "hotkey": hotkey,
            "error": error
        }),
    );
}
