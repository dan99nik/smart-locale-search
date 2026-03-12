use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub semantic_search_enabled: bool,
    pub max_file_size_mb: i64,
    pub excluded_patterns: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            semantic_search_enabled: true,
            max_file_size_mb: 50,
            excluded_patterns: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "__pycache__".to_string(),
                ".DS_Store".to_string(),
            ],
        }
    }
}
