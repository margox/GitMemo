import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  MessageSquare, StickyNote, BookOpen, FileText, HardDrive, GitBranch, Settings, Power, Clipboard,
} from "lucide-react";

interface AppStats {
  conversations: number;
  daily_notes: number;
  manuals: number;
  scratch_notes: number;
  total_size_kb: number;
  unpushed: number;
}

interface AppStatus {
  initialized: boolean;
  sync_dir: string;
  git_remote: string;
  git_branch: string;
  unpushed: number;
}

interface DesktopSettings {
  autostart: boolean;
  clipboard_autostart: boolean;
}

function Toggle({ enabled, onToggle }: { enabled: boolean; onToggle: () => void }) {
  return (
    <button
      onClick={onToggle}
      style={{
        width: 44,
        height: 24,
        borderRadius: 12,
        background: enabled ? "var(--accent)" : "#333",
        position: "relative",
        border: "none",
        cursor: "pointer",
        transition: "background 0.2s",
        flexShrink: 0,
      }}
    >
      <div
        style={{
          width: 18,
          height: 18,
          borderRadius: 9,
          background: "#fff",
          position: "absolute",
          top: 3,
          left: enabled ? 23 : 3,
          transition: "left 0.2s",
        }}
      />
    </button>
  );
}

export default function DashboardPage() {
  const [stats, setStats] = useState<AppStats | null>(null);
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [settings, setSettings] = useState<DesktopSettings | null>(null);
  const [error, setError] = useState("");

  useEffect(() => { loadData(); }, []);

  const loadData = async () => {
    try {
      const [s, st, se] = await Promise.all([
        invoke<AppStats>("get_stats"),
        invoke<AppStatus>("get_status"),
        invoke<DesktopSettings>("get_settings"),
      ]);
      setStats(s);
      setStatus(st);
      setSettings(se);
    } catch (e) {
      setError(`${e}`);
    }
  };

  const toggleAutostart = async () => {
    if (!settings) return;
    try {
      await invoke<string>("set_autostart", { enabled: !settings.autostart });
      setSettings({ ...settings, autostart: !settings.autostart });
    } catch (e) { console.error(e); }
  };

  const toggleClipboardAutostart = async () => {
    if (!settings) return;
    try {
      await invoke<string>("set_clipboard_autostart", { enabled: !settings.clipboard_autostart });
      setSettings({ ...settings, clipboard_autostart: !settings.clipboard_autostart });
    } catch (e) { console.error(e); }
  };

  if (error) {
    return (
      <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%" }}>
        <div style={{ textAlign: "center", padding: "0 32px" }}>
          <GitBranch size={48} style={{ color: "#555", margin: "0 auto 16px" }} />
          <p style={{ fontSize: 16, color: "var(--red)", marginBottom: 8 }}>{error}</p>
          <p style={{ fontSize: 13, color: "var(--text-secondary)" }}>
            Run <code style={{ background: "var(--bg-hover)", padding: "2px 8px", borderRadius: 4 }}>gitmemo init</code> to get started
          </p>
        </div>
      </div>
    );
  }

  if (!stats || !status) {
    return (
      <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%" }}>
        <p style={{ color: "var(--text-secondary)" }}>Loading...</p>
      </div>
    );
  }

  const cards = [
    { icon: MessageSquare, label: "Conversations", value: stats.conversations, color: "var(--accent)" },
    { icon: StickyNote, label: "Daily Notes", value: stats.daily_notes, color: "var(--green)" },
    { icon: BookOpen, label: "Manuals", value: stats.manuals, color: "var(--yellow)" },
    { icon: FileText, label: "Scratch Notes", value: stats.scratch_notes, color: "#c084fc" },
  ];

  const cardStyle = {
    background: "var(--bg-card)",
    border: "1px solid var(--border)",
    borderRadius: 10,
    padding: "20px 24px",
  };

  return (
    <div style={{ padding: "20px 32px 32px", overflowY: "auto", height: "100%" }}>
      <h1 style={{ fontSize: 22, fontWeight: 700, marginBottom: 20 }}>Dashboard</h1>

      {/* Stat Cards - 2x2 grid */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 24 }}>
        {cards.map((card) => {
          const Icon = card.icon;
          return (
            <div key={card.label} style={cardStyle}>
              <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 14 }}>
                <div style={{
                  width: 32, height: 32, borderRadius: 8,
                  background: `${card.color}15`,
                  display: "flex", alignItems: "center", justifyContent: "center",
                }}>
                  <Icon size={16} style={{ color: card.color }} />
                </div>
                <span style={{ fontSize: 12, color: "var(--text-secondary)", fontWeight: 500 }}>{card.label}</span>
              </div>
              <p style={{ fontSize: 32, fontWeight: 700, letterSpacing: -1 }}>{card.value}</p>
            </div>
          );
        })}
      </div>

      {/* Info Cards */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 32 }}>
        <div style={cardStyle}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 12 }}>
            <HardDrive size={14} style={{ color: "var(--text-secondary)" }} />
            <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>Storage</span>
          </div>
          <p style={{ fontSize: 18, fontWeight: 600 }}>
            {stats.total_size_kb >= 1024
              ? `${(stats.total_size_kb / 1024).toFixed(1)} MB`
              : `${stats.total_size_kb.toFixed(1)} KB`}
          </p>
          <p style={{ fontSize: 11, color: "var(--text-secondary)", marginTop: 6, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {status.sync_dir}
          </p>
        </div>

        <div style={cardStyle}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 12 }}>
            <GitBranch size={14} style={{ color: "var(--text-secondary)" }} />
            <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>Git Status</span>
          </div>
          <p style={{ fontSize: 18, fontWeight: 600 }}>
            {status.unpushed > 0 ? (
              <span style={{ color: "var(--yellow)" }}>{status.unpushed} unpushed</span>
            ) : (
              <span style={{ color: "var(--green)" }}>Synced</span>
            )}
          </p>
          <p style={{ fontSize: 11, color: "var(--text-secondary)", marginTop: 6, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={status.git_remote}>
            {status.git_remote || "No remote configured"}
          </p>
        </div>
      </div>

      {/* Settings */}
      {settings && (
        <>
          <div style={{ borderTop: "1px solid var(--border)", marginBottom: 24 }} />
          <div style={cardStyle}>
            <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 20 }}>
              <Settings size={16} style={{ color: "var(--text-secondary)" }} />
              <span style={{ fontSize: 15, fontWeight: 600 }}>Settings</span>
            </div>

            <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                  <Power size={15} style={{ color: "var(--text-secondary)" }} />
                  <div>
                    <p style={{ fontSize: 13, fontWeight: 500 }}>Launch at login</p>
                    <p style={{ fontSize: 11, color: "var(--text-secondary)", marginTop: 2 }}>Start GitMemo when you log in</p>
                  </div>
                </div>
                <Toggle enabled={settings.autostart} onToggle={toggleAutostart} />
              </div>

              <div style={{ borderTop: "1px solid var(--border)" }} />

              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                  <Clipboard size={15} style={{ color: "var(--text-secondary)" }} />
                  <div>
                    <p style={{ fontSize: 13, fontWeight: 500 }}>Auto-start clipboard monitor</p>
                    <p style={{ fontSize: 11, color: "var(--text-secondary)", marginTop: 2 }}>Begin capturing clipboard on launch</p>
                  </div>
                </div>
                <Toggle enabled={settings.clipboard_autostart} onToggle={toggleClipboardAutostart} />
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
