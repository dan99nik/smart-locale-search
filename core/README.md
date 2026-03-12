# Core

This directory is reserved for the shared Rust crate that will be extracted from the Tauri backend in a future version.

The core search engine, indexing pipeline, and embedding provider will be refactored into a standalone library crate, enabling:

- CLI search tool
- Integration into other applications
- Headless indexing server
- Library usage from other Rust projects
