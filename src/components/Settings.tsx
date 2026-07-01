import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
  AppSettings,
  ClipboardEntry,
  DictationMode,
  HotkeyStatus,
} from "../types";
import { hotkeyToKeycaps, keyEventToHotkey } from "../utils/hotkey";
import { formatHistoryTime } from "../utils/time";
import {
  CheckIcon,
  CloseIcon,
  CopyIcon,
  EyeIcon,
  EyeOffIcon,
  GlobeIcon,
  LockIcon,
  MicIconCurrent,
} from "./SettingsIcons";

const API_PLACEHOLDER = "sk-•••••••••••••••••";

export function Settings() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [history, setHistory] = useState<ClipboardEntry[]>([]);
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [apiKeySet, setApiKeySet] = useState(false);
  const [showApiKey, setShowApiKey] = useState(false);
  const [apiKeyDirty, setApiKeyDirty] = useState(false);
  const [hotkeyStatus, setHotkeyStatus] = useState<HotkeyStatus | null>(null);
  const [capturingHotkey, setCapturingHotkey] = useState(false);
  const [pendingHotkey, setPendingHotkey] = useState<string | null>(null);
  const [modalEntry, setModalEntry] = useState<ClipboardEntry | null>(null);
  const [copied, setCopied] = useState(false);
  const [copiedEntryId, setCopiedEntryId] = useState<string | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [apiKeySaved, setApiKeySaved] = useState(false);
  const [apiKeySaveError, setApiKeySaveError] = useState<string | null>(null);
  const copyResetTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const listCopyResetTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const captureTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const captureCommitted = useRef(false);
  const settingsRef = useRef(settings);
  settingsRef.current = settings;
  const [isMac] = useState(() =>
    navigator.platform.toLowerCase().includes("mac"),
  );

  const load = useCallback(async () => {
    setLoadError(null);
    try {
      const [s, keySet, hk, entries] = await Promise.all([
        invoke<AppSettings>("get_settings"),
        invoke<boolean>("get_api_key_set"),
        invoke<HotkeyStatus>("get_hotkey_status"),
        invoke<ClipboardEntry[]>("get_clipboard_history"),
      ]);
      setSettings({
        ...s,
        dictationMode: s.dictationMode === "english" ? "english" : "native",
      });
      setApiKeySet(keySet);
      setHotkeyStatus(hk);
      setHistory(entries);
      setApiKeyInput("");
      setApiKeyDirty(false);
      setShowApiKey(false);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setLoadError(message || "Failed to load settings");
    }
  }, []);

  useEffect(() => {
    load();
    const unsubs: Array<() => void> = [];
    listen<HotkeyStatus>("hotkey-status", (e) => {
      setHotkeyStatus((prev) => ({ ...prev, ...e.payload }));
    }).then((u) => unsubs.push(u));
    listen<ClipboardEntry>("clipboard-history-updated", (e) => {
      setHistory((prev) => [e.payload, ...prev.filter((h) => h.id !== e.payload.id)]);
    }).then((u) => unsubs.push(u));

    const window = getCurrentWindow();
    window
      .onFocusChanged(({ payload: focused }) => {
        if (focused) load();
      })
      .then((u) => unsubs.push(u));

    return () => unsubs.forEach((u) => u());
  }, [load]);

  const save = useCallback(async (next: AppSettings) => {
    await invoke("save_settings", { settings: next });
    setSettings(next);
  }, []);

  const setMode = useCallback(
    (dictationMode: DictationMode) => {
      const current = settingsRef.current;
      if (!current || current.dictationMode === dictationMode) return;
      const next = { ...current, dictationMode };
      setSettings(next);
      save(next);
    },
    [save],
  );

  const cancelHotkeyCapture = useCallback(async () => {
    if (captureTimer.current) clearTimeout(captureTimer.current);
    captureCommitted.current = false;
    setCapturingHotkey(false);
    setPendingHotkey(null);
    try {
      await invoke("resume_hotkey");
    } catch {
      /* already resumed */
    }
  }, []);

  const startHotkeyCapture = useCallback(async () => {
    if (capturingHotkey) {
      await cancelHotkeyCapture();
      return;
    }
    captureCommitted.current = false;
    await invoke("pause_hotkey");
    await getCurrentWindow().setFocus();
    setPendingHotkey(null);
    setCapturingHotkey(true);
    captureTimer.current = setTimeout(() => {
      cancelHotkeyCapture();
    }, 10000);
  }, [capturingHotkey, cancelHotkeyCapture]);

  const commitHotkey = useCallback(
    async (hotkey: string) => {
      if (captureCommitted.current) return;
      captureCommitted.current = true;
      const current = settingsRef.current;
      if (!current) return;
      if (captureTimer.current) clearTimeout(captureTimer.current);
      setCapturingHotkey(false);
      setPendingHotkey(hotkey);
      const next = { ...current, hotkey };
      setSettings(next);
      await save(next);
      try {
        await invoke("resume_hotkey");
      } catch {
        /* save_settings already re-registers when hotkey changes */
      }
    },
    [save],
  );

  const saveApiKey = useCallback(async (value: string) => {
    const trimmed = value.trim();
    if (!trimmed) return;
    setApiKeySaveError(null);
    try {
      await invoke("set_api_key", { key: trimmed });
      setApiKeySet(true);
      setApiKeyDirty(false);
      setShowApiKey(false);
      setApiKeyInput("");
      setApiKeySaved(true);
      window.setTimeout(() => setApiKeySaved(false), 2000);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setApiKeySaveError(message || "Failed to save API key");
    }
  }, []);

  const revealApiKey = useCallback(async () => {
    if (apiKeyDirty) return;
    const key = await invoke<string | null>("get_api_key");
    if (key) {
      setApiKeyInput(key);
    }
  }, [apiKeyDirty]);

  const handleApiKeyBlur = useCallback(() => {
    if (apiKeyDirty && apiKeyInput.trim()) {
      saveApiKey(apiKeyInput);
    }
  }, [apiKeyDirty, apiKeyInput, saveApiKey]);

  const toggleShowApiKey = useCallback(async () => {
    if (!showApiKey && !apiKeyDirty && apiKeySet) {
      await revealApiKey();
    }
    setShowApiKey((v) => !v);
  }, [showApiKey, apiKeyDirty, apiKeySet, revealApiKey]);

  const openModal = useCallback((entry: ClipboardEntry) => {
    setModalEntry(entry);
    setCopied(false);
    if (copyResetTimer.current) clearTimeout(copyResetTimer.current);
  }, []);

  const closeModal = useCallback(() => {
    setModalEntry(null);
    setCopied(false);
    if (copyResetTimer.current) clearTimeout(copyResetTimer.current);
  }, []);

  const copyModalText = useCallback(async () => {
    if (!modalEntry) return;
    try {
      await navigator.clipboard.writeText(modalEntry.text);
      setCopied(true);
      if (copyResetTimer.current) clearTimeout(copyResetTimer.current);
      copyResetTimer.current = setTimeout(() => setCopied(false), 1500);
    } catch {
      /* clipboard unavailable */
    }
  }, [modalEntry]);

  const copyHistoryEntry = useCallback(
    async (entry: ClipboardEntry, e: React.MouseEvent) => {
      e.stopPropagation();
      try {
        await navigator.clipboard.writeText(entry.text);
        setCopiedEntryId(entry.id);
        if (listCopyResetTimer.current) clearTimeout(listCopyResetTimer.current);
        listCopyResetTimer.current = setTimeout(() => setCopiedEntryId(null), 1500);
      } catch {
        /* clipboard unavailable */
      }
    },
    [],
  );

  useEffect(() => {
    if (!modalEntry) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") closeModal();
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [modalEntry, closeModal]);

  useEffect(() => {
    if (!capturingHotkey) return;

    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        cancelHotkeyCapture();
        return;
      }

      const hotkey = keyEventToHotkey(e, isMac);
      if (!hotkey) return;

      e.preventDefault();
      e.stopPropagation();
      setPendingHotkey(hotkey);
      commitHotkey(hotkey);
    };

    window.addEventListener("keydown", handleKey, true);
    return () => {
      window.removeEventListener("keydown", handleKey, true);
    };
  }, [capturingHotkey, isMac, cancelHotkeyCapture, commitHotkey]);

  if (!settings) {
    return (
      <div className="settings-loading">
        {loadError ? (
          <>
            <div className="settings-load-error">{loadError}</div>
            <button type="button" className="settings-retry-btn" onClick={load}>
              Retry
            </button>
          </>
        ) : (
          "Loading…"
        )}
      </div>
    );
  }

  const keycaps = capturingHotkey && pendingHotkey
    ? hotkeyToKeycaps(pendingHotkey, isMac)
    : hotkeyToKeycaps(settings.hotkey, isMac);
  const apiInputType = showApiKey ? "text" : "password";
  const apiPlaceholder = apiKeySet && !apiKeyDirty ? API_PLACEHOLDER : API_PLACEHOLDER;
  const apiValue = apiKeyDirty || showApiKey ? apiKeyInput : apiKeySet ? apiKeyInput : "";

  return (
    <div className="settings-page">
      <div className="settings-panel">
        <header className="settings-header">
          <div className="settings-brand">
            <img
              className="settings-brand-logo"
              src="/app-logo.png"
              alt=""
              width={36}
              height={36}
            />
            <div className="settings-brand-text">
              <h1>Polyflo</h1>
              <p>Push-to-talk dictation</p>
            </div>
          </div>
        </header>
        <div className="settings-content">
          <div className="settings-field settings-field--first">
            <div className="settings-field-label">Mode</div>
            <div className="settings-segmented" role="tablist" aria-label="Dictation mode">
              <button
                type="button"
                role="tab"
                aria-selected={settings.dictationMode === "native"}
                className={`settings-segment${settings.dictationMode === "native" ? " active" : ""}`}
                onClick={() => setMode("native")}
              >
                <MicIconCurrent />
                Transcribe
              </button>
              <button
                type="button"
                role="tab"
                aria-selected={settings.dictationMode === "english"}
                className={`settings-segment${settings.dictationMode === "english" ? " active" : ""}`}
                onClick={() => setMode("english")}
              >
                <GlobeIcon />
                Translate
              </button>
            </div>
          </div>

          <div className="settings-field settings-field--clipboard">
            <div className="settings-field-label">Clipboard</div>
            <div className="settings-history-box">
              {history.length === 0 ? (
                <div className="settings-history-empty">No transcripts yet</div>
              ) : (
                history.map((entry) => (
                  <div
                    key={entry.id}
                    className="settings-history-item"
                    role="button"
                    tabIndex={0}
                    onClick={() => openModal(entry)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        openModal(entry);
                      }
                    }}
                  >
                    <div className="settings-history-text-wrap">
                      <span className="settings-history-text">{entry.text}</span>
                      <div className="settings-history-time">
                        {formatHistoryTime(entry.timestamp)}
                      </div>
                    </div>
                    <button
                      type="button"
                      className={`settings-history-copy-btn${copiedEntryId === entry.id ? " copied" : ""}`}
                      aria-label={copiedEntryId === entry.id ? "Copied" : "Copy to clipboard"}
                      onClick={(e) => copyHistoryEntry(entry, e)}
                    >
                      {copiedEntryId === entry.id ? <CheckIcon /> : <CopyIcon />}
                    </button>
                  </div>
                ))
              )}
            </div>
          </div>

          <div className="settings-field">
            <div className="settings-field-label-row">
              <div>
                <div className="settings-field-label">Hotkey</div>
                <div className="settings-field-desc">Hold to dictate anywhere</div>
              </div>
            </div>
            <div className="settings-hotkey-row">
              <div className="settings-keycap-group">
                {capturingHotkey ? (
                  <span className="settings-keycap" style={{ fontFamily: "Inter, sans-serif", fontSize: "11.5px" }}>
                    Press keys…
                  </span>
                ) : (
                  keycaps.map((cap, i) => (
                    <span key={`${cap}-${i}`} style={{ display: "contents" }}>
                      {i > 0 && <span className="settings-keycap-plus">+</span>}
                      <span className="settings-keycap">{cap}</span>
                    </span>
                  ))
                )}
              </div>
              <button
                type="button"
                className={`settings-change-btn${capturingHotkey ? " settings-change-btn--capturing" : ""}`}
                onClick={startHotkeyCapture}
              >
                {capturingHotkey ? "Cancel" : "Change"}
              </button>
            </div>
            {hotkeyStatus && !hotkeyStatus.registered && hotkeyStatus.error && (
              <div className="settings-field-desc" style={{ color: "#c0392b" }}>
                {hotkeyStatus.error}
              </div>
            )}
          </div>

          <div className="settings-field">
            <div className="settings-field-label">API Key</div>
            <div className="settings-input-wrap">
              <input
                className="settings-api-input"
                type={apiInputType}
                placeholder={apiPlaceholder}
                value={apiValue}
                autoComplete="off"
                spellCheck={false}
                onChange={(e) => {
                  setApiKeyInput(e.target.value);
                  setApiKeyDirty(true);
                }}
                onBlur={handleApiKeyBlur}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.currentTarget.blur();
                  }
                }}
              />
              <button
                type="button"
                className="settings-eye-btn"
                aria-label={showApiKey ? "Hide key" : "Show key"}
                onClick={toggleShowApiKey}
              >
                {showApiKey ? <EyeOffIcon /> : <EyeIcon />}
              </button>
            </div>
            <div className="settings-helper-text">
              <LockIcon />
              {apiKeySaved
                ? "Saved locally — verified"
                : "Stored locally, never shared"}
            </div>
            {apiKeySaveError && (
              <div className="settings-field-desc settings-field-desc--error">
                {apiKeySaveError}
              </div>
            )}
          </div>
        </div>
      </div>

      <div
        className={`settings-overlay${modalEntry ? " active" : ""}`}
        onClick={(e) => {
          if (e.target === e.currentTarget) closeModal();
        }}
      >
        <div className="settings-modal">
          <div className="settings-modal-header">
            <div className="settings-modal-header-text">
              <div className="settings-field-label">Full transcript</div>
              <div className="settings-field-desc">
                {modalEntry ? formatHistoryTime(modalEntry.timestamp) : "—"}
              </div>
            </div>
            <div className="settings-modal-actions">
              <button
                type="button"
                className={`settings-copy-btn${copied ? " copied" : ""}`}
                aria-label={copied ? "Copied" : "Copy to clipboard"}
                onClick={copyModalText}
              >
                {copied ? <CheckIcon /> : <CopyIcon />}
              </button>
              <button
                type="button"
                className="settings-close-btn"
                aria-label="Close"
                onClick={closeModal}
              >
                <CloseIcon />
              </button>
            </div>
          </div>
          <div className="settings-modal-body">
            {modalEntry?.text ?? ""}
          </div>
        </div>
      </div>
    </div>
  );
}
