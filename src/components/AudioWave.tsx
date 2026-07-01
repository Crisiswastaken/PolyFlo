import { useEffect, useRef, useState } from "react";

interface AudioWaveProps {
  level: number;
  active: boolean;
}

const BAR_COUNT = 4;
const BAR_MIN = 3;
const BAR_MAX = 24;
const BAR_SPREADS = [0.82, 1.0, 0.72, 0.92];

function boostLevel(level: number): number {
  if (level <= 0) return 0;
  return Math.min(1, Math.pow(level, 0.42) * 1.45);
}

export function AudioWave({ level, active }: AudioWaveProps) {
  const [bars, setBars] = useState<number[]>(() => Array(BAR_COUNT).fill(BAR_MIN));
  const levelRef = useRef(0);
  const frameRef = useRef<number | null>(null);

  useEffect(() => {
    levelRef.current = level;
  }, [level]);

  useEffect(() => {
    if (!active) {
      if (frameRef.current !== null) {
        cancelAnimationFrame(frameRef.current);
        frameRef.current = null;
      }
      setBars(Array(BAR_COUNT).fill(BAR_MIN));
      return;
    }

    const tick = () => {
      const boosted = boostLevel(levelRef.current);
      const speaking = boosted > 0.04;

      setBars((prev) =>
        prev.map((current, i) => {
          const spread = BAR_SPREADS[i] ?? 1;
          const jitter = speaking
            ? (Math.random() - 0.5) * 5 * boosted
            : (Math.random() - 0.5) * 0.4;
          const target = speaking
            ? Math.min(BAR_MAX, Math.max(BAR_MIN + 1, BAR_MIN + (BAR_MAX - BAR_MIN) * boosted * spread + jitter))
            : BAR_MIN + 0.5 + Math.abs(jitter) * 0.3;
          const alpha = target > current ? 0.72 : 0.18;
          return current + (target - current) * alpha;
        }),
      );

      frameRef.current = requestAnimationFrame(tick);
    };

    frameRef.current = requestAnimationFrame(tick);

    return () => {
      if (frameRef.current !== null) {
        cancelAnimationFrame(frameRef.current);
        frameRef.current = null;
      }
    };
  }, [active]);

  return (
    <div className="audio-wave" aria-hidden="true">
      {bars.map((h, i) => (
        <span
          key={i}
          className="audio-wave__bar"
          style={{ height: `${h}px` }}
        />
      ))}
    </div>
  );
}
