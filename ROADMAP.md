# Roadmap

## v0.1 - Foundation

- [x] Project skeleton (Tauri + React + TypeScript + Tailwind)
- [x] SQLite database with FTS5
- [x] File indexing engine with recursive directory scanning
- [x] Text extraction (txt, md, pdf, docx, csv, code files)
- [x] Filename search
- [x] Content search (full-text via FTS5)

## v0.2 - Search Quality

- [x] Fuzzy search
- [x] Snippet previews with keyword highlighting
- [x] Combined ranking (text match + recency)
- [ ] Search result pagination
- [ ] Incremental indexing (watch for file changes)

## v0.3 - Semantic Search

- [x] Embedding provider abstraction
- [x] multilingual-e5-small ONNX integration
- [x] Vector storage in SQLite
- [x] Cosine similarity search
- [x] Combined ranking (text + semantic + recency)
- [ ] Model download UI
- [ ] Batch embedding during indexing

## v0.4 - Image & Media Search

- [ ] EXIF metadata extraction
- [ ] Image thumbnail preview
- [ ] Audio file metadata (ID3 tags)
- [ ] Video file metadata

## v0.5 - Performance & Polish

- [ ] HNSW vector index for faster semantic search
- [ ] Background indexing with progress bar
- [ ] File watcher for real-time index updates
- [ ] Exclude patterns configuration UI
- [ ] Search history

## v0.6 - Advanced Features

- [ ] Natural language query understanding
- [ ] Search filters (by type, date, size)
- [ ] Saved searches / bookmarks
- [ ] Keyboard shortcuts customization
- [ ] Multi-language UI

## v0.7 - Platform Polish

- [ ] Windows installer (NSIS)
- [ ] macOS DMG with code signing
- [ ] Auto-update via GitHub Releases
- [ ] System tray integration
- [ ] Global hotkey to invoke search

## Future

- [ ] Linux support (.deb, .AppImage)
- [ ] OCR for scanned PDFs and images
- [ ] Email search (local mailbox files)
- [ ] Browser bookmark search
- [ ] Plugin system for custom extractors
- [ ] Larger embedding models (e5-base, e5-large)
