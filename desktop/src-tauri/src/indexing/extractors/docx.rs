use std::path::Path;

use crate::indexing::config::MAX_TEXT_EXTRACT_BYTES;

pub fn extract(path: &Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    if meta.len() > MAX_TEXT_EXTRACT_BYTES as u64 * 5 {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let docx = docx_rs::read_docx(&bytes).ok()?;
    let mut text = String::new();
    for child in docx.document.children {
        if let docx_rs::DocumentChild::Paragraph(p) = child {
            for child in &p.children {
                if let docx_rs::ParagraphChild::Run(run) = child {
                    for child in &run.children {
                        if let docx_rs::RunChild::Text(t) = child {
                            text.push_str(&t.text);
                            if text.len() > MAX_TEXT_EXTRACT_BYTES {
                                return Some(text);
                            }
                        }
                    }
                }
            }
            text.push('\n');
        }
    }
    if text.trim().is_empty() { None } else { Some(text) }
}
