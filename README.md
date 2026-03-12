# Smart Local Search

> [🇺🇸 English](README.md) | [🇷🇺 Русский](README.ru.md)

A privacy-first local AI search engine for personal computers.
Find files by meaning, content, and context — all processing happens entirely on your machine.

## What It Does

Smart Local Search indexes your files and lets you find them using natural language queries:

- **"find the document where late fee is 0.5%"** — searches inside document contents
- **"find the spreadsheet where I calculated marketing budget"** — semantic meaning search
- **"find the code file where websocket reconnect logic exists"** — code-aware search
- **"find photos with text on them"** — OCR-based image text search
- **"find images of a sunset on the beach"** — AI-powered visual image search
- **"find the folder with design assets"** — folder and path search

## Privacy Guarantee

**All indexing and search happens locally. Your files never leave your computer.**

- 100% local processing — no cloud APIs, no backend servers
- Works fully offline after initial setup
- No user accounts, no telemetry, no tracking
- The architecture makes accidental data leaks impossible

Networking is only used optionally to download AI models from Hugging Face and to check for GitHub updates.

## Features

- **Multi-format indexing** — Text, PDF, DOCX, Excel (XLSX/XLS), CSV, and 20+ code languages
- **Full-text search** — SQLite FTS5 for instant keyword matching
- **Semantic search** — AI-powered meaning-based search using local embeddings (multilingual-e5-small)
- **Code search** — Language-aware chunking with function/class name detection and line-level navigation
- **OCR image search** — Extract and search text inside images using local Tesseract OCR
- **Visual image search** — Find images by describing what's in them using a local CLIP model
- **Fuzzy search** — Find files even with approximate or misspelled queries
- **Smart ranking** — Results ranked by text match + semantic similarity + recency
- **Highlighted snippets** — See matching text excerpts with query terms highlighted in context
- **Keyboard-driven** — Full keyboard navigation (arrows, Enter, Escape)
- **File actions** — Open file, open at line (code), reveal in folder
- **Incremental indexing** — Only re-indexes changed files via content hashing
- **In-app model download** — Download AI models directly from the Settings UI with progress tracking
- **Dark UI** — Clean, modern dark interface built with Tailwind CSS

## Supported File Types

| Category | Extensions |
|----------|-----------|
| Text | `.txt`, `.md` |
| Documents | `.pdf`, `.docx`, `.csv`, `.xlsx`, `.xls` |
| Code | `.js`, `.ts`, `.jsx`, `.tsx`, `.py`, `.rs`, `.go`, `.java`, `.cs`, `.cpp`, `.c`, `.h`, `.hpp`, `.rb`, `.sh`, `.json`, `.html`, `.css`, `.yaml`, `.yml`, `.toml`, `.xml`, `.sql` |
| Images | `.jpg`, `.jpeg`, `.png`, `.webp` (OCR text extraction + CLIP visual search) |

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Desktop framework | [Tauri 2](https://tauri.app/) |
| Frontend | React, TypeScript, Tailwind CSS, Vite |
| Backend | Rust |
| Database | SQLite with FTS5 |
| Text embeddings | [multilingual-e5-small](https://huggingface.co/intfloat/multilingual-e5-small) (ONNX, 384-dim) |
| Image embeddings | [CLIP ViT-B/32](https://huggingface.co/Marqo/onnx-open_clip-ViT-B-32) (ONNX, 512-dim) |
| OCR | [Tesseract](https://github.com/tesseract-ocr/tesseract) (system dependency) |

## Installation

### Download

Download the latest release from [GitHub Releases](https://github.com/nicholasgriffintn/smart-local-search/releases):

- **macOS**: `.dmg` installer (Apple Silicon)
- **Windows**: `.msi` or `.exe` installer

### First Launch

1. Open the app and go to **Settings**
2. Download the **multilingual-e5-small** model for semantic text search
3. *(Optional)* Download the **CLIP ViT-B/32** model for visual image search
4. Add folders you want to index
5. Start searching

All models are downloaded directly within the app — no terminal commands needed.

### Optional: Enable OCR Image Search

To search for text inside images, install Tesseract OCR:

```bash
# macOS
brew install tesseract

# Ubuntu / Debian
sudo apt install tesseract-ocr

# Windows — download installer from:
# https://github.com/UB-Mannheim/tesseract/wiki
```

Without Tesseract, the app works normally — OCR features are simply skipped.

## Build from Source

### Prerequisites

- [Rust](https://rustup.rs/) (1.77.2+)
- [Node.js](https://nodejs.org/) (18+)
- [Tauri prerequisites](https://tauri.app/start/prerequisites/)

### Build

```bash
git clone https://github.com/nicholasgriffintn/smart-local-search.git
cd smart-local-search/desktop

npm install

# Development mode (hot reload)
npm run tauri dev

# Production build
npm run tauri build
```

Build output:

- **macOS**: `src-tauri/target/release/bundle/dmg/`
- **Windows**: `src-tauri/target/release/bundle/nsis/`

## Project Structure

```
smart-local-search/
├── desktop/                  # Tauri desktop application
│   ├── src/                  # React frontend
│   │   ├── components/       # UI components
│   │   ├── hooks/            # React hooks
│   │   ├── lib/              # Tauri bridge, types
│   │   └── styles/           # Global styles
│   └── src-tauri/            # Rust backend
│       └── src/
│           ├── commands/     # Tauri IPC commands
│           ├── indexing/     # File indexing pipeline
│           ├── search/       # Search engines & ranker
│           ├── models/       # Data structures
│           ├── db/           # SQLite database & migrations
│           ├── embedding/    # Text & image embedding providers
│           └── model_manager/# Model download & management
├── docs/                     # Architecture documentation
├── ROADMAP.md
└── LICENSE (MIT)
```

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the full development plan.

## Contributing

Contributions are welcome! To get started:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes
4. Push to the branch
5. Open a Pull Request

For major changes, please open an issue first to discuss the approach.

### Development Setup

```bash
cd desktop
npm install
npm run tauri dev
```

The app opens in development mode with hot reload for the React frontend and automatic Rust recompilation.

## Support the Project

If you find Smart Local Search useful, consider supporting development:

- [Sponsor on GitHub](https://github.com/sponsors/nicholasgriffintn)
- [Support on Patreon](https://www.patreon.com/dan99nik)
- [Support on Boosty](https://boosty.to/dan99nik)
- **BTC**: `bc1qqm22w9le2cj5uv786g60p5dntg2w7rqqyrvhaf`
- **ETH**: `0x4E845B5Ca8C40972730b3545f98A3546193E9DE9`

## License

MIT License. See [LICENSE](LICENSE) for details.

---

**Smart Local Search** — Find your files by meaning. Privately.
