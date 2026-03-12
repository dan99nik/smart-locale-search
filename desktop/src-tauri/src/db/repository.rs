use crate::models::file::{FileChunk, IndexedFile, IndexedRoot, IndexStats};
use crate::models::search::SearchResult;
use rusqlite::{params, Connection, Transaction};

pub fn add_root(conn: &Connection, path: &str) -> Result<i64, rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO indexed_roots (path, added_time) VALUES (?1, ?2)",
        params![path, now],
    )?;
    let id = conn.query_row(
        "SELECT id FROM indexed_roots WHERE path = ?1",
        params![path],
        |row| row.get(0),
    )?;
    Ok(id)
}

pub fn remove_root(conn: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    let file_ids: Vec<i64> = {
        let mut stmt = conn.prepare("SELECT id FROM indexed_files WHERE root_id = ?1")?;
        let rows = stmt.query_map(params![id], |row| row.get(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };
    for file_id in &file_ids {
        conn.execute(
            "DELETE FROM vector_index WHERE chunk_id IN (SELECT id FROM file_chunks WHERE file_id = ?1)",
            params![file_id],
        )?;
        conn.execute("DELETE FROM image_vector_index WHERE file_id = ?1", params![file_id])?;
        conn.execute("DELETE FROM file_chunks WHERE file_id = ?1", params![file_id])?;
    }
    conn.execute("DELETE FROM indexed_files WHERE root_id = ?1", params![id])?;
    conn.execute("DELETE FROM indexed_roots WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_roots(conn: &Connection) -> Result<Vec<IndexedRoot>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT id, path, added_time, last_scan, enabled FROM indexed_roots ORDER BY path ASC")?;
    let roots = stmt
        .query_map([], |row| {
            Ok(IndexedRoot {
                id: row.get(0)?,
                path: row.get(1)?,
                added_time: row.get(2)?,
                last_scan: row.get(3)?,
                enabled: row.get::<_, i64>(4).unwrap_or(1) != 0,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(roots)
}

pub fn set_root_enabled(conn: &Connection, id: i64, enabled: bool) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE indexed_roots SET enabled = ?1 WHERE id = ?2",
        params![enabled as i64, id],
    )?;
    Ok(())
}

pub fn get_enabled_root_ids(conn: &Connection) -> Result<Vec<i64>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id FROM indexed_roots WHERE enabled = 1")?;
    let ids = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(ids)
}

pub fn count_files_for_root(conn: &Connection, root_id: i64) -> Result<i64, rusqlite::Error> {
    conn.query_row(
        "SELECT COUNT(*) FROM indexed_files WHERE root_id = ?1",
        params![root_id],
        |row| row.get(0),
    )
}

/// Fast check: does the file exist and has it changed? Returns (id, changed).
/// Uses mtime + size for a fast skip without reading file contents.
/// Also treats files with no chunks as "changed" so new extractors can process them.
pub fn check_file_changed(
    conn: &Connection,
    path: &str,
    modified_time: &str,
    size: i64,
) -> Result<Option<(i64, bool)>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT f.id, f.modified_time, f.size, (SELECT COUNT(*) FROM file_chunks c WHERE c.file_id = f.id) as chunk_count
         FROM indexed_files f WHERE f.path = ?1",
    )?;
    let result = stmt
        .query_row(params![path], |row| {
            let id: i64 = row.get(0)?;
            let db_mtime: String = row.get(1)?;
            let db_size: i64 = row.get(2)?;
            let chunk_count: i64 = row.get(3)?;
            Ok((id, db_mtime, db_size, chunk_count))
        })
        .ok();

    match result {
        None => Ok(None),
        Some((id, db_mtime, db_size, chunk_count)) => {
            let changed = db_mtime != modified_time || db_size != size || chunk_count == 0;
            Ok(Some((id, changed)))
        }
    }
}

pub fn upsert_file(conn: &Connection, file: &IndexedFile) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO indexed_files (root_id, path, filename, extension, size, modified_time, indexed_time, content_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(path) DO UPDATE SET
             size = excluded.size,
             modified_time = excluded.modified_time,
             indexed_time = excluded.indexed_time,
             content_hash = excluded.content_hash",
        params![
            file.root_id,
            file.path,
            file.filename,
            file.extension,
            file.size,
            file.modified_time,
            file.indexed_time,
            file.content_hash,
        ],
    )?;
    let id = conn.query_row(
        "SELECT id FROM indexed_files WHERE path = ?1",
        params![file.path],
        |row| row.get(0),
    )?;
    Ok(id)
}

/// Batch upsert files inside an existing transaction. Returns vec of (file_id, index_in_batch).
pub fn upsert_file_batch(tx: &Transaction, files: &[IndexedFile]) -> Result<Vec<i64>, rusqlite::Error> {
    let mut ids = Vec::with_capacity(files.len());
    let mut insert_stmt = tx.prepare_cached(
        "INSERT INTO indexed_files (root_id, path, filename, extension, size, modified_time, indexed_time, content_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(path) DO UPDATE SET
             size = excluded.size,
             modified_time = excluded.modified_time,
             indexed_time = excluded.indexed_time,
             content_hash = excluded.content_hash",
    )?;
    let mut select_stmt = tx.prepare_cached(
        "SELECT id FROM indexed_files WHERE path = ?1",
    )?;

    for file in files {
        insert_stmt.execute(params![
            file.root_id,
            file.path,
            file.filename,
            file.extension,
            file.size,
            file.modified_time,
            file.indexed_time,
            file.content_hash,
        ])?;
        let id: i64 = select_stmt.query_row(params![file.path], |row| row.get(0))?;
        ids.push(id);
    }
    Ok(ids)
}

pub fn delete_chunks_for_file(conn: &Connection, file_id: i64) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM vector_index WHERE chunk_id IN (SELECT id FROM file_chunks WHERE file_id = ?1)",
        params![file_id],
    )?;
    conn.execute("DELETE FROM file_chunks WHERE file_id = ?1", params![file_id])?;
    Ok(())
}

pub fn delete_chunks_batch(tx: &Transaction, file_ids: &[i64]) -> Result<(), rusqlite::Error> {
    let mut del_vec = tx.prepare_cached(
        "DELETE FROM vector_index WHERE chunk_id IN (SELECT id FROM file_chunks WHERE file_id = ?1)",
    )?;
    let mut del_img_vec = tx.prepare_cached(
        "DELETE FROM image_vector_index WHERE file_id = ?1",
    )?;
    let mut del_chunk = tx.prepare_cached(
        "DELETE FROM file_chunks WHERE file_id = ?1",
    )?;
    for &fid in file_ids {
        del_vec.execute(params![fid])?;
        del_img_vec.execute(params![fid])?;
        del_chunk.execute(params![fid])?;
    }
    Ok(())
}

pub fn insert_chunk(conn: &Connection, chunk: &FileChunk) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO file_chunks (file_id, chunk_idx, content, byte_start, byte_end, line_start, line_end, symbol_name)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            chunk.file_id,
            chunk.chunk_idx,
            chunk.content,
            chunk.byte_start,
            chunk.byte_end,
            chunk.line_start,
            chunk.line_end,
            chunk.symbol_name,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn insert_chunks_batch(tx: &Transaction, chunks: &[FileChunk]) -> Result<Vec<i64>, rusqlite::Error> {
    let mut stmt = tx.prepare_cached(
        "INSERT INTO file_chunks (file_id, chunk_idx, content, byte_start, byte_end, line_start, line_end, symbol_name)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )?;
    let mut ids = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        stmt.execute(params![
            chunk.file_id,
            chunk.chunk_idx,
            chunk.content,
            chunk.byte_start,
            chunk.byte_end,
            chunk.line_start,
            chunk.line_end,
            chunk.symbol_name,
        ])?;
        ids.push(tx.last_insert_rowid());
    }
    Ok(ids)
}

pub fn insert_vector(conn: &Connection, chunk_id: i64, embedding: &[u8]) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO vector_index (chunk_id, embedding) VALUES (?1, ?2)",
        params![chunk_id, embedding],
    )?;
    Ok(())
}

pub fn insert_vectors_batch(tx: &Transaction, vectors: &[(i64, Vec<u8>)]) -> Result<(), rusqlite::Error> {
    let mut stmt = tx.prepare_cached(
        "INSERT INTO vector_index (chunk_id, embedding) VALUES (?1, ?2)",
    )?;
    for (chunk_id, embedding) in vectors {
        stmt.execute(params![chunk_id, embedding])?;
    }
    Ok(())
}

pub fn update_root_scan_time(conn: &Connection, root_id: i64) -> Result<(), rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE indexed_roots SET last_scan = ?1 WHERE id = ?2",
        params![now, root_id],
    )?;
    Ok(())
}

pub fn get_index_stats(conn: &Connection) -> Result<IndexStats, rusqlite::Error> {
    let total_files: i64 =
        conn.query_row("SELECT COUNT(*) FROM indexed_files", [], |row| row.get(0))?;
    let total_chunks: i64 =
        conn.query_row("SELECT COUNT(*) FROM file_chunks", [], |row| row.get(0))?;
    let total_vectors: i64 =
        conn.query_row("SELECT COUNT(*) FROM vector_index", [], |row| row.get(0))?;
    let total_roots: i64 =
        conn.query_row("SELECT COUNT(*) FROM indexed_roots", [], |row| row.get(0))?;

    let page_count: i64 =
        conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
    let page_size: i64 =
        conn.query_row("PRAGMA page_size", [], |row| row.get(0))?;
    let db_size_bytes = page_count * page_size;

    Ok(IndexStats {
        total_files,
        total_chunks,
        total_vectors,
        total_roots,
        db_size_bytes,
    })
}

pub fn search_filename(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, rusqlite::Error> {
    let fts_query = query
        .split_whitespace()
        .map(|w| format!("\"{}\"*", w))
        .collect::<Vec<_>>()
        .join(" ");

    let mut stmt = conn.prepare(
        "SELECT f.id, f.path, f.filename, f.extension, f.size, f.modified_time,
                rank
         FROM files_fts fts
         JOIN indexed_files f ON f.id = fts.rowid
         JOIN indexed_roots r ON r.id = f.root_id
         WHERE files_fts MATCH ?1 AND r.enabled = 1
         ORDER BY rank
         LIMIT ?2",
    )?;

    let results = stmt
        .query_map(params![fts_query, limit as i64], |row| {
            let rank: f64 = row.get(6)?;
            Ok(SearchResult {
                file_id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                extension: row.get(3)?,
                size: row.get(4)?,
                modified_time: row.get(5)?,
                snippet: None,
                score: -rank,
                match_type: "filename".to_string(),
                line_start: None,
                line_end: None,
                symbol_name: None,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(results)
}

pub fn search_content(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, rusqlite::Error> {
    let fts_query = query
        .split_whitespace()
        .map(|w| format!("\"{}\"*", w))
        .collect::<Vec<_>>()
        .join(" ");

    let mut stmt = conn.prepare(
        "SELECT f.id, f.path, f.filename, f.extension, f.size, f.modified_time,
                snippet(file_chunks_fts, 0, '<mark>', '</mark>', '...', 40) as snip,
                fts.rank,
                c.line_start, c.line_end, c.symbol_name
         FROM file_chunks_fts fts
         JOIN file_chunks c ON c.id = fts.rowid
         JOIN indexed_files f ON f.id = c.file_id
         JOIN indexed_roots r ON r.id = f.root_id
         WHERE file_chunks_fts MATCH ?1 AND r.enabled = 1
         GROUP BY f.id
         ORDER BY fts.rank
         LIMIT ?2",
    )?;

    let results = stmt
        .query_map(params![fts_query, limit as i64], |row| {
            let rank: f64 = row.get(7)?;
            Ok(SearchResult {
                file_id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                extension: row.get(3)?,
                size: row.get(4)?,
                modified_time: row.get(5)?,
                snippet: row.get(6)?,
                score: -rank,
                match_type: "content".to_string(),
                line_start: row.get(8)?,
                line_end: row.get(9)?,
                symbol_name: row.get(10)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(results)
}

pub fn search_fuzzy(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, rusqlite::Error> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT f.id, f.path, f.filename, f.extension, f.size, f.modified_time
         FROM indexed_files f
         JOIN indexed_roots r ON r.id = f.root_id
         WHERE (f.filename LIKE ?1 OR f.path LIKE ?1) AND r.enabled = 1
         ORDER BY f.modified_time DESC
         LIMIT ?2",
    )?;

    let results = stmt
        .query_map(params![pattern, limit as i64], |row| {
            Ok(SearchResult {
                file_id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                extension: row.get(3)?,
                size: row.get(4)?,
                modified_time: row.get(5)?,
                snippet: None,
                score: 0.5,
                match_type: "fuzzy".to_string(),
                line_start: None,
                line_end: None,
                symbol_name: None,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(results)
}

pub fn get_all_vectors(conn: &Connection) -> Result<Vec<(i64, Vec<f32>)>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT chunk_id, embedding FROM vector_index")?;
    let vectors = stmt
        .query_map([], |row| {
            let chunk_id: i64 = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            let floats: Vec<f32> = blob
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            Ok((chunk_id, floats))
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(vectors)
}

pub fn get_file_for_chunk(
    conn: &Connection,
    chunk_id: i64,
) -> Result<SearchResult, rusqlite::Error> {
    get_file_for_chunk_with_query(conn, chunk_id, None)
}

pub fn get_file_for_chunk_with_query(
    conn: &Connection,
    chunk_id: i64,
    query: Option<&str>,
) -> Result<SearchResult, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT f.id, f.path, f.filename, f.extension, f.size, f.modified_time, c.content,
                c.line_start, c.line_end, c.symbol_name
         FROM file_chunks c
         JOIN indexed_files f ON f.id = c.file_id
         JOIN indexed_roots r ON r.id = f.root_id
         WHERE c.id = ?1 AND r.enabled = 1",
    )?;
    stmt.query_row(params![chunk_id], |row| {
        let content: Option<String> = row.get(6)?;
        let snippet = content.map(|c| make_snippet(&c, query));
        Ok(SearchResult {
            file_id: row.get(0)?,
            path: row.get(1)?,
            filename: row.get(2)?,
            extension: row.get(3)?,
            size: row.get(4)?,
            modified_time: row.get(5)?,
            snippet,
            score: 0.0,
            match_type: "semantic".to_string(),
            line_start: row.get(7)?,
            line_end: row.get(8)?,
            symbol_name: row.get(9)?,
        })
    })
}

fn make_snippet(content: &str, query: Option<&str>) -> String {
    const WINDOW: usize = 200;

    if content.len() <= WINDOW {
        return content.to_string();
    }

    if let Some(q) = query {
        let lower_content = content.to_lowercase();
        for term in q.split_whitespace().filter(|t| t.len() >= 2) {
            if let Some(pos) = lower_content.find(&term.to_lowercase()) {
                let half = WINDOW / 2;
                let raw_start = pos.saturating_sub(half);
                let raw_end = (pos + term.len() + half).min(content.len());

                let mut start = raw_start;
                while start > 0 && !content.is_char_boundary(start) {
                    start -= 1;
                }
                let mut end = raw_end;
                while end < content.len() && !content.is_char_boundary(end) {
                    end += 1;
                }

                let prefix = if start > 0 { "..." } else { "" };
                let suffix = if end < content.len() { "..." } else { "" };
                return format!("{}{}{}", prefix, &content[start..end], suffix);
            }
        }
    }

    let mut end = WINDOW;
    while end < content.len() && !content.is_char_boundary(end) {
        end += 1;
    }
    format!("{}...", &content[..end])
}

pub fn get_existing_file(
    conn: &Connection,
    path: &str,
) -> Result<Option<(i64, Option<String>)>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, content_hash FROM indexed_files WHERE path = ?1",
    )?;
    let result = stmt
        .query_row(params![path], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
        })
        .ok();
    Ok(result)
}

/// Remove indexed_files entries whose paths no longer exist on disk for a given root.
pub fn cleanup_deleted_files(conn: &Connection, root_id: i64) -> Result<usize, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, path FROM indexed_files WHERE root_id = ?1")?;
    let rows: Vec<(i64, String)> = stmt
        .query_map(params![root_id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    let mut removed = 0;
    for (file_id, path) in &rows {
        if !std::path::Path::new(path).exists() {
            conn.execute(
                "DELETE FROM vector_index WHERE chunk_id IN (SELECT id FROM file_chunks WHERE file_id = ?1)",
                params![file_id],
            )?;
            conn.execute("DELETE FROM image_vector_index WHERE file_id = ?1", params![file_id])?;
            conn.execute("DELETE FROM file_chunks WHERE file_id = ?1", params![file_id])?;
            conn.execute("DELETE FROM indexed_files WHERE id = ?1", params![file_id])?;
            removed += 1;
        }
    }
    Ok(removed)
}

// ── Image vector operations ──────────────────────────────────────────

pub fn upsert_image_vector(conn: &Connection, file_id: i64, embedding: &[u8]) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO image_vector_index (file_id, embedding) VALUES (?1, ?2)
         ON CONFLICT(file_id) DO UPDATE SET embedding = excluded.embedding",
        params![file_id, embedding],
    )?;
    Ok(())
}

pub fn upsert_image_vectors_batch(tx: &Transaction, vectors: &[(i64, Vec<u8>)]) -> Result<(), rusqlite::Error> {
    let mut stmt = tx.prepare_cached(
        "INSERT INTO image_vector_index (file_id, embedding) VALUES (?1, ?2)
         ON CONFLICT(file_id) DO UPDATE SET embedding = excluded.embedding",
    )?;
    for (file_id, embedding) in vectors {
        stmt.execute(params![file_id, embedding])?;
    }
    Ok(())
}

pub fn get_all_image_vectors(conn: &Connection) -> Result<Vec<(i64, Vec<f32>)>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT iv.file_id, iv.embedding
         FROM image_vector_index iv
         JOIN indexed_files f ON f.id = iv.file_id
         JOIN indexed_roots r ON r.id = f.root_id
         WHERE r.enabled = 1"
    )?;
    let vectors = stmt
        .query_map([], |row| {
            let file_id: i64 = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            let floats: Vec<f32> = blob
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            Ok((file_id, floats))
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(vectors)
}

pub fn get_file_by_id(conn: &Connection, file_id: i64) -> Result<SearchResult, rusqlite::Error> {
    conn.query_row(
        "SELECT f.id, f.path, f.filename, f.extension, f.size, f.modified_time
         FROM indexed_files f
         JOIN indexed_roots r ON r.id = f.root_id
         WHERE f.id = ?1 AND r.enabled = 1",
        params![file_id],
        |row| {
            Ok(SearchResult {
                file_id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                extension: row.get(3)?,
                size: row.get(4)?,
                modified_time: row.get(5)?,
                snippet: None,
                score: 0.0,
                match_type: "image_semantic".to_string(),
                line_start: None,
                line_end: None,
                symbol_name: None,
            })
        },
    )
}

pub fn has_image_vector(conn: &Connection, file_id: i64) -> Result<bool, rusqlite::Error> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM image_vector_index WHERE file_id = ?1",
        params![file_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}
