import {
  LayoutDashboard,
  StickyNote,
  Clipboard,
  Search,
  GitBranch,
  RefreshCw,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import type { Page } from "../App";

interface SidebarProps {
  currentPage: Page;
  onNavigate: (page: Page) => void;
}

const navItems: { id: Page; icon: typeof LayoutDashboard; label: string }[] = [
  { id: "dashboard", icon: LayoutDashboard, label: "Dashboard" },
  { id: "notes", icon: StickyNote, label: "Notes" },
  { id: "clipboard", icon: Clipboard, label: "Clipboard" },
  { id: "search", icon: Search, label: "Search" },
];

export default function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  const [syncing, setSyncing] = useState(false);
  const [syncMsg, setSyncMsg] = useState("");

  const handleSync = async () => {
    setSyncing(true);
    setSyncMsg("");
    try {
      const result = await invoke<string>("sync_to_git");
      setSyncMsg(result);
    } catch (e) {
      setSyncMsg(`Error: ${e}`);
    } finally {
      setSyncing(false);
      setTimeout(() => setSyncMsg(""), 3000);
    }
  };

  return (
    <div
      className="flex flex-col w-[200px] border-r h-full"
      style={{
        background: "var(--bg-card)",
        borderColor: "var(--border)",
      }}
    >
      {/* Logo */}
      <div className="flex items-center gap-2 px-4 py-4 border-b" style={{ borderColor: "var(--border)" }}>
        <GitBranch size={20} style={{ color: "var(--accent)" }} />
        <span className="font-bold text-[15px]">GitMemo</span>
      </div>

      {/* Nav */}
      <nav className="flex-1 py-2">
        {navItems.map((item) => {
          const Icon = item.icon;
          const active = currentPage === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onNavigate(item.id)}
              className="flex items-center gap-3 w-full px-4 py-2.5 text-[13px] transition-colors"
              style={{
                background: active ? "var(--bg-hover)" : "transparent",
                color: active ? "var(--accent)" : "var(--text-secondary)",
                fontWeight: active ? 600 : 400,
              }}
            >
              <Icon size={16} />
              {item.label}
            </button>
          );
        })}
      </nav>

      {/* Sync button */}
      <div className="p-3 border-t" style={{ borderColor: "var(--border)" }}>
        <button
          onClick={handleSync}
          disabled={syncing}
          className="flex items-center justify-center gap-2 w-full py-2 rounded-md text-[12px] transition-colors"
          style={{
            background: syncing ? "var(--bg-hover)" : "var(--bg)",
            color: "var(--text-secondary)",
            border: "1px solid var(--border)",
          }}
        >
          <RefreshCw size={13} className={syncing ? "animate-spin" : ""} />
          {syncing ? "Syncing..." : "Sync to Git"}
        </button>
        {syncMsg && (
          <p className="text-[11px] mt-1.5 text-center" style={{ color: "var(--text-secondary)" }}>
            {syncMsg}
          </p>
        )}
      </div>
    </div>
  );
}
