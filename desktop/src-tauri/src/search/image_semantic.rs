use std::sync::Arc;

use crate::db::{repository, Database};
use crate::embedding::ImageEmbeddingProvider;
use crate::models::search::SearchResult;

pub fn search(
    db: &Database,
    provider: &Arc<dyn ImageEmbeddingProvider>,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    let query_embedding = provider.embed_text(query).map_err(|e| e.to_string())?;

    let conn = db.lock_conn();
    let all_vectors = repository::get_all_image_vectors(&conn).map_err(|e| e.to_string())?;

    if all_vectors.is_empty() {
        return Ok(vec![]);
    }

    let mut scored: Vec<(i64, f64)> = all_vectors
        .iter()
        .map(|(file_id, vec)| {
            let sim = cosine_similarity(&query_embedding, vec);
            (*file_id, sim)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit * 2);

    let threshold = 0.15;

    let mut results = Vec::new();
    for (file_id, score) in &scored {
        if *score < threshold {
            continue;
        }
        if let Ok(mut result) = repository::get_file_by_id(&conn, *file_id) {
            result.score = *score;
            result.match_type = "image_semantic".to_string();
            result.snippet = Some(format!("Visual similarity: {:.0}%", score * 100.0));
            results.push(result);
        }
    }

    results.truncate(limit);
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
    if denom == 0.0 { 0.0 } else { dot / denom }
}
