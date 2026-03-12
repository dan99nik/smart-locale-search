use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_id: i64,
    pub path: String,
    pub filename: String,
    pub extension: Option<String>,
    pub size: i64,
    pub modified_time: String,
    pub snippet: Option<String>,
    pub score: f64,
    pub match_type: String,
    pub line_start: Option<i64>,
    pub line_end: Option<i64>,
    pub symbol_name: Option<String>,
}
