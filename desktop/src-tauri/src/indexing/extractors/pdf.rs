use std::path::Path;

use crate::indexing::config::MAX_TEXT_EXTRACT_BYTES;

pub fn extract(path: &Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    if meta.len() > MAX_TEXT_EXTRACT_BYTES as u64 * 5 {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let text = pdf_extract::extract_text_from_mem(&bytes).ok()?;
    if text.len() > MAX_TEXT_EXTRACT_BYTES {
        Some(text[..MAX_TEXT_EXTRACT_BYTES].to_string())
    } else if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}
