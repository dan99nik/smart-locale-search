use std::path::Path;

use crate::indexing::config::MAX_TEXT_EXTRACT_BYTES;

pub fn extract(path: &Path) -> Option<String> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(true)
        .from_path(path)
        .ok()?;

    let mut text = String::new();

    if let Ok(headers) = reader.headers() {
        text.push_str(&headers.iter().collect::<Vec<_>>().join(" | "));
        text.push('\n');
    }

    for result in reader.records() {
        if text.len() > MAX_TEXT_EXTRACT_BYTES {
            break;
        }
        if let Ok(record) = result {
            text.push_str(&record.iter().collect::<Vec<_>>().join(" | "));
            text.push('\n');
        }
    }

    if text.trim().is_empty() { None } else { Some(text) }
}
