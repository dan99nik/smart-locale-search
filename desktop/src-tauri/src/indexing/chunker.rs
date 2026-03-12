const CHUNK_SIZE: usize = 1500;
const CHUNK_OVERLAP: usize = 200;

pub struct TextChunk {
    pub content: String,
    pub byte_start: usize,
    pub byte_end: usize,
}

fn ceil_char_boundary(text: &str, index: usize) -> usize {
    let mut i = index.min(text.len());
    while i < text.len() && !text.is_char_boundary(i) {
        i += 1;
    }
    i
}

fn floor_char_boundary(text: &str, index: usize) -> usize {
    let mut i = index.min(text.len());
    while i > 0 && !text.is_char_boundary(i) {
        i -= 1;
    }
    i
}

pub fn chunk_text(text: &str) -> Vec<TextChunk> {
    if text.len() <= CHUNK_SIZE {
        return vec![TextChunk {
            content: text.to_string(),
            byte_start: 0,
            byte_end: text.len(),
        }];
    }

    let mut chunks = Vec::new();
    let mut start = 0;

    while start < text.len() {
        let mut end = ceil_char_boundary(text, start + CHUNK_SIZE);

        if end < text.len() {
            if let Some(pos) = find_break_point(&text[start..end]) {
                end = start + pos;
            }
        }

        let chunk_str = &text[start..end];
        if !chunk_str.trim().is_empty() {
            chunks.push(TextChunk {
                content: chunk_str.to_string(),
                byte_start: start,
                byte_end: end,
            });
        }

        let advance = if end > start + CHUNK_OVERLAP {
            end - start - CHUNK_OVERLAP
        } else {
            end - start
        };
        start += advance.max(1);
        start = ceil_char_boundary(text, start);
    }

    chunks
}

fn find_break_point(text: &str) -> Option<usize> {
    if let Some(pos) = text.rfind("\n\n") {
        if pos > text.len() / 2 {
            return Some(pos + 2);
        }
    }
    if let Some(pos) = text.rfind('\n') {
        if pos > text.len() / 2 {
            return Some(pos + 1);
        }
    }
    for delim in &[". ", "! ", "? "] {
        if let Some(pos) = text.rfind(delim) {
            if pos > text.len() / 3 {
                return Some(pos + delim.len());
            }
        }
    }
    None
}
