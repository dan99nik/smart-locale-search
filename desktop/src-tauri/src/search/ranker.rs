use std::collections::HashMap;
use std::sync::Arc;

use crate::db::Database;
use crate::embedding::{EmbeddingProvider, ImageEmbeddingProvider};
use crate::models::search::SearchResult;
use super::{filename, fulltext, fuzzy, semantic, image_semantic};

const TEXT_WEIGHT: f64 = 0.35;
const SEMANTIC_WEIGHT: f64 = 0.55;
const RECENCY_WEIGHT: f64 = 0.1;

pub fn combined_search(
    db: &Database,
    embedding_provider: Option<&Arc<dyn EmbeddingProvider>>,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    combined_search_full(db, embedding_provider, None, query, limit)
}

pub fn combined_search_full(
    db: &Database,
    embedding_provider: Option<&Arc<dyn EmbeddingProvider>>,
    vision_provider: Option<&Arc<dyn ImageEmbeddingProvider>>,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    let search_limit = limit * 3;

    let filename_results = filename::search(db, query, search_limit).unwrap_or_default();
    let content_results = fulltext::search(db, query, search_limit).unwrap_or_default();
    let fuzzy_results = fuzzy::search(db, query, search_limit).unwrap_or_default();

    let semantic_results = if let Some(provider) = embedding_provider {
        semantic::search(db, provider, query, search_limit).unwrap_or_default()
    } else {
        vec![]
    };

    let image_results = if let Some(vp) = vision_provider {
        image_semantic::search(db, vp, query, search_limit).unwrap_or_default()
    } else {
        vec![]
    };

    // Merge results by file path
    let mut scored: HashMap<String, ScoredFile> = HashMap::new();

    for r in &filename_results {
        let entry = scored.entry(r.path.clone()).or_insert_with(|| ScoredFile::from(r));
        entry.text_score = entry.text_score.max(normalize_score(r.score, &filename_results) * 1.5);
    }

    for r in &content_results {
        let entry = scored.entry(r.path.clone()).or_insert_with(|| ScoredFile::from(r));
        entry.text_score = entry.text_score.max(normalize_score(r.score, &content_results));
        if r.snippet.is_some() {
            entry.result.snippet = r.snippet.clone();
        }
        if r.line_start.is_some() {
            entry.result.line_start = r.line_start;
            entry.result.line_end = r.line_end;
        }
        if r.symbol_name.is_some() {
            entry.result.symbol_name = r.symbol_name.clone();
        }
    }

    for r in &fuzzy_results {
        let entry = scored.entry(r.path.clone()).or_insert_with(|| ScoredFile::from(r));
        entry.text_score = entry.text_score.max(0.3);
    }

    let sem_top_score = semantic_results.first().map(|r| r.score).unwrap_or(0.0);
    let sem_cutoff = sem_top_score * 0.95;

    for r in semantic_results.iter().filter(|r| r.score >= sem_cutoff) {
        let entry = scored.entry(r.path.clone()).or_insert_with(|| ScoredFile::from(r));
        entry.semantic_score = r.score;
        if entry.result.snippet.is_none() && r.snippet.is_some() {
            entry.result.snippet = r.snippet.clone();
        }
        if entry.result.line_start.is_none() && r.line_start.is_some() {
            entry.result.line_start = r.line_start;
            entry.result.line_end = r.line_end;
        }
        if entry.result.symbol_name.is_none() && r.symbol_name.is_some() {
            entry.result.symbol_name = r.symbol_name.clone();
        }
    }

    for r in &image_results {
        let entry = scored.entry(r.path.clone()).or_insert_with(|| ScoredFile::from(r));
        entry.image_semantic_score = r.score;
        entry.is_image_match = true;
        if entry.result.snippet.is_none() && r.snippet.is_some() {
            entry.result.snippet = r.snippet.clone();
        }
    }

    // Compute final scores
    let mut results: Vec<SearchResult> = scored
        .into_values()
        .map(|sf| {
            let recency = compute_recency_score(&sf.result.modified_time);
            let image_boost = sf.image_semantic_score * SEMANTIC_WEIGHT;
            let final_score =
                TEXT_WEIGHT * sf.text_score
                + SEMANTIC_WEIGHT * sf.semantic_score
                + RECENCY_WEIGHT * recency
                + image_boost;
            let mut result = sf.result;
            result.score = final_score;
            result.match_type = determine_match_type_full(
                sf.text_score, sf.semantic_score, sf.image_semantic_score, sf.is_image_match,
            );
            result
        })
        .collect();

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);

    Ok(results)
}

struct ScoredFile {
    result: SearchResult,
    text_score: f64,
    semantic_score: f64,
    image_semantic_score: f64,
    is_image_match: bool,
}

impl ScoredFile {
    fn from(r: &SearchResult) -> Self {
        Self {
            result: r.clone(),
            text_score: 0.0,
            semantic_score: 0.0,
            image_semantic_score: 0.0,
            is_image_match: false,
        }
    }
}

fn normalize_score(score: f64, results: &[SearchResult]) -> f64 {
    if results.is_empty() {
        return 0.0;
    }
    let max = results
        .iter()
        .map(|r| r.score)
        .fold(0.0f64, |a, b| a.max(b));
    if max == 0.0 {
        0.0
    } else {
        (score / max).min(1.0)
    }
}

fn compute_recency_score(modified_time: &str) -> f64 {
    let modified = chrono::DateTime::parse_from_rfc3339(modified_time)
        .map(|dt| dt.timestamp())
        .unwrap_or(0);
    let now = chrono::Utc::now().timestamp();
    let age_days = ((now - modified) as f64) / 86400.0;

    (-age_days / 365.0).exp()
}

fn determine_match_type(text_score: f64, semantic_score: f64) -> String {
    determine_match_type_full(text_score, semantic_score, 0.0, false)
}

fn determine_match_type_full(
    text_score: f64,
    semantic_score: f64,
    image_semantic_score: f64,
    is_image_match: bool,
) -> String {
    if is_image_match && image_semantic_score > text_score && image_semantic_score > semantic_score {
        return "image_semantic".to_string();
    }
    if text_score > 0.5 && semantic_score > 0.5 {
        "exact+semantic".to_string()
    } else if text_score > semantic_score {
        "exact".to_string()
    } else if semantic_score > 0.0 {
        "semantic".to_string()
    } else {
        "fuzzy".to_string()
    }
}
