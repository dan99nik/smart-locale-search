use std::path::Path;
use tauri::State;

use crate::indexing::extractors::ocr;
use crate::models::search::SearchResult;
use crate::search::ranker;
use crate::AppState;

#[tauri::command]
pub async fn search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let db = &state.db;
    let embed_guard = state.embedding_provider.read().map_err(|e| e.to_string())?;
    let vision_guard = state.vision_provider.read().map_err(|e| e.to_string())?;
    let provider = embed_guard.as_ref();
    let vision_provider = vision_guard.as_ref();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ranker::combined_search_full(db, provider, vision_provider, &query, 50)
    }));

    match result {
        Ok(inner) => inner,
        Err(_) => Err("Search panicked internally".to_string()),
    }
}

#[tauri::command]
pub async fn get_image_thumbnail(path: String) -> Result<String, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err("File not found".to_string());
    }

    let img = image::open(p).map_err(|e| format!("Failed to open image: {}", e))?;
    let thumb = img.thumbnail(120, 120);

    let mut buf = std::io::Cursor::new(Vec::new());
    thumb.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode thumbnail: {}", e))?;

    let b64 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        buf.into_inner(),
    );

    Ok(format!("data:image/png;base64,{}", b64))
}

#[tauri::command]
pub async fn check_ocr_available() -> Result<bool, String> {
    Ok(ocr::check_tesseract())
}
