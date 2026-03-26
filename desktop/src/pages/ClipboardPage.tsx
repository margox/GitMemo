import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { Clipboard, Play, Square, Save, Clock } from "lucide-react";

interface ClipboardStatus {
  watching: boolean;
  clips_count: number;
  clips_dir: string;
}

interface ClipboardEvent {
  saved: boolean;
  path: string;
  preview: string;
  timestamp: string;
}

interface FileEntry {
  name: string;
  path: string;
  source_type: string;
  modified: string;
  size: number;
  preview: string;
}

export default function ClipboardPage() {
  const [status, setStatus] = useState<ClipboardStatus | null>(null);
  const [recentClips, setRecentClips] = useState<ClipboardEvent[]>([]);
  const [savedClips, setSavedClips] = useState<FileEntry[]>([]);
  const [toast, setToast] = useState("");

  useEffect(() => {
    loadStatus();
    loadSavedClips();

    // Listen for clipboard-saved events
    const unlisten = listen<ClipboardEvent>("clipboard-saved", (event) => {
      setRecentClips((prev) => [event.payload, ...prev].slice(0, 20));
      loadSavedClips();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const loadStatus = async () => {
    try {
      const s = await invoke<ClipboardStatus>("get_clipboard_status");
      setStatus(s);
    } catch (e) {
      console.error(e);
    }
  };

  const loadSavedClips = async () => {
    try {
      const files = await invoke<FileEntry[]>("list_files", { folder: "clips" });
      setSavedClips(files);
    } catch (e) {
      console.error(e);
    }
  };

  const toggleWatch = async () => {
    try {
      if (status?.watching) {
        await invoke<string>("stop_clipboard_watch");
      } else {
        await invoke<string>("start_clipboard_watch");
      }
      loadStatus();
    } catch (e) {
      setToast(`Error: ${e}`);
      setTimeout(() => setToast(""), 3000);
    }
  };

  const saveNow = async () => {
    try {
      const text = await readText();
      if (!text || text.trim().length < 20) {
        setToast("Clipboard content too short (min 20 chars)");
        setTimeout(() => setToast(""), 2500);
        return;
      }
      const result = await invoke<ClipboardEvent>("save_clipboard_now", { content: text });
      setRecentClips((prev) => [result, ...prev].slice(0, 20));
      setToast("Saved!");
      loadSavedClips();
      loadStatus();
      setTimeout(() => setToast(""), 2000);
    } catch (e) {
      setToast(`Error: ${e}`);
      setTimeout(() => setToast(""), 3000);
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div
        className="flex items-center justify-between px-6 py-4 border-b"
        style={{ borderColor: "var(--border)" }}
      >
        <div className="flex items-center gap-3">
          <Clipboard size={20} style={{ color: "var(--accent)" }} />
          <h1 className="text-[18px] font-bold">Clipboard Monitor</h1>
          {status && (
            <span
              className="px-2 py-0.5 rounded-full text-[11px] font-medium"
              style={{
                background: status.watching ? "#0f2d0f" : "var(--bg-hover)",
                color: status.watching ? "var(--green)" : "var(--text-secondary)",
              }}
            >
              {status.watching ? "Watching" : "Stopped"}
            </span>
          )}
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={saveNow}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-[12px] transition-colors"
            style={{
              background: "var(--bg)",
              border: "1px solid var(--border)",
              color: "var(--text-secondary)",
            }}
          >
            <Save size={13} />
            Save Now
          </button>
          <button
            onClick={toggleWatch}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-[12px] font-medium transition-colors"
            style={{
              background: status?.watching ? "#2d1515" : "#0f2d0f",
              color: status?.watching ? "var(--red)" : "var(--green)",
              border: "1px solid",
              borderColor: status?.watching ? "#5a2020" : "#205a20",
            }}
          >
            {status?.watching ? (
              <>
                <Square size={13} />
                Stop
              </>
            ) : (
              <>
                <Play size={13} />
                Start
              </>
            )}
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-6">
        {/* Stats */}
        <div
          className="flex items-center gap-6 p-4 rounded-lg border mb-6"
          style={{ background: "var(--bg-card)", borderColor: "var(--border)" }}
        >
          <div>
            <p className="text-[11px]" style={{ color: "var(--text-secondary)" }}>
              Total Clips
            </p>
            <p className="text-[22px] font-bold">{status?.clips_count ?? 0}</p>
          </div>
          <div>
            <p className="text-[11px]" style={{ color: "var(--text-secondary)" }}>
              This Session
            </p>
            <p className="text-[22px] font-bold">{recentClips.length}</p>
          </div>
        </div>

        {/* Recent clips */}
        <h2 className="text-[14px] font-semibold mb-3">Recent Activity</h2>

        {recentClips.length === 0 && savedClips.length === 0 ? (
          <div
            className="flex flex-col items-center justify-center py-12 rounded-lg border"
            style={{ borderColor: "var(--border)", background: "var(--bg-card)" }}
          >
            <Clipboard size={40} style={{ color: "var(--border)" }} className="mb-3" />
            <p className="text-[13px]" style={{ color: "var(--text-secondary)" }}>
              No clips yet. Start watching or save manually.
            </p>
            <p className="text-[11px] mt-1" style={{ color: "var(--text-secondary)" }}>
              Copy text (20+ chars) to auto-capture
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {recentClips.map((clip, i) => (
              <div
                key={i}
                className="p-3 rounded-lg border"
                style={{ background: "var(--bg-card)", borderColor: "var(--border)" }}
              >
                <div className="flex items-center gap-2 mb-1">
                  <Clock size={11} style={{ color: "var(--text-secondary)" }} />
                  <span className="text-[11px]" style={{ color: "var(--text-secondary)" }}>
                    {clip.timestamp}
                  </span>
                  <span className="text-[10px] px-1.5 py-0.5 rounded" style={{ background: "var(--bg-hover)", color: "var(--green)" }}>
                    saved
                  </span>
                </div>
                <p className="text-[12px] truncate">{clip.preview}</p>
                <p className="text-[10px] mt-1" style={{ color: "var(--text-secondary)" }}>
                  {clip.path}
                </p>
              </div>
            ))}

            {savedClips.map((file) => (
              <div
                key={file.path}
                className="p-3 rounded-lg border"
                style={{ background: "var(--bg-card)", borderColor: "var(--border)" }}
              >
                <div className="flex items-center gap-2 mb-1">
                  <Clock size={11} style={{ color: "var(--text-secondary)" }} />
                  <span className="text-[11px]" style={{ color: "var(--text-secondary)" }}>
                    {file.modified}
                  </span>
                </div>
                <p className="text-[13px] font-medium">{file.name}</p>
                <p className="text-[11px] mt-0.5 truncate" style={{ color: "var(--text-secondary)" }}>
                  {file.preview}
                </p>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Toast */}
      {toast && (
        <div
          className="fixed bottom-4 right-4 px-4 py-2 rounded-lg text-[12px] shadow-lg"
          style={{
            background: toast.startsWith("Error") ? "#2d1515" : "var(--bg-card)",
            color: toast.startsWith("Error") ? "var(--red)" : "var(--green)",
            border: "1px solid var(--border)",
          }}
        >
          {toast}
        </div>
      )}
    </div>
  );
}
