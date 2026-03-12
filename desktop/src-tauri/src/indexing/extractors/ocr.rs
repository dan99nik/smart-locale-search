use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

use crate::indexing::config;

static TESSERACT_AVAILABLE: OnceLock<bool> = OnceLock::new();

fn is_tesseract_available() -> bool {
    *TESSERACT_AVAILABLE.get_or_init(|| {
        Command::new("tesseract")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

pub fn extract(path: &Path) -> Option<String> {
    if !is_tesseract_available() {
        log::debug!("Tesseract not available, skipping OCR for {:?}", path);
        return fallback_metadata(path);
    }

    let file_size = std::fs::metadata(path).ok()?.len();
    if file_size > config::OCR_MAX_IMAGE_SIZE {
        log::debug!("Image too large for OCR ({} bytes): {:?}", file_size, path);
        return fallback_metadata(path);
    }

    let prepared_path = prepare_image(path)?;
    let ocr_path = prepared_path.as_deref().unwrap_or(path);

    let raw_text = run_tesseract(ocr_path);

    if let Some(ref tmp) = prepared_path {
        let _ = std::fs::remove_file(tmp);
    }

    let text = match raw_text {
        Some(t) => {
            let cleaned = clean_ocr_text(&t);
            if cleaned.len() < config::OCR_MIN_TEXT_CHARS {
                fallback_metadata(path)
            } else {
                let meta = fallback_metadata(path).unwrap_or_default();
                Some(format!("{}\n{}", meta, cleaned))
            }
        }
        None => fallback_metadata(path),
    };

    text
}

fn prepare_image(path: &Path) -> Option<Option<std::path::PathBuf>> {
    let img = match image::open(path) {
        Ok(img) => img,
        Err(e) => {
            log::debug!("Failed to open image {:?}: {}", path, e);
            return Some(None);
        }
    };

    let (w, h) = (img.width(), img.height());
    let max_dim = config::OCR_MAX_DIMENSION;

    if w <= max_dim && h <= max_dim {
        return Some(None);
    }

    let resized = img.thumbnail(max_dim, max_dim);

    let tmp_dir = std::env::temp_dir();
    let tmp_name = format!("sls_ocr_{}.png", std::process::id());
    let tmp_path = tmp_dir.join(tmp_name);

    match resized.save(&tmp_path) {
        Ok(_) => Some(Some(tmp_path)),
        Err(e) => {
            log::debug!("Failed to save resized image: {}", e);
            Some(None)
        }
    }
}

fn run_tesseract(image_path: &Path) -> Option<String> {
    let tmp_dir = std::env::temp_dir();
    let out_base = tmp_dir.join(format!("sls_ocr_out_{}", std::process::id()));

    let result = Command::new("tesseract")
        .arg(image_path.to_string_lossy().as_ref())
        .arg(out_base.to_string_lossy().as_ref())
        .arg("-l").arg("eng+rus")
        .arg("--psm").arg("3")
        .arg("--oem").arg("3")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    let output_file = out_base.with_extension("txt");

    let text = match result {
        Ok(status) if status.success() => {
            std::fs::read_to_string(&output_file).ok()
        }
        _ => None,
    };

    let _ = std::fs::remove_file(&output_file);

    text
}

fn clean_ocr_text(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len().min(config::OCR_MAX_TEXT_CHARS));

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !result.ends_with('\n') {
                result.push('\n');
            }
            continue;
        }

        let tokens: Vec<&str> = trimmed.split_whitespace()
            .filter(|t| t.len() >= 2 || t.chars().all(|c| c.is_alphanumeric()))
            .filter(|t| !is_garbage_token(t))
            .collect();

        if tokens.is_empty() {
            continue;
        }

        let line_text = tokens.join(" ");
        if result.len() + line_text.len() + 1 > config::OCR_MAX_TEXT_CHARS {
            let remaining = config::OCR_MAX_TEXT_CHARS.saturating_sub(result.len());
            if remaining > 10 {
                let safe_end = line_text.char_indices()
                    .take_while(|&(i, _)| i < remaining)
                    .last()
                    .map(|(i, c)| i + c.len_utf8())
                    .unwrap_or(0);
                result.push_str(&line_text[..safe_end]);
            }
            break;
        }

        result.push_str(&line_text);
        result.push('\n');
    }

    result.trim().to_string()
}

fn is_garbage_token(token: &str) -> bool {
    if token.len() > 50 {
        return true;
    }
    let alpha_count = token.chars().filter(|c| c.is_alphabetic()).count();
    let total = token.chars().count();
    if total > 3 && alpha_count == 0 {
        let digit_count = token.chars().filter(|c| c.is_ascii_digit()).count();
        if digit_count == 0 {
            return true;
        }
    }
    let special = token.chars().filter(|c| !c.is_alphanumeric() && !c.is_whitespace()).count();
    if total > 4 && special as f64 / total as f64 > 0.6 {
        return true;
    }
    false
}

fn fallback_metadata(path: &Path) -> Option<String> {
    let filename = path.file_name()?.to_string_lossy().to_string();
    let parent = path.parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    Some(format!("{} {}", filename, parent))
}

pub fn check_tesseract() -> bool {
    is_tesseract_available()
}
