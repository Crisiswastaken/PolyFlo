const MODIFIER_KEYS = new Set([
  "Control",
  "Shift",
  "Alt",
  "Meta",
  "OS",
  "AltGraph",
]);

const MODIFIER_CODES = new Set([
  "ControlLeft",
  "ControlRight",
  "ShiftLeft",
  "ShiftRight",
  "AltLeft",
  "AltRight",
  "MetaLeft",
  "MetaRight",
  "OSLeft",
  "OSRight",
]);

export function isModifierKey(key: string, code?: string): boolean {
  if (MODIFIER_KEYS.has(key)) return true;
  if (code && MODIFIER_CODES.has(code)) return true;
  return false;
}

function codeToKeyName(code: string): string | null {
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  if (code === "Space") return "Space";
  if (code === "Comma") return ",";
  if (code === "Period") return ".";
  if (code === "Slash") return "/";
  if (code === "Backslash") return "\\";
  if (code === "BracketLeft") return "[";
  if (code === "BracketRight") return "]";
  if (code === "Semicolon") return ";";
  if (code === "Quote") return "'";
  if (code === "Backquote") return "`";
  if (code === "Minus") return "-";
  if (code === "Equal") return "=";
  if (code.startsWith("Arrow")) return code.slice(5);
  if (code.startsWith("F") && /^F\d+$/.test(code)) return code;
  if (code === "Tab") return "Tab";
  if (code === "Enter") return "Enter";
  if (code === "Backspace") return "Backspace";
  if (code === "Delete") return "Delete";
  if (code === "Home") return "Home";
  if (code === "End") return "End";
  if (code === "PageUp") return "PageUp";
  if (code === "PageDown") return "PageDown";
  if (code === "Insert") return "Insert";
  if (code === "Escape") return "Escape";
  return null;
}

export function keyEventToHotkey(e: KeyboardEvent, isMac: boolean): string | null {
  if (isModifierKey(e.key, e.code)) return null;

  const parts: string[] = [];

  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push(isMac ? "Cmd" : "Win");

  let key = codeToKeyName(e.code);
  if (!key) {
    if (e.key === " ") key = "Space";
    else if (e.key.length === 1) key = e.key.toUpperCase();
    else if (e.key.startsWith("Arrow")) key = e.key.slice(5);
    else key = e.key;
  }

  if (!key || isModifierKey(key)) return null;

  parts.push(key);
  return parts.join("+");
}

export function hotkeyToKeycaps(hotkey: string, isMac: boolean): string[] {
  return hotkey.split("+").map((part) => {
    const p = part.trim();
    if (isMac) {
      if (p === "Cmd" || p === "Command" || p === "Meta") return "⌘";
      if (p === "Ctrl" || p === "Control") return "⌃";
      if (p === "Alt" || p === "Option") return "⌥";
      if (p === "Shift") return "⇧";
      if (p === "Win" || p === "Super") return "⌘";
    } else {
      if (p === "Ctrl" || p === "Control") return "Ctrl";
      if (p === "Alt" || p === "Option") return "Alt";
      if (p === "Shift") return "Shift";
      if (p === "Cmd" || p === "Command" || p === "Meta" || p === "Win") return "Win";
    }
    if (p === "Space") return "Space";
    return p;
  });
}
