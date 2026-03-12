use crate::db::{repository, Database};
use crate::models::search::SearchResult;

pub fn search(db: &Database, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
    let conn = db.lock_conn();
    repository::search_filename(&conn, query, limit).map_err(|e| e.to_string())
}
