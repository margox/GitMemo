use gitmemo_core::storage::{database, files};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub source_type: String,
    pub title: String,
    pub file_path: String,
    pub snippet: String,
    pub date: String,
}

#[tauri::command]
pub fn search_all(
    query: String,
    type_filter: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<SearchResultItem>, String> {
    let sync_dir = files::sync_dir();
    if !sync_dir.exists() {
        return Err("GitMemo 未初始化".into());
    }

    let db_path = sync_dir.join(".metadata").join("index.db");
    let conn = database::open_or_create(&db_path).map_err(|e| e.to_string())?;
    database::build_index(&conn, &sync_dir).map_err(|e| e.to_string())?;

    let filter = type_filter.as_deref().unwrap_or("all");
    let max = limit.unwrap_or(20);

    let results = database::search(&conn, &query, filter, max).map_err(|e| e.to_string())?;

    Ok(results
        .into_iter()
        .map(|r| SearchResultItem {
            source_type: r.source_type,
            title: r.title,
            file_path: r.file_path,
            snippet: r.snippet,
            date: r.date,
        })
        .collect())
}

#[tauri::command]
pub fn recent_conversations(limit: Option<usize>, days: Option<u32>) -> Result<Vec<SearchResultItem>, String> {
    let sync_dir = files::sync_dir();
    if !sync_dir.exists() {
        return Err("GitMemo 未初始化".into());
    }

    let db_path = sync_dir.join(".metadata").join("index.db");
    let conn = database::open_or_create(&db_path).map_err(|e| e.to_string())?;
    database::build_index(&conn, &sync_dir).map_err(|e| e.to_string())?;

    let results =
        database::recent(&conn, limit.unwrap_or(20), days.unwrap_or(30)).map_err(|e| e.to_string())?;

    Ok(results
        .into_iter()
        .map(|r| SearchResultItem {
            source_type: r.source_type,
            title: r.title,
            file_path: r.file_path,
            snippet: r.snippet,
            date: r.date,
        })
        .collect())
}

#[tauri::command]
pub fn reindex() -> Result<u32, String> {
    let sync_dir = files::sync_dir();
    if !sync_dir.exists() {
        return Err("GitMemo 未初始化".into());
    }

    let db_path = sync_dir.join(".metadata").join("index.db");
    if db_path.exists() {
        let _ = std::fs::remove_file(&db_path);
    }

    let conn = database::open_or_create(&db_path).map_err(|e| e.to_string())?;
    let count = database::build_index(&conn, &sync_dir).map_err(|e| e.to_string())?;
    Ok(count)
}
