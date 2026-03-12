mod commands;
mod db;
mod embedding;
mod indexing;
mod model_manager;
mod models;
mod search;

use std::sync::atomic::AtomicU8;
use std::sync::{Arc, RwLock};

use tauri::Manager;

use embedding::{EmbeddingProvider, ImageEmbeddingProvider};

/// 0 = priority, 1 = idle-only
pub static INDEXING_MODE: AtomicU8 = AtomicU8::new(0);

pub struct AppState {
    pub db: db::Database,
    pub embedding_provider: RwLock<Option<Arc<dyn EmbeddingProvider>>>,
    pub vision_provider: RwLock<Option<Arc<dyn ImageEmbeddingProvider>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data dir");

            let db_path = app_data_dir.join("smart-local-search.db");
            let db = db::Database::new(&db_path).expect("Failed to initialize database");

            let embedding_provider = embedding::e5_small::create_provider(&app_data_dir);
            let vision_provider = embedding::clip::create_provider(&app_data_dir);

            app.manage(AppState {
                db,
                embedding_provider: RwLock::new(embedding_provider),
                vision_provider: RwLock::new(vision_provider),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search::search,
            commands::search::get_image_thumbnail,
            commands::search::check_ocr_available,
            commands::indexing::add_folder,
            commands::indexing::remove_folder,
            commands::indexing::get_indexed_roots,
            commands::indexing::get_index_stats,
            commands::indexing::set_root_enabled,
            commands::indexing::get_default_roots,
            commands::indexing::init_default_roots,
            commands::indexing::reindex,
            commands::indexing::cancel_indexing,
            commands::indexing::set_indexing_mode,
            commands::indexing::get_indexing_mode,
            commands::settings::open_file,
            commands::settings::open_file_at_line,
            commands::settings::reveal_in_folder,
            commands::settings::get_app_version,
            commands::model::get_model_status,
            commands::model::download_model,
            commands::model::get_all_model_statuses,
            commands::model::download_model_by_id,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
