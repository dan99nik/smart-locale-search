use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use super::config;

pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
}

fn should_enter(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    if entry.depth() > 0 && name.starts_with('.') {
        return false;
    }
    if config::SYSTEM_DIRS.iter().any(|skip| name.eq_ignore_ascii_case(skip)) {
        return false;
    }
    true
}

fn map_entry(result: Result<DirEntry, walkdir::Error>) -> Option<FileEntry> {
    let entry = result.ok()?;
    if !entry.file_type().is_file() {
        return None;
    }
    let meta = entry.metadata().ok()?;
    let size = meta.len();
    if size > config::MAX_FILE_SIZE {
        return None;
    }
    let ext = entry
        .path()
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if config::should_skip_extension(ext) {
        return None;
    }
    Some(FileEntry {
        path: entry.into_path(),
        size,
    })
}

pub fn scan_directory_iter(root: &Path) -> impl Iterator<Item = FileEntry> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(should_enter as fn(&DirEntry) -> bool)
        .filter_map(map_entry as fn(Result<DirEntry, walkdir::Error>) -> Option<FileEntry>)
}

pub fn count_files(root: &Path) -> usize {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(should_enter as fn(&DirEntry) -> bool)
        .filter_map(|e| e.ok())
        .filter(|e| {
            if !e.file_type().is_file() {
                return false;
            }
            let size = e.metadata().map(|m| m.len()).unwrap_or(0);
            if size > config::MAX_FILE_SIZE {
                return false;
            }
            let ext = e.path().extension().and_then(|e| e.to_str()).unwrap_or("");
            !config::should_skip_extension(ext)
        })
        .count()
}
