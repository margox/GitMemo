import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpen, ChevronLeft, File, Folder, RefreshCw } from "lucide-react";
import { useI18n } from "../hooks/useI18n";
import { Loading } from "../components/Loading";
import MarkdownView from "../components/MarkdownView";
import { CopyPathButton } from "../components/CopyPathButton";
import { useResizablePanel } from "../hooks/useResizablePanel";

type EditorRoot = "claude" | "cursor";

interface EditorRootsStatus {
  claude_path: string;
  claude_exists: boolean;
  cursor_path: string;
  cursor_exists: boolean;
}

interface EditorDirEntry {
  name: string;
  rel_path: string;
  is_dir: boolean;
}

function parentRel(rel: string): string {
  const t = rel.replace(/\\/g, "/").replace(/\/+$/, "");
  if (!t) return "";
  const i = t.lastIndexOf("/");
  return i < 0 ? "" : t.slice(0, i);
}

function isProbablyMarkdown(name: string): boolean {
  const lower = name.toLowerCase();
  return lower.endsWith(".md") || lower.endsWith(".mdx") || lower.endsWith(".mdc");
}

export default function EditorHomePage() {
  const { t } = useI18n();
  const panel = useResizablePanel("editor-home", 320);
  const [roots, setRoots] = useState<EditorRootsStatus | null>(null);
  const [root, setRoot] = useState<EditorRoot>("claude");
  const [rel, setRel] = useState("");
  const [entries, setEntries] = useState<EditorDirEntry[]>([]);
  const [listLoading, setListLoading] = useState(true);
  const [listError, setListError] = useState("");
  const [selectedFileRel, setSelectedFileRel] = useState<string | null>(null);
  const [fileContent, setFileContent] = useState("");
  const [fileAbs, setFileAbs] = useState("");
  const [fileLoading, setFileLoading] = useState(false);
  const [fileError, setFileError] = useState("");

  const loadRoots = useCallback(async () => {
    try {
      const r = await invoke<EditorRootsStatus>("get_editor_data_roots");
      setRoots(r);
      if (!r.claude_exists && r.cursor_exists) setRoot("cursor");
    } catch {
      setRoots(null);
    }
  }, []);

  const loadDir = useCallback(async () => {
    setListLoading(true);
    setListError("");
    try {
      const list = await invoke<EditorDirEntry[]>("list_editor_directory", { root, rel });
      setEntries(list);
    } catch (e) {
      setEntries([]);
      setListError(String(e));
    } finally {
      setListLoading(false);
    }
  }, [root, rel]);

  useEffect(() => {
    void loadRoots();
  }, [loadRoots]);

  useEffect(() => {
    void loadDir();
  }, [loadDir]);

  const openFile = async (fileRel: string) => {
    setSelectedFileRel(fileRel);
    setFileContent("");
    setFileError("");
    setFileAbs("");
    setFileLoading(true);
    try {
      const [text, abs] = await Promise.all([
        invoke<string>("read_editor_home_file", { root, rel: fileRel }),
        invoke<string>("resolve_editor_file_abs", { root, rel: fileRel }),
      ]);
      setFileContent(text);
      setFileAbs(abs);
    } catch (e) {
      setFileError(String(e));
    } finally {
      setFileLoading(false);
    }
  };

  const rootOk = root === "claude" ? roots?.claude_exists : roots?.cursor_exists;

  const handleRefresh = useCallback(() => {
    void loadRoots();
    void loadDir();
    if (selectedFileRel) void openFile(selectedFileRel);
  }, [loadRoots, loadDir, selectedFileRel, root]);

  return (
    <div style={{ display: "flex", height: "100%", flexDirection: "column" }}>
      <div style={{
        padding: "14px 20px", borderBottom: "1px solid var(--border)",
        display: "flex", alignItems: "center", gap: 12,
      }}>
        <FolderOpen size={18} style={{ color: "var(--accent)", flexShrink: 0 }} />
        <div style={{ flex: 1, minWidth: 0 }}>
          <h1 style={{ margin: 0, fontSize: 16, fontWeight: 700 }}>{t("editorHome.title")}</h1>
          <p style={{ margin: "4px 0 0", fontSize: 11, color: "var(--text-secondary)" }}>
            {t("editorHome.subtitle")}
          </p>
        </div>
        <button
          type="button"
          onClick={handleRefresh}
          title={t("common.refresh")}
          style={{
            background: "none", border: "none", cursor: "pointer", padding: 6, borderRadius: 6,
            color: "var(--text-secondary)", display: "flex", alignItems: "center",
          }}
          onMouseEnter={(e) => (e.currentTarget.style.color = "var(--accent)")}
          onMouseLeave={(e) => (e.currentTarget.style.color = "var(--text-secondary)")}
        >
          <RefreshCw size={14} />
        </button>
      </div>

      {roots && !roots.claude_exists && !roots.cursor_exists ? (
        <div style={{ padding: 24, fontSize: 13, color: "var(--text-secondary)" }}>
          {t("editorHome.missingBoth")}
        </div>
      ) : (
      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        <div style={{
          width: panel.width, borderRight: "1px solid var(--border)",
          display: "flex", flexDirection: "column", flexShrink: 0,
        }}>
          <div style={{ display: "flex", gap: 4, padding: "8px 12px", borderBottom: "1px solid var(--border)" }}>
            {(["claude", "cursor"] as EditorRoot[]).map((r) => {
              const exists = r === "claude" ? roots?.claude_exists : roots?.cursor_exists;
              return (
                <button
                  key={r}
                  type="button"
                  disabled={!exists}
                  onClick={() => { setRoot(r); setRel(""); setSelectedFileRel(null); setFileContent(""); }}
                  style={{
                    flex: 1, padding: "6px 8px", borderRadius: 8, fontSize: 11, fontWeight: root === r ? 600 : 400,
                    border: "none", cursor: exists ? "pointer" : "not-allowed", opacity: exists ? 1 : 0.45,
                    background: root === r ? "var(--accent)" : "var(--bg-hover)",
                    color: root === r ? "#fff" : "var(--text-secondary)",
                  }}
                >
                  {r === "claude" ? "Claude" : "Cursor"}
                </button>
              );
            })}
          </div>

          <div style={{
            display: "flex", alignItems: "center", gap: 6, padding: "8px 12px",
            fontSize: 11, color: "var(--text-secondary)", borderBottom: "1px solid var(--border)",
          }}>
            {rel ? (
              <button
                type="button"
                onClick={() => { setRel(parentRel(rel)); setSelectedFileRel(null); setFileContent(""); }}
                style={{
                  display: "flex", alignItems: "center", gap: 4, padding: "2px 6px",
                  borderRadius: 4, border: "1px solid var(--border)", background: "var(--bg-hover)",
                  cursor: "pointer", color: "var(--text-secondary)", fontSize: 11,
                }}
              >
                <ChevronLeft size={14} />
                {t("editorHome.up")}
              </button>
            ) : null}
            <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1 }} title={rel || "."}>
              {rel || "~"}
            </span>
          </div>

          <div style={{ flex: 1, overflowY: "auto" }}>
            {!rootOk ? (
              <div style={{ padding: 24, textAlign: "center", fontSize: 12, color: "var(--text-secondary)" }}>
                {root === "claude" ? roots?.claude_path : roots?.cursor_path}
                <p style={{ marginTop: 8 }}>{t("editorHome.missingDir")}</p>
              </div>
            ) : listLoading ? (
              <Loading compact text={t("dashboard.loading")} />
            ) : listError ? (
              <p style={{ padding: 16, fontSize: 12, color: "var(--red)" }}>{listError}</p>
            ) : entries.length === 0 ? (
              <p style={{ padding: 16, fontSize: 12, color: "var(--text-secondary)" }}>{t("editorHome.emptyDir")}</p>
            ) : (
              entries.map((e) => {
                const sel = selectedFileRel === e.rel_path;
                return (
                  <button
                    key={e.rel_path}
                    type="button"
                    onClick={() => {
                      if (e.is_dir) {
                        setRel(e.rel_path);
                        setSelectedFileRel(null);
                        setFileContent("");
                      } else {
                        void openFile(e.rel_path);
                      }
                    }}
                    style={{
                      display: "flex", alignItems: "center", gap: 8, width: "100%",
                      padding: "10px 14px", textAlign: "left", border: "none", borderBottom: "1px solid var(--border)",
                      background: sel ? "var(--accent)" : "transparent",
                      color: sel ? "#fff" : "var(--text)", cursor: "pointer", fontSize: 13,
                    }}
                  >
                    {e.is_dir ? <Folder size={14} style={{ flexShrink: 0, opacity: 0.85 }} /> : <File size={14} style={{ flexShrink: 0, opacity: 0.85 }} />}
                    <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{e.name}</span>
                  </button>
                );
              })
            )}
          </div>
        </div>

        <div onMouseDown={panel.onMouseDown} style={panel.handleStyle}>
          <div style={panel.handleHoverStyle} />
        </div>

        <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden", minWidth: 0 }}>
          {!selectedFileRel ? (
            <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center" }}>
              <p style={{ fontSize: 13, color: "var(--text-secondary)" }}>{t("editorHome.selectFile")}</p>
            </div>
          ) : (
            <>
              <div style={{
                display: "flex", alignItems: "center", gap: 8, padding: "10px 16px",
                borderBottom: "1px solid var(--border)", flexShrink: 0,
              }}>
                <span style={{ flex: 1, fontSize: 11, color: "var(--text-secondary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={fileAbs}>
                  {selectedFileRel}
                </span>
                {fileAbs ? <CopyPathButton absolutePath={fileAbs} /> : null}
              </div>
              <div style={{ flex: 1, overflow: "auto", padding: "16px 20px" }}>
                {fileLoading ? <Loading compact text={t("dashboard.loading")} /> : null}
                {!fileLoading && fileError ? (
                  <p style={{ fontSize: 12, color: "var(--red)" }}>{fileError}</p>
                ) : null}
                {!fileLoading && !fileError && selectedFileRel ? (
                  isProbablyMarkdown(selectedFileRel) ? (
                    <MarkdownView content={fileContent} />
                  ) : (
                    <pre style={{
                      margin: 0, fontSize: 12, lineHeight: 1.5, whiteSpace: "pre-wrap", wordBreak: "break-word",
                      fontFamily: "ui-monospace, monospace", color: "var(--text)",
                    }}>
                      {fileContent}
                    </pre>
                  )
                ) : null}
              </div>
            </>
          )}
        </div>
      </div>
      )}
    </div>
  );
}
