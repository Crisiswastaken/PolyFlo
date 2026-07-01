import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { AudioWave } from "./AudioWave";
import { OverlayLoader } from "./OverlayLoader";

type OverlayState = "idle" | "listening" | "processing" | "injecting";

function overlayClass(state: OverlayState): string {
  if (state === "listening") return "vox-sphere--listening";
  if (state === "processing" || state === "injecting") return "vox-sphere--processing";
  return "vox-sphere--idle";
}

export function Overlay() {
  const [state, setState] = useState<OverlayState>("idle");
  const [level, setLevel] = useState(0);

  useEffect(() => {
    const unsubs: Array<() => void> = [];

    listen<string>("dictation-state", (e) => {
      setState(e.payload as OverlayState);
      if (e.payload === "idle") {
        setLevel(0);
      }
    }).then((u) => unsubs.push(u));

    listen<number>("audio-level", (e) => {
      setLevel(e.payload);
    }).then((u) => unsubs.push(u));

    return () => unsubs.forEach((u) => u());
  }, []);

  const listening = state === "listening";
  const loading = state === "processing" || state === "injecting";

  return (
    <div className={`vox-sphere ${overlayClass(state)}`} role="status" aria-live="polite">
      <div className="vox-sphere__shell">
        <div className="vox-sphere__content">
          <div className={`vox-sphere__panel${listening ? " vox-sphere__panel--visible" : ""}`}>
            <AudioWave level={level} active={listening} />
          </div>
          <div className={`vox-sphere__panel${loading ? " vox-sphere__panel--visible" : ""}`}>
            <OverlayLoader />
          </div>
        </div>
      </div>
    </div>
  );
}
