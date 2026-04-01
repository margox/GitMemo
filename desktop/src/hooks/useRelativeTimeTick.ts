import { useState, useEffect } from "react";

/** Periodically bump state so `relativeTime()` labels refresh (e.g. "5 min ago"). */
export function useRelativeTimeTick(intervalMs = 30_000) {
  const [, setTick] = useState(0);
  useEffect(() => {
    const id = window.setInterval(() => setTick((n) => n + 1), intervalMs);
    return () => window.clearInterval(id);
  }, [intervalMs]);
}
