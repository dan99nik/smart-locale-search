use std::sync::Arc;

use crate::db::{repository, Database};
use crate::embedding::EmbeddingProvider;
use crate::models::search::SearchResult;

pub fn search(
    db: &Database,
    provider: &Arc<dyn EmbeddingProvider>,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    let query_embedding = provider.embed(query).map_err(|e| e.to_string())?;

    let conn = db.lock_conn();
    let all_vectors = repository::get_all_vectors(&conn).map_err(|e| e.to_string())?;

    if all_vectors.is_empty() {
        return Ok(vec![]);
    }

    let min_threshold = 0.45;

    let mut scored: Vec<(i64, f64)> = all_vectors
        .iter()
        .map(|(chunk_id, vec)| {
            let sim = cosine_similarity(&query_embedding, vec);
            (*chunk_id, sim)
        })
        .filter(|(_, sim)| *sim >= min_threshold)
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);

    let mut results = Vec::new();
    for (chunk_id, score) in &scored {
        if let Ok(mut result) = repository::get_file_for_chunk_with_query(&conn, *chunk_id, Some(query)) {
            result.score = *score;
            results.push(result);
        }
    }

    // Deduplicate by file path, keeping the highest score
    let mut seen = std::collections::HashSet::new();
    results.retain(|r| seen.insert(r.path.clone()));

    Ok(results)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0f64;
    let mut norm_a = 0.0f64;
    let mut norm_b = 0.0f64;
    for i in 0..a.len() {
        let ai = a[i] as f64;
        let bi = b[i] as f64;
        dot += ai * bi;
        norm_a += ai * ai;
        norm_b += bi * bi;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}
