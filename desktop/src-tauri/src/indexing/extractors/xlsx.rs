use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};

use crate::indexing::config::MAX_TEXT_EXTRACT_BYTES;

pub fn extract(path: &Path) -> Option<String> {
    let mut workbook = open_workbook_auto(path).ok()?;
    let mut text = String::new();

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

    for name in &sheet_names {
        if let Ok(range) = workbook.worksheet_range(name) {
            for row in range.rows() {
                let cells: Vec<String> = row
                    .iter()
                    .filter_map(|cell| match cell {
                        Data::String(s) => Some(s.clone()),
                        Data::Float(f) => Some(f.to_string()),
                        Data::Int(i) => Some(i.to_string()),
                        Data::Bool(b) => Some(b.to_string()),
                        Data::DateTime(dt) => Some(dt.to_string()),
                        Data::DateTimeIso(s) => Some(s.clone()),
                        Data::DurationIso(s) => Some(s.clone()),
                        Data::Empty | Data::Error(_) => None,
                    })
                    .collect();

                if !cells.is_empty() {
                    text.push_str(&cells.join(" | "));
                    text.push('\n');
                }

                if text.len() > MAX_TEXT_EXTRACT_BYTES {
                    return Some(text);
                }
            }
        }
    }

    if text.trim().is_empty() { None } else { Some(text) }
}
