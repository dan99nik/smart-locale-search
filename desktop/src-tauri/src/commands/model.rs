use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager};

use crate::model_manager::{self, ModelInfo};

static DOWNLOAD_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub async fn get_model_status(app_handle: AppHandle) -> Result<ModelInfo, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let mut info = model_manager::get_model_info(&app_data_dir);

    if DOWNLOAD_IN_PROGRESS.load(Ordering::SeqCst) && info.status == model_manager::ModelStatus::NotInstalled {
        info.status = model_manager::ModelStatus::Downloading;
    }

    Ok(info)
}

#[tauri::command]
pub async fn get_all_model_statuses(app_handle: AppHandle) -> Result<Vec<ModelInfo>, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let models = model_manager::get_all_models();
    let infos: Vec<ModelInfo> = models
        .iter()
        .map(|m| {
            let info = model_manager::get_model_info_for(&app_data_dir, m);
            if DOWNLOAD_IN_PROGRESS.load(Ordering::SeqCst)
                && info.status == model_manager::ModelStatus::NotInstalled
            {
                // Only mark as downloading if we can't tell which model is downloading
            }
            info
        })
        .collect();

    Ok(infos)
}

#[tauri::command]
pub async fn download_model(app_handle: AppHandle) -> Result<ModelInfo, String> {
    download_model_by_id(app_handle, "multilingual-e5-small".to_string()).await
}

#[tauri::command]
pub async fn download_model_by_id(app_handle: AppHandle, model_id: String) -> Result<ModelInfo, String> {
    if DOWNLOAD_IN_PROGRESS.swap(true, Ordering::SeqCst) {
        return Err("A download is already in progress.".to_string());
    }

    let model_def = model_manager::get_model_by_id(&model_id).ok_or_else(|| {
        DOWNLOAD_IN_PROGRESS.store(false, Ordering::SeqCst);
        format!("Unknown model: {}", model_id)
    })?;

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| {
            DOWNLOAD_IN_PROGRESS.store(false, Ordering::SeqCst);
            e.to_string()
        })?;

    let result = model_manager::download_model_by_def(app_handle.clone(), app_data_dir.clone(), model_def).await;

    DOWNLOAD_IN_PROGRESS.store(false, Ordering::SeqCst);

    match result {
        Ok(info) => {
            if info.status == model_manager::ModelStatus::Installed {
                hot_load_provider(&app_handle, &app_data_dir, &model_id);
            }
            Ok(info)
        }
        Err(e) => Err(e),
    }
}

fn hot_load_provider(app_handle: &AppHandle, app_data_dir: &std::path::Path, model_id: &str) {
    if let Some(state) = app_handle.try_state::<crate::AppState>() {
        match model_id {
            "multilingual-e5-small" => {
                let provider = crate::embedding::e5_small::create_provider(app_data_dir);
                if let Ok(mut guard) = state.embedding_provider.write() {
                    *guard = provider;
                    log::info!("E5 embedding provider hot-loaded after download");
                }
            }
            "open-clip-vit-b-32" => {
                let provider = crate::embedding::clip::create_provider(app_data_dir);
                if let Ok(mut guard) = state.vision_provider.write() {
                    *guard = provider;
                    log::info!("CLIP vision provider hot-loaded after download");
                }
            }
            _ => {}
        }
    }
}
