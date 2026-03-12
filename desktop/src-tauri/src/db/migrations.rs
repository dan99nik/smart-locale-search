use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS indexed_roots (
            id          INTEGER PRIMARY KEY,
            path        TEXT NOT NULL UNIQUE,
            added_time  TEXT NOT NULL,
            last_scan   TEXT
        );

        CREATE TABLE IF NOT EXISTS indexed_files (
            id            INTEGER PRIMARY KEY,
            root_id       INTEGER NOT NULL REFERENCES indexed_roots(id),
            path          TEXT NOT NULL UNIQUE,
            filename      TEXT NOT NULL,
            extension     TEXT,
            size          INTEGER NOT NULL,
            modified_time TEXT NOT NULL,
            indexed_time  TEXT NOT NULL,
            content_hash  TEXT
        );

        CREATE TABLE IF NOT EXISTS file_chunks (
            id         INTEGER PRIMARY KEY,
            file_id    INTEGER NOT NULL REFERENCES indexed_files(id) ON DELETE CASCADE,
            chunk_idx  INTEGER NOT NULL,
            content    TEXT NOT NULL,
            byte_start INTEGER,
            byte_end   INTEGER
        );

        CREATE TABLE IF NOT EXISTS vector_index (
            id        INTEGER PRIMARY KEY,
            chunk_id  INTEGER NOT NULL REFERENCES file_chunks(id) ON DELETE CASCADE,
            embedding BLOB NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_files_root ON indexed_files(root_id);
        CREATE INDEX IF NOT EXISTS idx_files_ext ON indexed_files(extension);
        CREATE INDEX IF NOT EXISTS idx_files_path ON indexed_files(path);
        CREATE INDEX IF NOT EXISTS idx_chunks_file ON file_chunks(file_id);
        CREATE INDEX IF NOT EXISTS idx_vectors_chunk ON vector_index(chunk_id);
        ",
    )?;

    // v0.2: add enabled column to indexed_roots
    let has_enabled: bool = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('indexed_roots') WHERE name='enabled'")?
        .query_row([], |row| row.get::<_, i64>(0))
        .unwrap_or(0)
        > 0;
    if !has_enabled {
        conn.execute_batch("ALTER TABLE indexed_roots ADD COLUMN enabled INTEGER NOT NULL DEFAULT 1;")?;
    }

    // v0.3: image_vector_index for CLIP embeddings
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS image_vector_index (
            id        INTEGER PRIMARY KEY,
            file_id   INTEGER NOT NULL REFERENCES indexed_files(id) ON DELETE CASCADE,
            embedding BLOB NOT NULL,
            UNIQUE(file_id)
        );
        CREATE INDEX IF NOT EXISTS idx_img_vec_file ON image_vector_index(file_id);",
    )?;

    // v0.4: add code search columns to file_chunks
    let has_line_start: bool = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('file_chunks') WHERE name='line_start'")?
        .query_row([], |row| row.get::<_, i64>(0))
        .unwrap_or(0)
        > 0;
    if !has_line_start {
        conn.execute_batch(
            "ALTER TABLE file_chunks ADD COLUMN line_start INTEGER;
             ALTER TABLE file_chunks ADD COLUMN line_end INTEGER;
             ALTER TABLE file_chunks ADD COLUMN symbol_name TEXT;",
        )?;
    }

    // FTS5 tables need separate handling since CREATE VIRTUAL TABLE IF NOT EXISTS
    // is supported in newer SQLite versions
    let has_chunks_fts: bool = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='file_chunks_fts'")?
        .exists([])?;
    if !has_chunks_fts {
        conn.execute_batch(
            "
            CREATE VIRTUAL TABLE file_chunks_fts USING fts5(
                content,
                content='file_chunks',
                content_rowid='id'
            );
            ",
        )?;
    }

    let has_files_fts: bool = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='files_fts'")?
        .exists([])?;
    if !has_files_fts {
        conn.execute_batch(
            "
            CREATE VIRTUAL TABLE files_fts USING fts5(
                filename,
                path,
                content='indexed_files',
                content_rowid='id'
            );
            ",
        )?;
    }

    // FTS triggers for automatic sync
    conn.execute_batch(
        "
        CREATE TRIGGER IF NOT EXISTS chunks_ai AFTER INSERT ON file_chunks BEGIN
            INSERT INTO file_chunks_fts(rowid, content) VALUES (new.id, new.content);
        END;
        CREATE TRIGGER IF NOT EXISTS chunks_ad AFTER DELETE ON file_chunks BEGIN
            INSERT INTO file_chunks_fts(file_chunks_fts, rowid, content) VALUES('delete', old.id, old.content);
        END;
        CREATE TRIGGER IF NOT EXISTS chunks_au AFTER UPDATE ON file_chunks BEGIN
            INSERT INTO file_chunks_fts(file_chunks_fts, rowid, content) VALUES('delete', old.id, old.content);
            INSERT INTO file_chunks_fts(rowid, content) VALUES (new.id, new.content);
        END;

        CREATE TRIGGER IF NOT EXISTS files_ai AFTER INSERT ON indexed_files BEGIN
            INSERT INTO files_fts(rowid, filename, path) VALUES (new.id, new.filename, new.path);
        END;
        CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON indexed_files BEGIN
            INSERT INTO files_fts(files_fts, rowid, filename, path) VALUES('delete', old.id, old.filename, old.path);
        END;
        CREATE TRIGGER IF NOT EXISTS files_au AFTER UPDATE ON indexed_files BEGIN
            INSERT INTO files_fts(files_fts, rowid, filename, path) VALUES('delete', old.id, old.filename, old.path);
            INSERT INTO files_fts(rowid, filename, path) VALUES (new.id, new.filename, new.path);
        END;
        ",
    )?;

    Ok(())
}
