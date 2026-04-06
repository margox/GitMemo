//! Capture conversations from Claude Code's native session logs.
//!
//! Claude Code writes two data sources automatically:
//! 1. `~/.claude/history.jsonl` — global index (sessionId, project, timestamp, display)
//! 2. `~/.claude/projects/{slug}/{sessionId}.jsonl` — full conversation (user + assistant messages)
//!
//! This module reads those files, converts to GitMemo markdown, and writes to the conversations directory.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, Seek, SeekFrom};
use std::path::{Path, PathBuf};

// ── State tracking ──────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CaptureState {
    /// Byte offset into history.jsonl (for incremental reads)
    pub history_byte_offset: u64,
    /// Per-session capture state
    #[serde(default)]
    pub captured_sessions: HashMap<String, SessionState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub last_line_count: usize,
    pub output_path: String,
    pub last_capture_ts: u64,
}

impl CaptureState {
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

// ── Data structures ─────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct SessionInfo {
    pub session_id: String,
    pub project: String,
    pub first_ts: u64,
    pub last_ts: u64,
    pub display_texts: Vec<String>,
}

#[derive(Debug)]
struct ConversationMessage {
    role: String,      // "user" or "assistant"
    text: String,
    timestamp: String, // HH:MM:SS
}

#[derive(Debug)]
struct ConversationContent {
    title: String,
    date_iso: String,
    session_id: String,
    project: String,
    messages: Vec<ConversationMessage>,
    total_lines: usize,
}

// ── history.jsonl parsing ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct HistoryEntry {
    display: String,
    timestamp: u64,
    project: String,
    #[serde(rename = "sessionId")]
    session_id: String,
}

/// Discover sessions with new activity since last capture.
fn discover_sessions(history_path: &Path, state: &mut CaptureState) -> Result<Vec<SessionInfo>> {
    if !history_path.exists() {
        return Ok(vec![]);
    }

    let file = std::fs::File::open(history_path)?;
    let file_len = file.metadata()?.len();

    // If file is shorter than our offset (e.g. after reinstall), reset
    if file_len < state.history_byte_offset {
        state.history_byte_offset = 0;
    }

    let mut reader = std::io::BufReader::new(file);
    reader.seek(SeekFrom::Start(state.history_byte_offset))?;

    let mut sessions: HashMap<String, SessionInfo> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<HistoryEntry>(&line) {
            let sid = entry.session_id.clone();
            let info = sessions.entry(sid.clone()).or_insert(SessionInfo {
                session_id: sid,
                project: entry.project.clone(),
                first_ts: entry.timestamp,
                last_ts: entry.timestamp,
                display_texts: Vec::new(),
            });
            if entry.timestamp < info.first_ts {
                info.first_ts = entry.timestamp;
            }
            if entry.timestamp > info.last_ts {
                info.last_ts = entry.timestamp;
            }
            if !entry.display.is_empty() && entry.display.len() < 200 {
                info.display_texts.push(entry.display);
            }
        }
    }

    state.history_byte_offset = file_len;

    // Only return sessions with new activity
    let result: Vec<SessionInfo> = sessions
        .into_values()
        .filter(|s| {
            match state.captured_sessions.get(&s.session_id) {
                Some(prev) => s.last_ts > prev.last_capture_ts,
                None => true,
            }
        })
        .collect();

    Ok(result)
}

// ── Per-session JSONL parsing ───────────────────────────────────────────────

fn project_slug(project: &str) -> String {
    project.replace('/', "-")
}

fn session_jsonl_path(session: &SessionInfo) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    let slug = project_slug(&session.project);
    home.join(".claude")
        .join("projects")
        .join(&slug)
        .join(format!("{}.jsonl", session.session_id))
}

#[derive(Deserialize)]
struct SessionEntry {
    #[serde(rename = "type")]
    entry_type: String,
    message: Option<serde_json::Value>,
    timestamp: Option<String>,
    #[serde(rename = "customTitle")]
    custom_title: Option<String>,
    #[serde(rename = "isMeta")]
    is_meta: Option<bool>,
    #[serde(rename = "isSnapshotUpdate")]
    is_snapshot_update: Option<bool>,
}

fn extract_conversation(session: &SessionInfo, state: &CaptureState) -> Result<ConversationContent> {
    let jsonl_path = session_jsonl_path(session);
    let skip_lines = state
        .captured_sessions
        .get(&session.session_id)
        .map(|s| s.last_line_count)
        .unwrap_or(0);

    let mut messages: Vec<ConversationMessage> = Vec::new();
    let mut title = String::new();
    let mut total_lines = 0usize;

    if jsonl_path.exists() {
        let file = std::fs::File::open(&jsonl_path)?;
        let reader = std::io::BufReader::new(file);

        for (i, line) in reader.lines().enumerate() {
            total_lines = i + 1;
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: SessionEntry = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Extract custom title if available
            if entry.entry_type == "custom-title" {
                if let Some(t) = entry.custom_title {
                    title = t;
                }
                continue;
            }

            // Skip snapshot updates and system entries
            if entry.is_snapshot_update.unwrap_or(false)
                || entry.entry_type == "file-history-snapshot"
                || entry.entry_type == "system"
                || entry.entry_type == "queue-operation"
                || entry.entry_type == "agent-name"
            {
                continue;
            }

            // Skip already-captured lines (for incremental capture)
            if i < skip_lines {
                continue;
            }

            // Skip meta commands (/clear, /help, etc.)
            if entry.is_meta.unwrap_or(false) {
                continue;
            }

            let ts = entry
                .timestamp
                .as_deref()
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                .map(|dt| {
                    let local: chrono::DateTime<chrono::Local> = dt.into();
                    local.format("%H:%M:%S").to_string()
                })
                .unwrap_or_default();

            if let Some(msg) = entry.message {
                let role = msg
                    .get("role")
                    .and_then(|r| r.as_str())
                    .unwrap_or("")
                    .to_string();
                let content = msg.get("content");

                let text = match content {
                    Some(serde_json::Value::String(s)) => {
                        // Filter command tags
                        if s.starts_with("<command-name>") || s.starts_with("<local-command-caveat>") {
                            continue;
                        }
                        s.clone()
                    }
                    Some(serde_json::Value::Array(arr)) => {
                        // Extract text blocks only (skip tool_use/tool_result)
                        let texts: Vec<String> = arr
                            .iter()
                            .filter_map(|block| {
                                if block.get("type")?.as_str()? == "text" {
                                    Some(block.get("text")?.as_str()?.to_string())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if texts.is_empty() {
                            continue;
                        }
                        texts.join("\n")
                    }
                    _ => continue,
                };

                if text.trim().is_empty() {
                    continue;
                }

                if (role == "user" || role == "assistant") && !text.trim().is_empty() {
                    messages.push(ConversationMessage {
                        role,
                        text,
                        timestamp: ts,
                    });
                }
            }
        }
    } else {
        // Fallback: use display texts from history.jsonl
        for display in &session.display_texts {
            messages.push(ConversationMessage {
                role: "user".to_string(),
                text: display.clone(),
                timestamp: String::new(),
            });
        }
    }

    // Generate title if not found
    if title.is_empty() {
        title = messages
            .iter()
            .find(|m| m.role == "user")
            .map(|m| {
                let t: String = m.text.chars().take(40).collect();
                t.lines().next().unwrap_or(&t).trim().to_string()
            })
            .unwrap_or_else(|| "Untitled session".to_string());
    }

    // Generate ISO date
    let date_iso = chrono::DateTime::from_timestamp_millis(session.first_ts as i64)
        .map(|dt| {
            let local: chrono::DateTime<chrono::Local> = dt.into();
            local.to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
        })
        .unwrap_or_else(|| chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false));

    Ok(ConversationContent {
        title,
        date_iso,
        session_id: session.session_id.clone(),
        project: session.project.clone(),
        messages,
        total_lines,
    })
}

// ── Markdown generation ─────────────────────────────────────────────────────

fn sanitize_title(title: &str) -> String {
    let clean: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' || c > '\u{4e00}' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let clean = clean.trim().replace(' ', "-");
    if clean.len() > 60 {
        clean.chars().take(60).collect()
    } else {
        clean
    }
}

fn to_markdown(content: &ConversationContent) -> String {
    let mut md = String::new();

    // Frontmatter
    md.push_str("---\n");
    md.push_str(&format!("title: {}\n", content.title));
    md.push_str(&format!("date: {}\n", content.date_iso));
    md.push_str(&format!("session_id: {}\n", content.session_id));
    md.push_str(&format!("project: {}\n", content.project));
    md.push_str("source: claude-code-capture\n");
    md.push_str(&format!(
        "messages: {}\n",
        content.messages.iter().filter(|m| m.role == "user").count()
    ));
    md.push_str("---\n\n");

    // Title
    md.push_str(&format!("# {}\n\n", content.title));

    // Messages (limit to ~300 lines)
    let mut line_count = 0;
    for msg in &content.messages {
        let role_label = if msg.role == "user" { "User" } else { "Assistant" };
        let header = if msg.timestamp.is_empty() {
            format!("### {}\n\n", role_label)
        } else {
            format!("### {} ({})\n\n", role_label, msg.timestamp)
        };
        md.push_str(&header);

        // Truncate very long messages
        let text = if msg.text.lines().count() > 80 {
            let truncated: String = msg.text.lines().take(80).collect::<Vec<_>>().join("\n");
            format!("{}\n\n*...truncated...*", truncated)
        } else {
            msg.text.clone()
        };

        md.push_str(&text);
        md.push_str("\n\n");

        line_count += text.lines().count() + 4;
        if line_count > 300 {
            md.push_str("*...conversation truncated for brevity...*\n");
            break;
        }
    }

    md
}

// ── Output path ─────────────────────────────────────────────────────────────

fn output_rel_path(content: &ConversationContent) -> String {
    let dt = chrono::DateTime::parse_from_rfc3339(&content.date_iso)
        .map(|d| {
            let local: chrono::DateTime<chrono::Local> = d.into();
            local
        })
        .unwrap_or_else(|_| chrono::Local::now());

    let month_dir = dt.format("%Y-%m").to_string();
    let date_prefix = dt.format("%m-%d").to_string();
    let safe_title = sanitize_title(&content.title);

    format!("conversations/{}/{}-{}.md", month_dir, date_prefix, safe_title)
}

// ── Check for existing captures ─────────────────────────────────────────────

fn find_existing_session_file(sync_dir: &Path, session_id: &str) -> Option<String> {
    let convos_dir = sync_dir.join("conversations");
    if !convos_dir.exists() {
        return None;
    }

    for entry in walkdir::WalkDir::new(&convos_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
    {
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            if content.contains(&format!("session_id: {}", session_id)) {
                let rel = entry
                    .path()
                    .strip_prefix(sync_dir)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .to_string();
                return Some(rel);
            }
        }
    }
    None
}

// ── Public API ──────────────────────────────────────────────────────────────

pub struct CaptureResult {
    pub new_sessions: usize,
    pub updated_sessions: usize,
    pub skipped: usize,
}

/// Run capture: discover new sessions, extract conversations, write markdown.
pub fn run_capture(sync_dir: &Path, project_filter: Option<&str>, dry_run: bool) -> Result<CaptureResult> {
    let state_path = sync_dir.join(".metadata").join("capture_state.json");
    let mut state = CaptureState::load(&state_path);

    let history_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".claude")
        .join("history.jsonl");

    let sessions = discover_sessions(&history_path, &mut state)?;

    let mut result = CaptureResult {
        new_sessions: 0,
        updated_sessions: 0,
        skipped: 0,
    };

    for session in &sessions {
        // Apply project filter if specified
        if let Some(filter) = project_filter {
            if !session.project.contains(filter) {
                result.skipped += 1;
                continue;
            }
        }

        // Check if already captured by AI (via /save or CLAUDE.md instruction)
        if let Some(existing) = find_existing_session_file(sync_dir, &session.session_id) {
            // Session already saved — check if we need to update
            if let Some(prev) = state.captured_sessions.get(&session.session_id) {
                if session.last_ts <= prev.last_capture_ts {
                    result.skipped += 1;
                    continue;
                }
            }
            // Update path to match existing file
            state.captured_sessions.entry(session.session_id.clone()).or_insert(SessionState {
                last_line_count: 0,
                output_path: existing,
                last_capture_ts: 0,
            });
        }

        let content = extract_conversation(session, &state)?;

        if content.messages.is_empty() {
            result.skipped += 1;
            continue;
        }

        let rel_path = state
            .captured_sessions
            .get(&session.session_id)
            .map(|s| s.output_path.clone())
            .unwrap_or_else(|| output_rel_path(&content));

        if dry_run {
            eprintln!(
                "  [dry-run] {} → {} ({} messages)",
                session.session_id,
                rel_path,
                content.messages.len()
            );
        } else {
            let md = to_markdown(&content);
            super::files::write_note(sync_dir, &rel_path, &md)?;
        }

        let is_new = !state.captured_sessions.contains_key(&session.session_id);
        state.captured_sessions.insert(
            session.session_id.clone(),
            SessionState {
                last_line_count: content.total_lines,
                output_path: rel_path,
                last_capture_ts: session.last_ts,
            },
        );

        if is_new {
            result.new_sessions += 1;
        } else {
            result.updated_sessions += 1;
        }
    }

    if !dry_run {
        state.save(&state_path)?;
    }

    Ok(result)
}
