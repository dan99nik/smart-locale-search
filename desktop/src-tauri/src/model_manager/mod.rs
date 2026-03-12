use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

// ── Model registry ──────────────────────────────────────────────────
// All model metadata lives here. Nothing is hard-coded in the UI.

pub struct ModelDef {
    pub id: &'static str,
    pub display_name: &'static str,
    pub version: &'static str,
    pub files: &'static [ModelFileDef],
}

pub struct ModelFileDef {
    pub filename: &'static str,
    pub url: &'static str,
    pub expected_size: Option<u64>,
}

pub static E5_SMALL: ModelDef = ModelDef {
    id: "multilingual-e5-small",
    display_name: "Multilingual E5 Small",
    version: "1.0",
    files: &[
        ModelFileDef {
            filename: "model.onnx",
            url: "https://huggingface.co/intfloat/multilingual-e5-small/resolve/main/onnx/model.onnx",
            expected_size: Some(134_000_000), // ~134 MB
        },
        ModelFileDef {
            filename: "tokenizer.json",
            url: "https://huggingface.co/intfloat/multilingual-e5-small/resolve/main/tokenizer.json",
            expected_size: None,
        },
    ],
};

pub static CLIP_VIT_B32: ModelDef = ModelDef {
    id: "open-clip-vit-b-32",
    display_name: "CLIP ViT-B/32 (Image Search)",
    version: "1.0",
    files: &[
        ModelFileDef {
            filename: "clip-visual.onnx",
            url: "https://huggingface.co/Marqo/onnx-open_clip-ViT-B-32/resolve/main/onnx16-open_clip-ViT-B-32-openai-visual.onnx",
            expected_size: Some(176_000_000), // ~176 MB
        },
        ModelFileDef {
            filename: "clip-textual.onnx",
            url: "https://huggingface.co/Marqo/onnx-open_clip-ViT-B-32/resolve/main/onnx16-open_clip-ViT-B-32-openai-textual.onnx",
            expected_size: Some(127_000_000), // ~127 MB
        },
    ],
};

pub fn get_all_models() -> Vec<&'static ModelDef> {
    vec![&E5_SMALL, &CLIP_VIT_B32]
}

pub fn get_model_by_id(id: &str) -> Option<&'static ModelDef> {
    get_all_models().into_iter().find(|m| m.id == id)
}

// ── Types shared with the frontend via serde ────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelStatus {
    NotInstalled,
    Downloading,
    Installed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub version: String,
    pub status: ModelStatus,
    pub model_path: String,
    pub files: Vec<ModelFileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFileInfo {
    pub filename: String,
    pub expected_size: Option<u64>,
    pub actual_size: Option<u64>,
    pub present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub file: String,
    pub file_index: usize,
    pub file_count: usize,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub percent: Option<f64>,
}

// ── Queries ─────────────────────────────────────────────────────────

pub fn model_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("models").join(E5_SMALL.id)
}

pub fn model_dir_for(app_data_dir: &Path, model_def: &ModelDef) -> PathBuf {
    app_data_dir.join("models").join(model_def.id)
}

pub fn get_model_info(app_data_dir: &Path) -> ModelInfo {
    get_model_info_for(app_data_dir, &E5_SMALL)
}

pub fn get_model_info_for(app_data_dir: &Path, model_def: &ModelDef) -> ModelInfo {
    let dir = model_dir_for(app_data_dir, model_def);
    let files: Vec<ModelFileInfo> = model_def
        .files
        .iter()
        .map(|f| {
            let path = dir.join(f.filename);
            let (present, actual_size) = if path.exists() {
                let size = std::fs::metadata(&path).map(|m| m.len()).ok();
                (true, size)
            } else {
                (false, None)
            };
            ModelFileInfo {
                filename: f.filename.to_string(),
                expected_size: f.expected_size,
                actual_size,
                present,
            }
        })
        .collect();

    let all_present = files.iter().all(|f| f.present);

    ModelInfo {
        id: model_def.id.to_string(),
        display_name: model_def.display_name.to_string(),
        version: model_def.version.to_string(),
        status: if all_present {
            ModelStatus::Installed
        } else {
            ModelStatus::NotInstalled
        },
        model_path: dir.to_string_lossy().to_string(),
        files,
    }
}

// ── Download ────────────────────────────────────────────────────────

pub async fn download_model(
    app_handle: AppHandle,
    app_data_dir: PathBuf,
) -> Result<ModelInfo, String> {
    download_model_by_def(app_handle, app_data_dir, &E5_SMALL).await
}

pub async fn download_model_by_def(
    app_handle: AppHandle,
    app_data_dir: PathBuf,
    model_def: &ModelDef,
) -> Result<ModelInfo, String> {
    let dir = model_dir_for(&app_data_dir, model_def);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create model directory: {}", e))?;

    let file_count = model_def.files.len();

    for (idx, file_def) in model_def.files.iter().enumerate() {
        let dest = dir.join(file_def.filename);

        // Skip files that already exist with a plausible size
        if dest.exists() {
            if let Some(expected) = file_def.expected_size {
                let actual = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
                // Within 5% tolerance — model hosting may add/strip headers
                if (actual as f64) > (expected as f64 * 0.90) {
                    log::info!("Skipping {}: already present ({} bytes)", file_def.filename, actual);
                    emit_progress(&app_handle, file_def.filename, idx, file_count, actual, Some(actual), Some(100.0));
                    continue;
                }
            } else if std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0) > 0 {
                log::info!("Skipping {}: already present", file_def.filename);
                let sz = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
                emit_progress(&app_handle, file_def.filename, idx, file_count, sz, Some(sz), Some(100.0));
                continue;
            }
        }

        download_file(&app_handle, file_def, &dir, idx, file_count).await?;
    }

    Ok(get_model_info_for(&app_data_dir, model_def))
}

async fn download_file(
    app_handle: &AppHandle,
    file_def: &ModelFileDef,
    dir: &Path,
    file_index: usize,
    file_count: usize,
) -> Result<(), String> {
    let temp_path = dir.join(format!("{}.download", file_def.filename));
    let final_path = dir.join(file_def.filename);

    log::info!("Downloading {} from {}", file_def.filename, file_def.url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .get(file_def.url)
        .send()
        .await
        .map_err(|e| {
            cleanup_temp(&temp_path);
            format!("Download request failed for {}: {}", file_def.filename, friendly_reqwest_error(&e))
        })?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed for {}: HTTP {}",
            file_def.filename,
            response.status()
        ));
    }

    let total_bytes = response.content_length();

    let mut file = std::fs::File::create(&temp_path).map_err(|e| {
        format!("Cannot create temp file {}: {}", temp_path.display(), e)
    })?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;

    let mut last_emit = std::time::Instant::now();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| {
            cleanup_temp(&temp_path);
            format!("Download interrupted for {}: {}", file_def.filename, e)
        })?;

        file.write_all(&chunk).map_err(|e| {
            cleanup_temp(&temp_path);
            format!("Write failed for {}: {}", file_def.filename, e)
        })?;

        downloaded += chunk.len() as u64;

        // Throttle progress events to ~10 per second
        if last_emit.elapsed() >= std::time::Duration::from_millis(100) {
            let percent = total_bytes.map(|t| (downloaded as f64 / t as f64) * 100.0);
            emit_progress(app_handle, file_def.filename, file_index, file_count, downloaded, total_bytes, percent);
            last_emit = std::time::Instant::now();
        }
    }

    file.flush().map_err(|e| {
        cleanup_temp(&temp_path);
        format!("Flush failed for {}: {}", file_def.filename, e)
    })?;
    drop(file);

    // Atomic-ish rename: temp → final
    std::fs::rename(&temp_path, &final_path).map_err(|e| {
        cleanup_temp(&temp_path);
        format!("Rename failed for {}: {}", file_def.filename, e)
    })?;

    // Final 100% event
    emit_progress(app_handle, file_def.filename, file_index, file_count, downloaded, Some(downloaded), Some(100.0));

    log::info!("Downloaded {} ({} bytes)", file_def.filename, downloaded);
    Ok(())
}

fn emit_progress(
    app_handle: &AppHandle,
    filename: &str,
    file_index: usize,
    file_count: usize,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    percent: Option<f64>,
) {
    let _ = app_handle.emit(
        "model-download-progress",
        DownloadProgress {
            file: filename.to_string(),
            file_index,
            file_count,
            downloaded_bytes,
            total_bytes,
            percent,
        },
    );
}

fn cleanup_temp(path: &Path) {
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}

fn friendly_reqwest_error(e: &reqwest::Error) -> String {
    if e.is_connect() {
        return "Could not connect. Check your internet connection.".to_string();
    }
    if e.is_timeout() {
        return "Connection timed out. Try again later.".to_string();
    }
    if e.is_request() {
        return "Invalid request. The download URL may have changed.".to_string();
    }
    e.to_string()
}
