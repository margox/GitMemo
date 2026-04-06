import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Star } from "lucide-react";

export function StarButton({
  filePath,
  size = 16,
}: {
  filePath: string;
  size?: number;
}) {
  const [starred, setStarred] = useState(false);

  useEffect(() => {
    if (!filePath) return;
    invoke<string>("read_file", { filePath })
      .then(content => {
        const match = content.match(/starred:\s*(true|yes)/i);
        setStarred(!!match);
      })
      .catch(() => setStarred(false));
  }, [filePath]);

  const toggle = useCallback(async () => {
    const newVal = !starred;
    setStarred(newVal);
    try {
      await invoke("toggle_star", { filePath, starred: newVal });
    } catch (e) {
      setStarred(!newVal); // revert on error
      console.error("Failed to toggle star:", e);
    }
  }, [filePath, starred]);

  return (
    <button
      onClick={e => { e.stopPropagation(); void toggle(); }}
      title={starred ? "Remove star" : "Star"}
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        padding: 4,
        borderRadius: 4,
        border: "none",
        background: "transparent",
        cursor: "pointer",
        color: starred ? "var(--yellow)" : "var(--text-secondary)",
        transition: "color 0.15s",
      }}
    >
      <Star
        size={size}
        fill={starred ? "var(--yellow)" : "none"}
        strokeWidth={starred ? 0 : 1.5}
      />
    </button>
  );
}
