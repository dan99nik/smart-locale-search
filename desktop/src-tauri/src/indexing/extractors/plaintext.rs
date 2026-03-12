use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::indexing::config::MAX_TEXT_EXTRACT_BYTES;

pub fn extract(path: &Path) -> Option<String> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut text = String::new();

    for line in reader.lines() {
        match line {
            Ok(l) => {
                if text.len() + l.len() + 1 > MAX_TEXT_EXTRACT_BYTES {
                    let remaining = MAX_TEXT_EXTRACT_BYTES.saturating_sub(text.len());
                    if remaining > 0 {
                        let safe_end = l.char_indices()
                            .take_while(|&(i, _)| i < remaining)
                            .last()
                            .map(|(i, c)| i + c.len_utf8())
                            .unwrap_or(0);
                        text.push_str(&l[..safe_end]);
                    }
                    break;
                }
                text.push_str(&l);
                text.push('\n');
            }
            Err(_) => break,
        }
    }

    if text.trim().is_empty() { None } else { Some(text) }
}
