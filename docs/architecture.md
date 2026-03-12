# Architecture

## Overview

Smart Local Search is a desktop application built with Tauri (Rust backend + React frontend). All processing happens locally on the user's computer. No data is transmitted over the network.

## System Architecture

```
┌─────────────────────────────────────────────────┐
│                   Desktop App                    │
│  ┌─────────────────────────────────────────────┐ │
│  │           React UI (TypeScript)              │ │
│  │  Search Bar │ Results │ Settings │ Status    │ │
│  └───────────────────┬─────────────────────────┘ │
│                      │ Tauri IPC                  │
│  ┌───────────────────┴─────────────────────────┐ │
│  │             Rust Backend                     │ │
│  │  ┌──────────┐  ┌──────────┐  ┌───────────┐  │ │
│  │  │ Indexing  │  │  Search  │  │ Embedding │  │ │
│  │  │ Pipeline  │  │  Engine  │  │ Provider  │  │ │
│  │  └────┬─────┘  └────┬─────┘  └─────┬─────┘  │ │
│  │       │              │              │         │ │
│  │  ┌────┴──────────────┴──────────────┴─────┐  │ │
│  │  │           SQLite Database              │  │ │
│  │  │  FTS5 │ Metadata │ Chunks │ Vectors    │  │ │
│  │  └───────────────────────────────────────-┘  │ │
│  └──────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | Tauri 2 |
| Frontend | React 19, TypeScript, Tailwind CSS 4, Vite |
| Backend | Rust |
| Database | SQLite (via rusqlite, bundled) |
| Full-text search | SQLite FTS5 |
| Embeddings | ONNX Runtime (via ort crate) |
| Embedding model | multilingual-e5-small (384 dimensions) |
| Text extraction | pdf-extract, docx-rs, csv crate |

## Privacy Architecture

The application enforces privacy through architecture:

1. **No network access by default** - The Tauri CSP policy restricts all network requests
2. **No backend server** - Everything runs in-process
3. **No user accounts** - No authentication, no tracking
4. **Local database** - SQLite file stored in the app data directory
5. **Local models** - Embedding models run via ONNX Runtime on CPU

The only optional network usage is for checking GitHub updates.

## Indexing Pipeline

```
User selects folder
       │
       ▼
Directory Scanner (walkdir)
  - Recursive traversal
  - Skips hidden/system directories
  - Respects exclude patterns
       │
       ▼
File Type Detector
  - Text: txt, md
  - Documents: pdf, docx, csv
  - Code: js, ts, py, rs, etc.
  - Images: jpg, png, webp
       │
       ▼
Text Extractor (per format)
  - Plain text: direct read
  - PDF: pdf-extract
  - DOCX: docx-rs
  - CSV: csv crate
  - Images: filename + path
       │
       ▼
Content Hasher (SHA-256)
  - Skip re-indexing unchanged files
       │
       ├─────────────────────┐
       ▼                     ▼
Text Chunker            SQLite FTS5
  - ~1500 char chunks      - Full-text index
  - 200 char overlap       - Automatic triggers
  - Sentence-boundary
    aware
       │
       ▼
Embedding Generator (optional)
  - multilingual-e5-small
  - 384-dim vectors
  - Mean pooling + L2 norm
       │
       ▼
Vector Storage (SQLite BLOB)
```

## Search Architecture

The search engine runs four parallel search paths and merges results:

1. **Filename search** - FTS5 on `files_fts` (filename + path)
2. **Content search** - FTS5 on `file_chunks_fts` (chunk content)
3. **Fuzzy search** - SQL LIKE on filename and path
4. **Semantic search** - Cosine similarity on embedding vectors

### Ranking Formula

```
final_score = 0.3 * text_match + 0.5 * semantic_similarity + 0.2 * recency_score
```

- `text_match`: Normalized FTS5 rank [0,1], with filename match bonus (1.5x)
- `semantic_similarity`: Cosine similarity [0,1]
- `recency_score`: Exponential decay `e^(-age_days / 365)`

Results are deduplicated by file path and sorted by final score.

## Database Schema

See `desktop/src-tauri/src/db/migrations.rs` for the full schema. Key tables:

- `indexed_roots` - User-selected folders
- `indexed_files` - File metadata (path, size, modified time, content hash)
- `file_chunks` - Text chunks for FTS and embedding
- `file_chunks_fts` - FTS5 virtual table for content search
- `files_fts` - FTS5 virtual table for filename search
- `vector_index` - Embedding vectors as BLOBs
- `settings` - Key-value app settings

## Embedding Provider

The `EmbeddingProvider` trait abstracts the embedding model:

```rust
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError>;
    fn dimension(&self) -> usize;
    fn model_name(&self) -> &str;
}
```

The default implementation uses `multilingual-e5-small` via ONNX Runtime. If the model files are not present, semantic search is gracefully disabled and the app falls back to text-only search.

## Frontend Architecture

The React UI uses a simple component hierarchy:

```
App
├── Layout
│   ├── SearchBar (search input + settings button)
│   ├── ResultList
│   │   └── ResultItem (file icon, name, path, snippet, actions)
│   │       ├── FileIcon
│   │       └── SnippetPreview
│   └── IndexStatus (footer stats bar)
└── Settings (folder management, stats, about)
```

State is managed via custom hooks:
- `useSearch` - Search query, debouncing, results
- `useSettings` - Indexed folders, stats, reindex
- `useKeyboard` - Arrow key navigation, Enter to open

Communication with the Rust backend happens via Tauri IPC (`invoke`).
