use std::path::Path;

use crate::models::file::{file_category, FileCategory};
use super::extractors::{plaintext, pdf, docx, csv_ext, xlsx, ocr};

pub fn extract_text(path: &Path) -> Option<String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let category = file_category(&ext);
    match category {
        FileCategory::Text | FileCategory::Code => plaintext::extract(path),
        FileCategory::Document => match ext.as_str() {
            "pdf" => pdf::extract(path),
            "docx" => docx::extract(path),
            "csv" => csv_ext::extract(path),
            "xlsx" | "xls" => xlsx::extract(path),
            _ => None,
        },
        FileCategory::Image => ocr::extract(path),
        FileCategory::Other => None,
    }
}
