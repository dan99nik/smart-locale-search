use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use log::info;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};

use crate::db::repository;
use crate::db::Database;
use crate::embedding::{EmbeddingProvider, ImageEmbeddingProvider};
use crate::models::file::{file_category, FileCategory, FileChunk, IndexedFile};
use crate::INDEXING_MODE;
use super::chunker;
use super::code_chunker;
use super::config;
use super::extractor;
use super::scanner;

pub static INDEXING_CANCEL: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Serialize)]
pub struct IndexingEvent {
    pub phase: String,
    pub processed: usize,
    pub total: usize,
    pub percent: f64,
    pub current_file: String,
    pub status: String,
    pub skipped: usize,
    pub indexed: usize,
    pub errors: usize,
}

fn emit_progress(app: &AppHandle, event: &IndexingEvent) {
    let _ = app.emit("indexing-progress", event.clone());
}

fn throttle_if_idle_mode() {
    if INDEXING_MODE.load(Ordering::Relaxed) == 1 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn is_cancelled() -> bool {
    INDEXING_CANCEL.load(Ordering::Relaxed)
}

struct BatchItem {
    file: IndexedFile,
    text: Option<String>,
    generate_embedding: bool,
    is_image: bool,
}

pub fn index_root(
    db: &Database,
    embedding_provider: Option<&Arc<dyn EmbeddingProvider>>,
    root_path: &str,
    app: &AppHandle,
) -> Result<IndexRootResult, String> {
    index_root_full(db, embedding_provider, None, root_path, app)
}

pub fn index_root_full(
    db: &Database,
    embedding_provider: Option<&Arc<dyn EmbeddingProvider>>,
    vision_provider: Option<&Arc<dyn ImageEmbeddingProvider>>,
    root_path: &str,
    app: &AppHandle,
) -> Result<IndexRootResult, String> {
    let root = Path::new(root_path);
    if !root.exists() || !root.is_dir() {
        return Err(format!("Path does not exist or is not a directory: {}", root_path));
    }

    let root_id = {
        let conn = db.lock_conn();
        repository::add_root(&conn, root_path).map_err(|e| e.to_string())?
    };

    emit_progress(app, &IndexingEvent {
        phase: "scanning".into(),
        processed: 0,
        total: 0,
        percent: 0.0,
        current_file: root_path.into(),
        status: format!("Counting files in {}…", root_path.split('/').last().unwrap_or(root_path)),
        skipped: 0,
        indexed: 0,
        errors: 0,
    });

    let estimated_total = scanner::count_files(root);
    info!("Estimated {} files in {}", estimated_total, root_path);

    emit_progress(app, &IndexingEvent {
        phase: "scanning".into(),
        processed: 0,
        total: estimated_total,
        percent: 0.0,
        current_file: root_path.into(),
        status: format!("Found ~{} files, starting indexing…", estimated_total),
        skipped: 0,
        indexed: 0,
        errors: 0,
    });

    let now = chrono::Utc::now().to_rfc3339();
    let mut processed = 0usize;
    let mut skipped = 0usize;
    let mut indexed = 0usize;
    let mut errors = 0usize;
    let mut batch: Vec<BatchItem> = Vec::with_capacity(config::DB_BATCH_SIZE);

    for entry in scanner::scan_directory_iter(root) {
        if is_cancelled() {
            info!("Indexing cancelled at {}/{}", processed, estimated_total);
            break;
        }
        throttle_if_idle_mode();

        let path_str = entry.path.to_string_lossy().to_string();
        let filename = entry.path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let extension = entry.path
            .extension()
            .map(|e| e.to_string_lossy().to_string());

        let metadata = match std::fs::metadata(&entry.path) {
            Ok(m) => m,
            Err(_) => {
                processed += 1;
                errors += 1;
                continue;
            }
        };

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| {
                let dt: chrono::DateTime<chrono::Utc> = t.into();
                Some(dt.to_rfc3339())
            })
            .unwrap_or_else(|| now.clone());

        // Incremental check: skip if mtime+size unchanged
        {
            let conn = db.lock_conn();
            match repository::check_file_changed(&conn, &path_str, &modified, entry.size as i64) {
                Ok(Some((_id, false))) => {
                    // File unchanged
                    processed += 1;
                    skipped += 1;
                    if processed % config::PROGRESS_EMIT_INTERVAL == 0 {
                        emit_progress(app, &IndexingEvent {
                            phase: "indexing".into(),
                            processed,
                            total: estimated_total,
                            percent: progress_percent(processed, estimated_total),
                            current_file: filename.clone(),
                            status: format!("Indexing… {}/{} ({} skipped)", processed, estimated_total, skipped),
                            skipped,
                            indexed,
                            errors,
                        });
                    }
                    continue;
                }
                Ok(Some((_id, true))) => { /* changed, will re-index */ }
                Ok(None) => { /* new file */ }
                Err(_) => { /* DB error, try to index anyway */ }
            }
        }

        let text_content = extractor::extract_text(&entry.path);

        let content_hash = text_content.as_ref().map(|t| {
            let mut hasher = Sha256::new();
            hasher.update(t.as_bytes());
            hex::encode(hasher.finalize())
        });

        let ext_str = extension.as_deref().unwrap_or("");
        let text_len = text_content.as_ref().map(|t| t.len()).unwrap_or(0);
        let generate_embedding = config::should_generate_embedding(ext_str, text_len);
        let is_image = file_category(ext_str) == FileCategory::Image;

        batch.push(BatchItem {
            file: IndexedFile {
                id: 0,
                root_id,
                path: path_str.clone(),
                filename: filename.clone(),
                extension,
                size: entry.size as i64,
                modified_time: modified,
                indexed_time: now.clone(),
                content_hash,
            },
            text: text_content,
            generate_embedding,
            is_image,
        });

        processed += 1;

        if batch.len() >= config::DB_BATCH_SIZE {
            let result = flush_batch(db, embedding_provider, vision_provider, &mut batch, app);
            indexed += result.indexed;
            errors += result.errors;

            emit_progress(app, &IndexingEvent {
                phase: "indexing".into(),
                processed,
                total: estimated_total,
                percent: progress_percent(processed, estimated_total),
                current_file: filename.clone(),
                status: format!("Indexing… {}/{} ({} skipped)", processed, estimated_total, skipped),
                skipped,
                indexed,
                errors,
            });
        }

        if processed % config::PROGRESS_EMIT_INTERVAL == 0 {
            emit_progress(app, &IndexingEvent {
                phase: "indexing".into(),
                processed,
                total: estimated_total,
                percent: progress_percent(processed, estimated_total),
                current_file: filename.clone(),
                status: format!("Indexing… {}/{} ({} skipped)", processed, estimated_total, skipped),
                skipped,
                indexed,
                errors,
            });
        }
    }

    // Flush remaining batch
    if !batch.is_empty() {
        let result = flush_batch(db, embedding_provider, vision_provider, &mut batch, app);
        indexed += result.indexed;
        errors += result.errors;
    }

    // Clean up files that no longer exist on disk
    if !is_cancelled() {
        let conn = db.lock_conn();
        match repository::cleanup_deleted_files(&conn, root_id) {
            Ok(removed) => {
                if removed > 0 {
                    info!("Cleaned up {} deleted files from index for root {}", removed, root_path);
                }
            }
            Err(e) => log::warn!("Failed to clean up deleted files: {}", e),
        }
        let _ = repository::update_root_scan_time(&conn, root_id);
    }

    info!(
        "Indexing {}: processed={} indexed={} skipped={} errors={}",
        root_path, processed, indexed, skipped, errors
    );

    Ok(IndexRootResult { processed, indexed, skipped, errors })
}

pub struct IndexRootResult {
    pub processed: usize,
    pub indexed: usize,
    pub skipped: usize,
    pub errors: usize,
}

struct FlushResult {
    indexed: usize,
    errors: usize,
}

fn flush_batch(
    db: &Database,
    embedding_provider: Option<&Arc<dyn EmbeddingProvider>>,
    vision_provider: Option<&Arc<dyn ImageEmbeddingProvider>>,
    batch: &mut Vec<BatchItem>,
    _app: &AppHandle,
) -> FlushResult {
    let mut indexed = 0usize;
    let mut errors = 0usize;

    let mut conn = db.lock_conn();

    // Phase 1: fast transaction for file metadata + text chunks
    {
        let tx = match conn.transaction() {
            Ok(tx) => tx,
            Err(e) => {
                log::error!("Failed to start transaction: {}", e);
                batch.clear();
                return FlushResult { indexed: 0, errors: batch.len() };
            }
        };

        let files: Vec<IndexedFile> = batch.iter().map(|b| b.file.clone()).collect();
        let file_ids = match repository::upsert_file_batch(&tx, &files) {
            Ok(ids) => ids,
            Err(e) => {
                log::error!("Batch upsert failed: {}", e);
                let _ = tx.rollback();
                errors += batch.len();
                batch.clear();
                return FlushResult { indexed, errors };
            }
        };

        let _ = repository::delete_chunks_batch(&tx, &file_ids);

        let mut all_chunks: Vec<FileChunk> = Vec::new();
        let mut chunk_embed_flags: Vec<bool> = Vec::new();

        for (i, item) in batch.iter().enumerate() {
            if let Some(ref text) = item.text {
                let ext = item.file.extension.as_deref().unwrap_or("");
                let is_code = file_category(ext) == FileCategory::Code;

                if is_code {
                    let code_chunks = code_chunker::chunk_code(text, ext);
                    let cap = code_chunks.len().min(config::MAX_CHUNKS_PER_FILE);
                    for (idx, chunk) in code_chunks.iter().take(cap).enumerate() {
                        all_chunks.push(FileChunk {
                            id: 0,
                            file_id: file_ids[i],
                            chunk_idx: idx as i32,
                            content: chunk.content.clone(),
                            byte_start: Some(chunk.byte_start as i64),
                            byte_end: Some(chunk.byte_end as i64),
                            line_start: Some(chunk.line_start as i64),
                            line_end: Some(chunk.line_end as i64),
                            symbol_name: chunk.symbol_name.clone(),
                        });
                        chunk_embed_flags.push(item.generate_embedding);
                    }
                } else {
                    let chunks = chunker::chunk_text(text);
                    let cap = chunks.len().min(config::MAX_CHUNKS_PER_FILE);
                    for (idx, chunk) in chunks.iter().take(cap).enumerate() {
                        all_chunks.push(FileChunk {
                            id: 0,
                            file_id: file_ids[i],
                            chunk_idx: idx as i32,
                            content: chunk.content.clone(),
                            byte_start: Some(chunk.byte_start as i64),
                            byte_end: Some(chunk.byte_end as i64),
                            line_start: None,
                            line_end: None,
                            symbol_name: None,
                        });
                        chunk_embed_flags.push(item.generate_embedding);
                    }
                }
            }
            indexed += 1;
        }

        let chunk_ids = match repository::insert_chunks_batch(&tx, &all_chunks) {
            Ok(ids) => ids,
            Err(e) => {
                log::error!("Batch chunk insert failed: {}", e);
                let _ = tx.rollback();
                batch.clear();
                return FlushResult { indexed: 0, errors: batch.len() };
            }
        };

        if let Err(e) = tx.commit() {
            log::error!("Transaction commit failed: {}", e);
            batch.clear();
            return FlushResult { indexed: 0, errors: batch.len() };
        }

        // Phase 2: generate embeddings outside the main transaction
        if let Some(provider) = embedding_provider {
            let embed_pairs: Vec<(i64, &str)> = chunk_ids.iter().zip(all_chunks.iter()).zip(chunk_embed_flags.iter())
                .filter_map(|((id, chunk), &needs_embed)| {
                    if needs_embed { Some((*id, chunk.content.as_str())) } else { None }
                })
                .collect();

            if !embed_pairs.is_empty() {
                let mut vectors: Vec<(i64, Vec<u8>)> = Vec::new();

                for (chunk_id, content) in &embed_pairs {
                    if is_cancelled() { break; }
                    throttle_if_idle_mode();

                    match provider.embed(content) {
                        Ok(embedding) => {
                            let bytes: Vec<u8> = embedding
                                .iter()
                                .flat_map(|f| f.to_le_bytes())
                                .collect();
                            vectors.push((*chunk_id, bytes));
                        }
                        Err(_) => {}
                    }
                }

                if !vectors.is_empty() {
                    let tx2 = match conn.transaction() {
                        Ok(tx) => tx,
                        Err(e) => {
                            log::warn!("Failed to start vector transaction: {}", e);
                            batch.clear();
                            return FlushResult { indexed, errors };
                        }
                    };
                    if let Err(e) = repository::insert_vectors_batch(&tx2, &vectors) {
                        log::warn!("Batch vector insert failed: {}", e);
                    }
                    let _ = tx2.commit();
                }
            }
        }

        // Phase 3: generate CLIP image embeddings
        if let Some(vp) = vision_provider {
            let image_items: Vec<(i64, &str)> = batch.iter().enumerate()
                .filter_map(|(i, item)| {
                    if item.is_image { Some((file_ids[i], item.file.path.as_str())) } else { None }
                })
                .collect();

            if !image_items.is_empty() {
                let mut img_vectors: Vec<(i64, Vec<u8>)> = Vec::new();

                for (file_id, path_str) in &image_items {
                    if is_cancelled() { break; }
                    throttle_if_idle_mode();

                    let img_path = std::path::Path::new(path_str);
                    match vp.embed_image(img_path) {
                        Ok(embedding) => {
                            let bytes: Vec<u8> = embedding
                                .iter()
                                .flat_map(|f| f.to_le_bytes())
                                .collect();
                            img_vectors.push((*file_id, bytes));
                        }
                        Err(e) => {
                            log::debug!("CLIP embed failed for {}: {}", path_str, e);
                        }
                    }
                }

                if !img_vectors.is_empty() {
                    let tx3 = match conn.transaction() {
                        Ok(tx) => tx,
                        Err(e) => {
                            log::warn!("Failed to start image vector transaction: {}", e);
                            batch.clear();
                            return FlushResult { indexed, errors };
                        }
                    };
                    if let Err(e) = repository::upsert_image_vectors_batch(&tx3, &img_vectors) {
                        log::warn!("Image vector batch insert failed: {}", e);
                    }
                    let _ = tx3.commit();
                }
            }
        }
    }

    batch.clear();
    FlushResult { indexed, errors }
}

fn progress_percent(processed: usize, total: usize) -> f64 {
    if total == 0 { 0.0 } else { (processed as f64 / total as f64) * 100.0 }
}

pub fn reindex_all(
    db: &Database,
    embedding_provider: Option<&Arc<dyn EmbeddingProvider>>,
    app: &AppHandle,
) -> Result<usize, String> {
    reindex_all_full(db, embedding_provider, None, app)
}

pub fn reindex_all_full(
    db: &Database,
    embedding_provider: Option<&Arc<dyn EmbeddingProvider>>,
    vision_provider: Option<&Arc<dyn ImageEmbeddingProvider>>,
    app: &AppHandle,
) -> Result<usize, String> {
    INDEXING_CANCEL.store(false, Ordering::Relaxed);

    let roots = {
        let conn = db.lock_conn();
        repository::get_roots(&conn).map_err(|e| e.to_string())?
    };

    let enabled: Vec<_> = roots.into_iter().filter(|r| r.enabled).collect();
    let total_roots = enabled.len();
    let mut grand_indexed = 0usize;
    let mut grand_skipped = 0usize;
    let mut grand_errors = 0usize;

    for (i, root) in enabled.iter().enumerate() {
        if is_cancelled() {
            break;
        }
        emit_progress(app, &IndexingEvent {
            phase: "scanning".into(),
            processed: 0,
            total: 0,
            percent: 0.0,
            current_file: root.path.clone(),
            status: format!("Scanning folder {}/{}…", i + 1, total_roots),
            skipped: grand_skipped,
            indexed: grand_indexed,
            errors: grand_errors,
        });
        match index_root_full(db, embedding_provider, vision_provider, &root.path, app) {
            Ok(result) => {
                grand_indexed += result.indexed;
                grand_skipped += result.skipped;
                grand_errors += result.errors;
            }
            Err(e) => log::error!("Failed to reindex {}: {}", root.path, e),
        }
    }

    let status = if is_cancelled() {
        format!("Stopped — {} indexed, {} skipped", grand_indexed, grand_skipped)
    } else {
        format!("Completed — {} indexed, {} skipped", grand_indexed, grand_skipped)
    };

    emit_progress(app, &IndexingEvent {
        phase: "completed".into(),
        processed: grand_indexed + grand_skipped,
        total: grand_indexed + grand_skipped,
        percent: 100.0,
        current_file: String::new(),
        status,
        skipped: grand_skipped,
        indexed: grand_indexed,
        errors: grand_errors,
    });

    Ok(grand_indexed)
}

pub fn cancel_indexing() {
    INDEXING_CANCEL.store(true, Ordering::Relaxed);
}
