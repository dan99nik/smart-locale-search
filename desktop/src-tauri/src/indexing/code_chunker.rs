use regex::Regex;

pub struct CodeChunk {
    pub content: String,
    pub line_start: usize,
    pub line_end: usize,
    pub symbol_name: Option<String>,
    pub byte_start: usize,
    pub byte_end: usize,
}

const MAX_CHUNK_LINES: usize = 500;
const FALLBACK_CHUNK_LINES: usize = 200;

pub fn chunk_code(text: &str, extension: &str) -> Vec<CodeChunk> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return vec![];
    }

    if lines.len() <= MAX_CHUNK_LINES {
        let symbol = detect_primary_symbol(&lines, extension);
        return vec![CodeChunk {
            content: text.to_string(),
            line_start: 1,
            line_end: lines.len(),
            symbol_name: symbol,
            byte_start: 0,
            byte_end: text.len(),
        }];
    }

    let boundaries = find_symbol_boundaries(&lines, extension);

    if boundaries.is_empty() {
        return chunk_by_lines(&lines, text);
    }

    merge_boundaries_into_chunks(&lines, text, &boundaries)
}

#[derive(Debug)]
struct SymbolBoundary {
    line: usize,
    name: String,
}

fn find_symbol_boundaries(lines: &[&str], ext: &str) -> Vec<SymbolBoundary> {
    let patterns = get_patterns_for_ext(ext);
    if patterns.is_empty() {
        return vec![];
    }

    let regexes: Vec<(Regex, usize)> = patterns
        .iter()
        .filter_map(|(pat, group)| Regex::new(pat).ok().map(|r| (r, *group)))
        .collect();

    let mut boundaries = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
            continue;
        }
        for (regex, group) in &regexes {
            if let Some(caps) = regex.captures(trimmed) {
                let name = caps.get(*group)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| trimmed.chars().take(60).collect());
                boundaries.push(SymbolBoundary { line: i, name });
                break;
            }
        }
    }
    boundaries
}

fn get_patterns_for_ext(ext: &str) -> Vec<(&'static str, usize)> {
    match ext {
        "rs" => vec![
            (r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)", 1),
            (r"^\s*(?:pub\s+)?struct\s+(\w+)", 1),
            (r"^\s*(?:pub\s+)?enum\s+(\w+)", 1),
            (r"^\s*(?:pub\s+)?trait\s+(\w+)", 1),
            (r"^\s*(?:pub\s+)?impl(?:<[^>]*>)?\s+(\w+)", 1),
            (r"^\s*(?:pub\s+)?mod\s+(\w+)", 1),
        ],
        "py" => vec![
            (r"^\s*def\s+(\w+)\s*\(", 1),
            (r"^\s*async\s+def\s+(\w+)\s*\(", 1),
            (r"^\s*class\s+(\w+)", 1),
        ],
        "js" | "jsx" | "ts" | "tsx" => vec![
            (r"^\s*(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s+(\w+)", 1),
            (r"^\s*(?:export\s+)?(?:default\s+)?class\s+(\w+)", 1),
            (r"^\s*(?:export\s+)?(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\(", 1),
            (r"^\s*(?:export\s+)?(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?(?:\([^)]*\)|[a-zA-Z_]\w*)\s*=>", 1),
            (r"^\s*(?:export\s+)?interface\s+(\w+)", 1),
            (r"^\s*(?:export\s+)?type\s+(\w+)\s*=", 1),
        ],
        "go" => vec![
            (r"^\s*func\s+(?:\([^)]+\)\s+)?(\w+)\s*\(", 1),
            (r"^\s*type\s+(\w+)\s+struct", 1),
            (r"^\s*type\s+(\w+)\s+interface", 1),
        ],
        "java" => vec![
            (r"^\s*(?:public|private|protected)?\s*(?:static\s+)?(?:final\s+)?class\s+(\w+)", 1),
            (r"^\s*(?:public|private|protected)?\s*(?:static\s+)?(?:final\s+)?interface\s+(\w+)", 1),
            (r"^\s*(?:public|private|protected)\s+(?:static\s+)?(?:\w+(?:<[^>]+>)?)\s+(\w+)\s*\(", 1),
        ],
        "cs" => vec![
            (r"^\s*(?:public|private|protected|internal)?\s*(?:static\s+)?(?:partial\s+)?class\s+(\w+)", 1),
            (r"^\s*(?:public|private|protected|internal)?\s*(?:static\s+)?interface\s+(\w+)", 1),
            (r"^\s*(?:public|private|protected|internal)\s+(?:static\s+)?(?:async\s+)?(?:virtual\s+)?(?:override\s+)?(?:\w+(?:<[^>]+>)?)\s+(\w+)\s*\(", 1),
        ],
        "c" | "h" => vec![
            (r"^\s*(?:static\s+)?(?:inline\s+)?(?:const\s+)?(?:unsigned\s+)?(?:signed\s+)?(?:void|int|char|float|double|long|short|size_t|bool|\w+_t|\w+\s*\*)\s+(\w+)\s*\(", 1),
            (r"^\s*typedef\s+struct\s+(\w+)", 1),
            (r"^\s*struct\s+(\w+)\s*\{", 1),
        ],
        "cpp" | "hpp" => vec![
            (r"^\s*(?:virtual\s+)?(?:static\s+)?(?:inline\s+)?(?:const\s+)?(?:void|int|char|float|double|long|short|bool|auto|string|std::\w+|\w+)\s+(\w+)\s*\(", 1),
            (r"^\s*class\s+(\w+)", 1),
            (r"^\s*struct\s+(\w+)", 1),
            (r"^\s*namespace\s+(\w+)", 1),
            (r"^\s*template\s*<[^>]*>\s*(?:class|struct)\s+(\w+)", 1),
        ],
        _ => vec![],
    }
}

fn merge_boundaries_into_chunks(
    lines: &[&str],
    text: &str,
    boundaries: &[SymbolBoundary],
) -> Vec<CodeChunk> {
    let mut chunks = Vec::new();
    let line_offsets = compute_line_offsets(text);

    for (i, boundary) in boundaries.iter().enumerate() {
        let start_line = boundary.line;
        let end_line = if i + 1 < boundaries.len() {
            (boundaries[i + 1].line).min(start_line + MAX_CHUNK_LINES)
        } else {
            lines.len().min(start_line + MAX_CHUNK_LINES)
        };

        if start_line >= end_line {
            continue;
        }

        let byte_start = line_offsets[start_line];
        let byte_end = if end_line < line_offsets.len() {
            line_offsets[end_line]
        } else {
            text.len()
        };

        let content = &text[byte_start..byte_end];
        if content.trim().is_empty() {
            continue;
        }

        chunks.push(CodeChunk {
            content: content.to_string(),
            line_start: start_line + 1,
            line_end: end_line,
            symbol_name: Some(boundary.name.clone()),
            byte_start,
            byte_end,
        });
    }

    if chunks.is_empty() {
        return chunk_by_lines(lines, text);
    }

    // If the first symbol doesn't start at line 0, capture the preamble (imports, etc.)
    if boundaries[0].line > 0 {
        let preamble_end = boundaries[0].line;
        let cap = preamble_end.min(MAX_CHUNK_LINES);
        if cap > 0 {
            let byte_end = line_offsets[cap.min(line_offsets.len() - 1)];
            let content = &text[..byte_end];
            if !content.trim().is_empty() {
                chunks.insert(0, CodeChunk {
                    content: content.to_string(),
                    line_start: 1,
                    line_end: cap,
                    symbol_name: None,
                    byte_start: 0,
                    byte_end,
                });
            }
        }
    }

    chunks
}

fn chunk_by_lines(lines: &[&str], text: &str) -> Vec<CodeChunk> {
    let mut chunks = Vec::new();
    let line_offsets = compute_line_offsets(text);
    let mut start = 0;

    while start < lines.len() {
        let mut end = (start + FALLBACK_CHUNK_LINES).min(lines.len());

        // Try to break at a blank line
        if end < lines.len() {
            for probe in (start + FALLBACK_CHUNK_LINES / 2..end).rev() {
                if lines[probe].trim().is_empty() {
                    end = probe + 1;
                    break;
                }
            }
        }

        let byte_start = line_offsets[start];
        let byte_end = if end < line_offsets.len() {
            line_offsets[end]
        } else {
            text.len()
        };

        let content = &text[byte_start..byte_end];
        if !content.trim().is_empty() {
            chunks.push(CodeChunk {
                content: content.to_string(),
                line_start: start + 1,
                line_end: end,
                symbol_name: None,
                byte_start,
                byte_end,
            });
        }

        start = end;
    }

    chunks
}

fn compute_line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (i, ch) in text.char_indices() {
        if ch == '\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

fn detect_primary_symbol(lines: &[&str], ext: &str) -> Option<String> {
    let patterns = get_patterns_for_ext(ext);
    if patterns.is_empty() {
        return None;
    }

    let regexes: Vec<(Regex, usize)> = patterns
        .iter()
        .filter_map(|(pat, group)| Regex::new(pat).ok().map(|r| (r, *group)))
        .collect();

    for line in lines {
        let trimmed = line.trim();
        for (regex, group) in &regexes {
            if let Some(caps) = regex.captures(trimmed) {
                return caps.get(*group).map(|m| m.as_str().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_file_single_chunk() {
        let code = "fn main() {\n    println!(\"hi\");\n}\n";
        let chunks = chunk_code(code, "rs");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].symbol_name.as_deref(), Some("main"));
    }

    #[test]
    fn test_small_multi_fn_is_single_chunk() {
        let code = "use std::io;\n\npub fn hello() {\n    println!(\"hello\");\n}\n\npub fn world() {\n    println!(\"world\");\n}\n";
        let chunks = chunk_code(code, "rs");
        assert_eq!(chunks.len(), 1);
        // Primary symbol is the first one found
        assert!(chunks[0].symbol_name.is_some());
    }

    #[test]
    fn test_large_rust_file_splits() {
        let mut code = String::from("use std::io;\n\n");
        for i in 0..20 {
            code.push_str(&format!("pub fn func_{}() {{\n", i));
            for j in 0..30 {
                code.push_str(&format!("    let x{} = {};\n", j, j));
            }
            code.push_str("}\n\n");
        }
        let chunks = chunk_code(&code, "rs");
        assert!(chunks.len() >= 2, "Expected >=2 chunks, got {}", chunks.len());
        assert!(chunks.iter().any(|c| c.symbol_name.as_deref() == Some("func_0")));
        assert!(chunks.iter().any(|c| c.symbol_name.as_deref() == Some("func_10")));
    }

    #[test]
    fn test_large_python_file_splits() {
        let mut code = String::from("import os\n\n");
        for i in 0..20 {
            code.push_str(&format!("def func_{}(x):\n", i));
            for j in 0..30 {
                code.push_str(&format!("    y{} = x + {}\n", j, j));
            }
            code.push('\n');
        }
        let chunks = chunk_code(&code, "py");
        assert!(chunks.len() >= 2, "Expected >=2 chunks, got {}", chunks.len());
        assert!(chunks.iter().any(|c| c.symbol_name.as_deref() == Some("func_0")));
    }

    #[test]
    fn test_fallback_for_json() {
        let lines: Vec<String> = (0..600).map(|i| format!("  \"key_{}\": \"value_{}\",", i, i)).collect();
        let code = format!("{{\n{}\n}}", lines.join("\n"));
        let chunks = chunk_code(&code, "json");
        assert!(chunks.len() >= 2, "Expected >=2 chunks, got {}", chunks.len());
        for chunk in &chunks {
            assert!(chunk.line_end - chunk.line_start + 1 <= FALLBACK_CHUNK_LINES + 10);
        }
    }

    #[test]
    fn test_js_arrow_functions() {
        let mut code = String::from("import React from 'react';\n\n");
        for i in 0..20 {
            code.push_str(&format!("export const Component{} = () => {{\n", i));
            for j in 0..30 {
                code.push_str(&format!("  const val{} = {};\n", j, j));
            }
            code.push_str("  return null;\n};\n\n");
        }
        let chunks = chunk_code(&code, "tsx");
        assert!(chunks.len() >= 2, "Expected >=2 chunks, got {}", chunks.len());
        assert!(chunks.iter().any(|c| c.symbol_name.as_deref() == Some("Component0")));
    }

    #[test]
    fn test_line_numbers_correct() {
        let mut code = String::from("use std::io;\n\n");
        for i in 0..20 {
            code.push_str(&format!("pub fn func_{}() {{\n", i));
            for j in 0..30 {
                code.push_str(&format!("    let x{} = {};\n", j, j));
            }
            code.push_str("}\n\n");
        }
        let chunks = chunk_code(&code, "rs");
        for chunk in &chunks {
            assert!(chunk.line_start >= 1);
            assert!(chunk.line_end >= chunk.line_start);
            assert!(chunk.byte_end > chunk.byte_start);
        }
    }
}
