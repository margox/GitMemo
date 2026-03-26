import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MessageSquare, StickyNote, BookOpen, FileText, HardDrive, GitBranch } from "lucide-react";

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

export default function DashboardPage() {
  const [stats, setStats] = useState<AppStats | null>(null);
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [s, st] = await Promise.all([
        invoke<AppStats>("get_stats"),
        invoke<AppStatus>("get_status"),
      ]);
      setStats(s);
      setStatus(st);
    } catch (e) {
      setError(`${e}`);
    }
  };

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-[15px] mb-2" style={{ color: "var(--red)" }}>
            {error}
          </p>
          <p className="text-[13px]" style={{ color: "var(--text-secondary)" }}>
            请先运行 <code className="px-1.5 py-0.5 rounded" style={{ background: "var(--bg-hover)" }}>gitmemo init</code> 初始化
          </p>
        </div>
      </div>
    );
  }

  if (!stats || !status) {
    return (
      <div className="flex items-center justify-center h-full">
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

  return (
    <div className="p-6 overflow-y-auto h-full">
      <h1 className="text-[20px] font-bold mb-6">Dashboard</h1>

      {/* Stat Cards */}
      <div className="grid grid-cols-2 gap-4 mb-6">
        {cards.map((card) => {
          const Icon = card.icon;
          return (
            <div
              key={card.label}
              className="p-4 rounded-lg border"
              style={{ background: "var(--bg-card)", borderColor: "var(--border)" }}
            >
              <div className="flex items-center gap-3 mb-3">
                <Icon size={18} style={{ color: card.color }} />
                <span className="text-[12px]" style={{ color: "var(--text-secondary)" }}>
                  {card.label}
                </span>
              </div>
              <p className="text-[28px] font-bold">{card.value}</p>
            </div>
          );
        })}
      </div>

      {/* Info Cards */}
      <div className="grid grid-cols-2 gap-4">
        <div
          className="p-4 rounded-lg border"
          style={{ background: "var(--bg-card)", borderColor: "var(--border)" }}
        >
          <div className="flex items-center gap-2 mb-3">
            <HardDrive size={16} style={{ color: "var(--text-secondary)" }} />
            <span className="text-[12px]" style={{ color: "var(--text-secondary)" }}>
              Storage
            </span>
          </div>
          <p className="text-[16px] font-semibold">{stats.total_size_kb.toFixed(1)} KB</p>
          <p className="text-[11px] mt-1" style={{ color: "var(--text-secondary)" }}>
            {status.sync_dir}
          </p>
        </div>

        <div
          className="p-4 rounded-lg border"
          style={{ background: "var(--bg-card)", borderColor: "var(--border)" }}
        >
          <div className="flex items-center gap-2 mb-3">
            <GitBranch size={16} style={{ color: "var(--text-secondary)" }} />
            <span className="text-[12px]" style={{ color: "var(--text-secondary)" }}>
              Git Status
            </span>
          </div>
          <p className="text-[14px] font-semibold">
            {status.unpushed > 0 ? (
              <span style={{ color: "var(--yellow)" }}>{status.unpushed} unpushed</span>
            ) : (
              <span style={{ color: "var(--green)" }}>Synced</span>
            )}
          </p>
          <p
            className="text-[11px] mt-1 truncate"
            style={{ color: "var(--text-secondary)" }}
            title={status.git_remote}
          >
            {status.git_remote || "No remote configured"}
          </p>
        </div>
      </div>
    </div>
  );
}
