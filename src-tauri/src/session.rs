use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use tauri::window::Color;
use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition, Size};
use tokio::task::JoinHandle;

use crate::audio::capture::{AudioBuffer, AudioCaptureGuard};
use crate::config::{AppSettings, DictationMode};
use crate::events::{emit_audio_level, emit_clipboard_history_updated, emit_dictation_state, emit_error};
use crate::inject::{self, focus};
use crate::secrets;
use crate::stt::sarvam as stt;
use crate::translate::sarvam as translate;

/// Minimum hotkey hold before we treat input as intentional (ms).
const MIN_LISTEN_MS: u64 = 180;
/// Minimum PCM payload before calling STT (~180 ms at 16 kHz mono 16-bit).
const MIN_PCM_BYTES: usize = 5760;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    Idle,
    Listening,
    Processing,
    Injecting,
}

pub struct SessionController {
    app: AppHandle,
    state: SessionState,
    audio_guard: Option<AudioCaptureGuard>,
    audio_buffer: Option<AudioBuffer>,
    level_task: Option<JoinHandle<()>>,
    level_running: Arc<AtomicBool>,
    settings: Option<AppSettings>,
    listen_started_at: Option<Instant>,
}

impl SessionController {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            state: SessionState::Idle,
            audio_guard: None,
            audio_buffer: None,
            level_task: None,
            level_running: Arc::new(AtomicBool::new(false)),
            settings: None,
            listen_started_at: None,
        }
    }

    pub fn state(&self) -> &SessionState {
        &self.state
    }

    pub async fn start(&mut self, settings: AppSettings) -> Result<(), String> {
        if self.state != SessionState::Idle {
            return Ok(());
        }

        if secrets::get_api_key()?.is_none() {
            return Err("Sarvam API key not configured — add one in Settings".to_string());
        }

        let pressed_at = Instant::now();

        focus::capture_target_window();
        let (buffer, guard) = AudioCaptureGuard::start()?;

        let level_running = Arc::new(AtomicBool::new(true));
        let level_flag = level_running.clone();
        let buf_level = buffer.clone();
        let app_level = self.app.clone();

        let level_task = tokio::spawn(async move {
            while level_flag.load(Ordering::SeqCst) {
                tokio::time::sleep(std::time::Duration::from_millis(16)).await;
                emit_audio_level(&app_level, buf_level.peak_level());
            }
        });

        self.state = SessionState::Listening;
        self.audio_guard = Some(guard);
        self.audio_buffer = Some(buffer);
        self.level_running = level_running;
        self.level_task = Some(level_task);
        self.settings = Some(settings);
        self.listen_started_at = Some(pressed_at);

        emit_dictation_state(&self.app, "listening");
        Ok(())
    }

    pub async fn finalize(&mut self) -> Result<(), String> {
        if self.state != SessionState::Listening {
            return Ok(());
        }

        self.stop_level_task().await;
        self.audio_guard = None;

        let hold_ms = self
            .listen_started_at
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(0);

        let pcm = self
            .audio_buffer
            .as_ref()
            .map(|buf| buf.drain_pcm())
            .unwrap_or_default();

        if is_accidental_press(hold_ms, pcm.len()) {
            tracing::debug!(
                "Ignoring accidental hotkey tap ({hold_ms} ms, {} PCM bytes)",
                pcm.len()
            );
            self.reset_to_idle().await;
            return Ok(());
        }

        self.state = SessionState::Processing;
        emit_dictation_state(&self.app, "processing");

        let api_key = secrets::get_api_key()?.unwrap_or_default();

        let dictation_mode = self
            .app
            .try_state::<crate::AppState>()
            .map(|s| s.config.lock().unwrap().get().dictation_mode)
            .unwrap_or_else(|| self.settings.clone().unwrap_or_default().dictation_mode);

        let text = match dictation_mode {
            DictationMode::Native => {
                match stt::transcribe_pcm(&api_key, &pcm, "transcribe").await {
                    Ok(text) => text,
                    Err(e) => {
                        emit_error(&self.app, "stt_error", &e);
                        self.reset_to_idle().await;
                        return Ok(());
                    }
                }
            }
            DictationMode::English => {
                match translate::transcribe_to_english(&api_key, &pcm).await {
                    Ok(text) => text,
                    Err(e) => {
                        emit_error(&self.app, "translate_error", &e);
                        self.reset_to_idle().await;
                        return Ok(());
                    }
                }
            }
        };

        if text.trim().is_empty() {
            emit_error(&self.app, "empty_transcript", "No speech detected");
            self.reset_to_idle().await;
            return Ok(());
        }

        self.state = SessionState::Injecting;
        emit_dictation_state(&self.app, "injecting");

        let app = self.app.clone();
        let text_to_inject = text.clone();
        let injection = tauri::async_runtime::spawn_blocking(move || {
            inject::inject_text(&app, &text_to_inject)
        })
        .await
        .unwrap_or(inject::InjectionResult::Failed {
            reason: "Paste task failed".to_string(),
        });

        match &injection {
            inject::InjectionResult::Failed { reason } => {
                emit_error(&self.app, "paste_failed", reason);
            }
            inject::InjectionResult::ClipboardWithToast => {
                tracing::info!("Transcript on clipboard (auto-paste unavailable)");
            }
            inject::InjectionResult::Pasted => {}
        }

        if let Some(state) = self.app.try_state::<crate::AppState>() {
            let entry = state
                .history
                .lock()
                .unwrap()
                .add(text.clone(), dictation_mode);
            emit_clipboard_history_updated(&self.app, &entry);
        }

        self.reset_to_idle().await;
        Ok(())
    }

    pub async fn cancel(&mut self) {
        if self.state == SessionState::Idle {
            return;
        }
        self.reset_to_idle().await;
    }

    async fn stop_level_task(&mut self) {
        self.level_running.store(false, Ordering::SeqCst);
        if let Some(task) = self.level_task.take() {
            let _ = task.await;
        }
    }

    async fn reset_to_idle(&mut self) {
        self.stop_level_task().await;
        self.audio_guard = None;
        self.audio_buffer = None;
        self.settings = None;
        self.listen_started_at = None;
        self.state = SessionState::Idle;
        focus::clear_target_window();
        emit_dictation_state(&self.app, "idle");
    }
}

fn is_accidental_press(hold_ms: u64, pcm_bytes: usize) -> bool {
    hold_ms < MIN_LISTEN_MS || pcm_bytes < MIN_PCM_BYTES
}

pub fn init_overlay(app: &AppHandle) -> Result<(), String> {
    let overlay = app
        .get_webview_window("overlay")
        .ok_or("Overlay window not found")?;

    let _ = overlay.set_shadow(false);
    let _ = overlay.set_background_color(Some(Color(0, 0, 0, 0)));

    resize_overlay_for_state(app, "idle")?;
    overlay.show().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn resize_overlay_for_state(app: &AppHandle, state: &str) -> Result<(), String> {
    let overlay = app
        .get_webview_window("overlay")
        .ok_or("Overlay window not found")?;

    let (w, h) = overlay_size_for_state(state);
    overlay
        .set_size(Size::Logical(LogicalSize::new(w, h)))
        .map_err(|e| e.to_string())?;
    position_overlay(app, state)
}

fn overlay_size_for_state(state: &str) -> (f64, f64) {
    if state == "idle" {
        (12.0, 12.0)
    } else {
        (64.0, 64.0)
    }
}

fn overlay_margin_for_state(state: &str) -> f64 {
    if state == "idle" {
        80.0
    } else {
        56.0
    }
}

fn position_overlay(app: &AppHandle, state: &str) -> Result<(), String> {
    let overlay = app
        .get_webview_window("overlay")
        .ok_or("Overlay window not found")?;

    let scale = overlay.scale_factor().map_err(|e| e.to_string())?;
    let size = overlay.inner_size().map_err(|e| e.to_string())?;
    let overlay_width = size.width as i32;
    let overlay_height = size.height as i32;

    if let Ok(monitors) = app.available_monitors() {
        if let Some(monitor) = monitors.into_iter().next() {
            let monitor_size = monitor.size();
            let pos = monitor.position();
            let margin_bottom = (overlay_margin_for_state(state) * scale).round() as i32;
            let x = pos.x + (monitor_size.width as i32 - overlay_width) / 2;
            let y = pos.y + monitor_size.height as i32 - overlay_height - margin_bottom;
            let _ = overlay.set_position(PhysicalPosition::new(x, y));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{is_accidental_press, SessionState, MIN_LISTEN_MS, MIN_PCM_BYTES};

    #[test]
    fn state_transitions_are_distinct() {
        assert_ne!(SessionState::Idle, SessionState::Listening);
        assert_ne!(SessionState::Listening, SessionState::Processing);
        assert_ne!(SessionState::Processing, SessionState::Injecting);
    }

    #[test]
    fn accidental_press_is_short_hold_or_short_audio() {
        assert!(is_accidental_press(50, MIN_PCM_BYTES));
        assert!(is_accidental_press(MIN_LISTEN_MS, 100));
        assert!(!is_accidental_press(MIN_LISTEN_MS, MIN_PCM_BYTES));
    }
}
