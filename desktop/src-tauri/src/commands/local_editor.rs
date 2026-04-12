//! Read-only browse of `~/.claude` and `~/.cursor` (not GitMemo sync dir).
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_VIEW_BYTES: u64 = 8 * 1024 * 1024;

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
}

fn editor_root_dir(root: &str) -> Result<PathBuf, String> {
    let home = home_dir().ok_or_else(|| "HOME/USERPROFILE not set".to_string())?;
    let p = match root {
        "claude" => home.join(".claude"),
        "cursor" => home.join(".cursor"),
        _ => return Err("root must be \"claude\" or \"cursor\"".into()),
    };
    if !p.is_dir() {
        return Err(format!("Directory does not exist: {}", p.display()));
    }
    p.canonicalize()
        .map_err(|e| format!("{}: {}", p.display(), e))
}

/// Resolve `rel` under `root` and ensure the result stays inside `root` (no `..` escape).
fn resolve_under_root(root: &Path, rel: &str) -> Result<PathBuf, String> {
    validate_rel(rel)?;
    let rel = rel.trim().replace('\\', "/");
    let joined = if rel.is_empty() {
        root.to_path_buf()
    } else {
        root.join(rel)
    };
    let joined = joined.canonicalize().map_err(|e| format!("{}", e))?;
    if !joined.starts_with(root) {
        return Err("Path escapes editor root".into());
    }
    Ok(joined)
}

fn validate_rel(rel: &str) -> Result<(), String> {
    if rel.contains("..") {
        return Err("Invalid path".into());
    }
    for seg in rel.replace('\\', "/").split('/') {
        if seg == ".." {
            return Err("Invalid path".into());
        }
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct EditorRootsStatus {
    pub claude_path: String,
    pub claude_exists: bool,
    pub cursor_path: String,
    pub cursor_exists: bool,
}

#[derive(Debug, Serialize)]
pub struct EditorDirEntry {
    pub name: String,
    /// Relative path from root using `/`.
    pub rel_path: String,
    pub is_dir: bool,
}

#[tauri::command]
pub fn get_editor_data_roots() -> Result<EditorRootsStatus, String> {
    let Some(home) = home_dir() else {
        return Ok(EditorRootsStatus {
            claude_path: String::new(),
            claude_exists: false,
            cursor_path: String::new(),
            cursor_exists: false,
        });
    };
    let claude = home.join(".claude");
    let cursor = home.join(".cursor");
    let claude_exists = claude.is_dir();
    let cursor_exists = cursor.is_dir();
    Ok(EditorRootsStatus {
        claude_path: claude.to_string_lossy().into(),
        claude_exists,
        cursor_path: cursor.to_string_lossy().into(),
        cursor_exists,
    })
}

#[tauri::command]
pub fn list_editor_directory(root: String, rel: String) -> Result<Vec<EditorDirEntry>, String> {
    let base = editor_root_dir(root.trim())?;
    let target = resolve_under_root(&base, rel.trim())?;
    if !target.is_dir() {
        return Err("Not a directory".into());
    }

    let mut out: Vec<EditorDirEntry> = Vec::new();
    for entry in fs::read_dir(&target).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        let is_dir = path.is_dir();
        let rel_path = if rel.trim().is_empty() {
            name.clone()
        } else {
            format!("{}/{}", rel.trim().trim_end_matches('/'), name)
        };
        out.push(EditorDirEntry {
            name,
            rel_path,
            is_dir,
        });
    }

    out.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(out)
}

#[tauri::command]
pub fn read_editor_home_file(root: String, rel: String) -> Result<String, String> {
    let base = editor_root_dir(root.trim())?;
    let path = resolve_under_root(&base, rel.trim())?;
    if !path.is_file() {
        return Err("Not a file".into());
    }
    let len = path.metadata().map_err(|e| e.to_string())?.len();
    if len > MAX_VIEW_BYTES {
        return Err(format!(
            "File too large to preview ({} MB; max {} MB). Open in an external editor.",
            len / 1024 / 1024,
            MAX_VIEW_BYTES / 1024 / 1024
        ));
    }
    let f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    f.take(MAX_VIEW_BYTES)
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

#[tauri::command]
pub fn resolve_editor_file_abs(root: String, rel: String) -> Result<String, String> {
    let base = editor_root_dir(root.trim())?;
    let path = resolve_under_root(&base, rel.trim())?;
    if !path.is_file() {
        return Err("Not a file".into());
    }
    Ok(path.to_string_lossy().into())
}
