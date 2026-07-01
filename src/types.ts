export type DictationMode = "native" | "english";

export interface AppSettings {
  hotkey: string;
  dictationMode: DictationMode;
}

export interface ClipboardEntry {
  id: string;
  text: string;
  timestamp: number;
  mode: DictationMode;
}

export interface PlatformInfo {
  os: string;
  injectionReliable: boolean;
  pasteModifier: string;
}

export interface HotkeyStatus {
  registered: boolean;
  hotkey: string;
  error?: string;
}

export interface ErrorEvent {
  code: string;
  message: string;
}
