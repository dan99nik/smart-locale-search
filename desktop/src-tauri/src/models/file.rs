use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedRoot {
    pub id: i64,
    pub path: String,
    pub added_time: String,
    pub last_scan: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFile {
    pub id: i64,
    pub root_id: i64,
    pub path: String,
    pub filename: String,
    pub extension: Option<String>,
    pub size: i64,
    pub modified_time: String,
    pub indexed_time: String,
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    pub id: i64,
    pub file_id: i64,
    pub chunk_idx: i32,
    pub content: String,
    pub byte_start: Option<i64>,
    pub byte_end: Option<i64>,
    pub line_start: Option<i64>,
    pub line_end: Option<i64>,
    pub symbol_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_files: i64,
    pub total_chunks: i64,
    pub total_vectors: i64,
    pub total_roots: i64,
    pub db_size_bytes: i64,
}

pub const TEXT_EXTENSIONS: &[&str] = &["txt", "md"];
pub const DOCUMENT_EXTENSIONS: &[&str] = &["pdf", "docx", "csv", "xlsx", "xls"];
pub const CODE_EXTENSIONS: &[&str] = &[
    "js", "ts", "py", "cs", "cpp", "json", "html", "css", "rs", "go", "java", "rb", "sh",
    "yaml", "yml", "toml", "xml", "sql", "jsx", "tsx", "c", "h", "hpp",
];
pub const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];

pub fn file_category(ext: &str) -> FileCategory {
    let ext_lower = ext.to_lowercase();
    let ext_ref = ext_lower.as_str();
    if TEXT_EXTENSIONS.contains(&ext_ref) {
        FileCategory::Text
    } else if DOCUMENT_EXTENSIONS.contains(&ext_ref) {
        FileCategory::Document
    } else if CODE_EXTENSIONS.contains(&ext_ref) {
        FileCategory::Code
    } else if IMAGE_EXTENSIONS.contains(&ext_ref) {
        FileCategory::Image
    } else {
        FileCategory::Other
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileCategory {
    Text,
    Document,
    Code,
    Image,
    Other,
}
