import type { DictationMode } from "../types";

interface SplitFlapProps {
  mode: DictationMode;
  onChange: (mode: DictationMode) => void;
}

const MODES: { value: DictationMode; label: string; hint: string }[] = [
  { value: "native", label: "NATIVE", hint: "Paste in spoken language" },
  { value: "english", label: "ENGLISH", hint: "Translate to English, then paste" },
];

export function SplitFlap({ mode, onChange }: SplitFlapProps) {
  return (
    <div className="split-flap">
      <div className="split-flap__board">
        {MODES.map((m) => (
          <button
            key={m.value}
            type="button"
            className={`split-flap__cell${mode === m.value ? " split-flap__cell--active" : ""}`}
            onClick={() => onChange(m.value)}
            aria-pressed={mode === m.value}
          >
            <span className="split-flap__label">{m.label}</span>
          </button>
        ))}
      </div>
      <p className="split-flap__hint">
        {MODES.find((m) => m.value === mode)?.hint}
      </p>
    </div>
  );
}
