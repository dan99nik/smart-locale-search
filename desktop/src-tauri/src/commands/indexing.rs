use std::sync::atomic::Ordering;

use tauri::{AppHandle, State};

use crate::db::repository;
use crate::indexing::pipeline;
use crate::models::file::{IndexStats, IndexedRoot};
use crate::{AppState, INDEXING_MODE};

#[tauri::command]
pub async fn add_folder(state: State<'_, AppState>, path: String) -> Result<IndexedRoot, String> {
    let conn = state.db.lock_conn();
    let _id = repository::add_root(&conn, &path).map_err(|e| e.to_string())?;
    let roots = repository::get_roots(&conn).map_err(|e| e.to_string())?;
    roots
        .into_iter()
        .find(|r| r.path == path)
        .ok_or_else(|| "Root not found after adding".to_string())
}

#[tauri::command]
pub async fn remove_folder(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock_conn();
    repository::remove_root(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_indexed_roots(state: State<'_, AppState>) -> Result<Vec<IndexedRoot>, String> {
    let conn = state.db.lock_conn();
    repository::get_roots(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_index_stats(state: State<'_, AppState>) -> Result<IndexStats, String> {
    let conn = state.db.lock_conn();
    repository::get_index_stats(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_root_enabled(
    state: State<'_, AppState>,
    id: i64,
    enabled: bool,
) -> Result<(), String> {
    let conn = state.db.lock_conn();
    repository::set_root_enabled(&conn, id, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_default_roots() -> Result<Vec<String>, String> {
    Ok(crate::indexing::defaults::detect_default_roots())
}

#[tauri::command]
pub async fn init_default_roots(state: State<'_, AppState>) -> Result<Vec<IndexedRoot>, String> {
    let conn = state.db.lock_conn();
    let existing = repository::get_roots(&conn).map_err(|e| e.to_string())?;
    if !existing.is_empty() {
        return Ok(existing);
    }

    let defaults = crate::indexing::defaults::detect_default_roots();
    for path in &defaults {
        let _ = repository::add_root(&conn, path);
    }
    repository::get_roots(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reindex(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let db = &state.db;
    let embed_guard = state.embedding_provider.read().map_err(|e| e.to_string())?;
    let vision_guard = state.vision_provider.read().map_err(|e| e.to_string())?;
    let embed_provider = embed_guard.as_ref();
    let vision_provider = vision_guard.as_ref();
    let total = pipeline::reindex_all_full(db, embed_provider, vision_provider, &app_handle)?;
    Ok(format!("Reindexed {} files", total))
}

#[tauri::command]
pub async fn cancel_indexing() -> Result<(), String> {
    pipeline::cancel_indexing();
    Ok(())
}

#[tauri::command]
pub async fn set_indexing_mode(mode: String) -> Result<(), String> {
    match mode.as_str() {
        "priority" => INDEXING_MODE.store(0, Ordering::Relaxed),
        "idle" => INDEXING_MODE.store(1, Ordering::Relaxed),
        _ => return Err(format!("Unknown indexing mode: {}", mode)),
    }
    Ok(())
}

#[tauri::command]
pub async fn get_indexing_mode() -> Result<String, String> {
    match INDEXING_MODE.load(Ordering::Relaxed) {
        0 => Ok("priority".to_string()),
        _ => Ok("idle".to_string()),
    }
}
