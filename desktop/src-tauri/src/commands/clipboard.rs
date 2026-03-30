use gitmemo_core::storage::{files, git};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter};

static WATCHING: AtomicBool = AtomicBool::new(false);

/// Minimum characters to save (filter out passwords, short copies)
const MIN_LENGTH: usize = 10;

/// Interval between clipboard checks (ms)
const POLL_INTERVAL_MS: u64 = 300;

#[derive(Debug, Clone, Serialize)]
pub struct ClipboardEvent {
    pub saved: bool,
    pub path: String,
    pub preview: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct ClipboardStatus {
    pub watching: bool,
    pub clips_count: usize,
    pub clips_dir: String,
}

#[tauri::command]
pub fn get_clipboard_status() -> Result<ClipboardStatus, String> {
    let sync_dir = files::sync_dir();
    let clips_dir = sync_dir.join("clips");

    let count = if clips_dir.exists() {
        walkdir::WalkDir::new(&clips_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .count()
    } else {
        0
    };

    Ok(ClipboardStatus {
        watching: WATCHING.load(Ordering::SeqCst),
        clips_count: count,
        clips_dir: clips_dir.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn start_clipboard_watch(app: AppHandle) -> Result<String, String> {
    if WATCHING.load(Ordering::SeqCst) {
        return Ok("Clipboard watch already running".into());
    }

    WATCHING.store(true, Ordering::SeqCst);

    std::thread::spawn(move || {
        clipboard_poll_loop(app);
    });

    Ok("Clipboard watch started".into())
}

#[tauri::command]
pub fn stop_clipboard_watch() -> Result<String, String> {
    WATCHING.store(false, Ordering::SeqCst);
    Ok("Clipboard watch stopped".into())
}

#[tauri::command]
pub fn save_clipboard_now(content: String) -> Result<ClipboardEvent, String> {
    save_clip_content(&content)
}

fn clipboard_poll_loop(app: AppHandle) {
    // Use arboard for native clipboard access (no subprocess overhead)
    let mut clipboard = match arboard::Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            log::error!("Failed to init clipboard: {}", e);
            WATCHING.store(false, Ordering::SeqCst);
            return;
        }
    };

    // Initialize with current clipboard content hash
    let mut last_hash = clipboard
        .get_text()
        .ok()
        .filter(|t| !t.is_empty())
        .map(|t| content_hash(&t))
        .unwrap_or_default();

    while WATCHING.load(Ordering::SeqCst) {
        if let Ok(text) = clipboard.get_text() {
            if !text.is_empty() && text.len() >= MIN_LENGTH {
                let hash = content_hash(&text);
                if hash != last_hash {
                    last_hash = hash;
                    if let Ok(event) = save_clip_content(&text) {
                        let _ = app.emit("clipboard-saved", &event);
                    }
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS));
    }
}

fn save_clip_content(content: &str) -> Result<ClipboardEvent, String> {
    let sync_dir = files::sync_dir();
    if !sync_dir.exists() {
        return Err("GitMemo not initialized".into());
    }

    let now = chrono::Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let time_str = now.format("%H-%M-%S").to_string();

    // Create clips directory
    let clips_dir = sync_dir.join("clips").join(&date_str);
    std::fs::create_dir_all(&clips_dir).map_err(|e| e.to_string())?;

    // Generate title from first line
    let title: String = content
        .lines()
        .next()
        .unwrap_or("clip")
        .chars()
        .take(30)
        .collect::<String>()
        .trim()
        .to_string();

    let safe_title: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let filename = format!("{}-{}.md", time_str, safe_title);
    let full_path = clips_dir.join(&filename);
    let rel_path = format!("clips/{}/{}", date_str, filename);

    // Write markdown
    let md = format!(
        "---\ndate: {}\nsource: clipboard\nchars: {}\n---\n\n{}\n",
        now.format("%Y-%m-%d %H:%M:%S"),
        content.len(),
        content
    );
    std::fs::write(&full_path, &md).map_err(|e| e.to_string())?;

    // Async git sync (don't block)
    let dir = sync_dir.clone();
    let msg = format!("clip: {}", title.chars().take(40).collect::<String>());
    std::thread::spawn(move || {
        let _ = git::commit_and_push(&dir, &msg);
    });

    let preview: String = content.chars().take(80).collect();

    Ok(ClipboardEvent {
        saved: true,
        path: rel_path,
        preview,
        timestamp: now.format("%H:%M:%S").to_string(),
    })
}

fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
