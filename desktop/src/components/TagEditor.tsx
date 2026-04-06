import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Tag, Plus, X } from "lucide-react";

export function TagEditor({
  filePath,
  compact = false,
}: {
  filePath: string;
  compact?: boolean;
}) {
  const [tags, setTags] = useState<string[]>([]);
  const [allTags, setAllTags] = useState<string[]>([]);
  const [editing, setEditing] = useState(false);
  const [input, setInput] = useState("");
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);

  // Load current file tags from frontmatter
  useEffect(() => {
    if (!filePath) return;
    invoke<string>("read_file", { filePath })
      .then(content => {
        const match = content.match(/^---[\s\S]*?tags:\s*\[?([^\]\n]*)\]?[\s\S]*?---/);
        if (match?.[1]) {
          setTags(match[1].split(",").map(t => t.trim()).filter(Boolean));
        } else {
          setTags([]);
        }
      })
      .catch(() => setTags([]));
  }, [filePath]);

  // Load all available tags for autocomplete
  useEffect(() => {
    invoke<string[]>("get_all_tags").then(setAllTags).catch(() => {});
  }, []);

  const saveTags = useCallback(async (newTags: string[]) => {
    setTags(newTags);
    try {
      await invoke("update_file_tags", { filePath, tags: newTags });
    } catch (e) {
      console.error("Failed to save tags:", e);
    }
  }, [filePath]);

  const addTag = useCallback((tag: string) => {
    const t = tag.trim().toLowerCase();
    if (t && !tags.includes(t)) {
      void saveTags([...tags, t]);
    }
    setInput("");
    setSuggestions([]);
  }, [tags, saveTags]);

  const removeTag = useCallback((tag: string) => {
    void saveTags(tags.filter(t => t !== tag));
  }, [tags, saveTags]);

  const handleInputChange = useCallback((val: string) => {
    setInput(val);
    if (val.trim()) {
      const lower = val.toLowerCase();
      setSuggestions(
        allTags
          .filter(t => t.toLowerCase().includes(lower) && !tags.includes(t))
          .slice(0, 5)
      );
    } else {
      setSuggestions([]);
    }
  }, [allTags, tags]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Enter" && input.trim()) {
      e.preventDefault();
      addTag(input);
    } else if (e.key === "Escape") {
      setEditing(false);
      setInput("");
      setSuggestions([]);
    } else if (e.key === "Backspace" && !input && tags.length > 0) {
      void saveTags(tags.slice(0, -1));
    }
  }, [input, addTag, tags, saveTags]);

  const pillStyle = (color?: string): React.CSSProperties => ({
    display: "inline-flex",
    alignItems: "center",
    gap: 4,
    padding: compact ? "1px 6px" : "2px 8px",
    borderRadius: 12,
    fontSize: compact ? 10 : 11,
    background: color || "var(--accent)15",
    color: "var(--accent)",
    fontWeight: 500,
    whiteSpace: "nowrap",
  });

  return (
    <div style={{ display: "flex", flexWrap: "wrap", gap: 4, alignItems: "center" }}>
      <Tag size={compact ? 10 : 12} style={{ color: "var(--text-secondary)", flexShrink: 0 }} />

      {tags.map(tag => (
        <span key={tag} style={pillStyle()}>
          {tag}
          <button
            onClick={() => removeTag(tag)}
            style={{
              display: "inline-flex", padding: 0, border: "none",
              background: "none", cursor: "pointer", color: "var(--accent)",
              lineHeight: 1,
            }}
          >
            <X size={compact ? 8 : 10} />
          </button>
        </span>
      ))}

      {editing ? (
        <div style={{ position: "relative" }}>
          <input
            ref={inputRef}
            value={input}
            onChange={e => handleInputChange(e.target.value)}
            onKeyDown={handleKeyDown}
            onBlur={() => { setTimeout(() => { setEditing(false); setSuggestions([]); }, 150); }}
            autoFocus
            placeholder="add tag..."
            style={{
              width: 80,
              padding: "2px 6px",
              fontSize: compact ? 10 : 11,
              border: "1px solid var(--border)",
              borderRadius: 6,
              background: "var(--bg-input)",
              color: "var(--text)",
              outline: "none",
            }}
          />
          {suggestions.length > 0 && (
            <div style={{
              position: "absolute",
              top: "100%",
              left: 0,
              zIndex: 100,
              marginTop: 2,
              background: "var(--bg-card)",
              border: "1px solid var(--border)",
              borderRadius: 6,
              boxShadow: "0 4px 12px rgba(0,0,0,0.2)",
              minWidth: 100,
            }}>
              {suggestions.map(s => (
                <button
                  key={s}
                  onMouseDown={e => { e.preventDefault(); addTag(s); }}
                  style={{
                    display: "block",
                    width: "100%",
                    padding: "6px 10px",
                    fontSize: 11,
                    textAlign: "left",
                    border: "none",
                    background: "transparent",
                    color: "var(--text)",
                    cursor: "pointer",
                  }}
                >
                  {s}
                </button>
              ))}
            </div>
          )}
        </div>
      ) : (
        <button
          onClick={() => setEditing(true)}
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: 2,
            padding: "2px 6px",
            borderRadius: 12,
            border: "1px dashed var(--border)",
            background: "transparent",
            color: "var(--text-secondary)",
            fontSize: compact ? 10 : 11,
            cursor: "pointer",
          }}
        >
          <Plus size={compact ? 8 : 10} />
        </button>
      )}
    </div>
  );
}
