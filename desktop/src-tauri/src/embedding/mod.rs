pub mod e5_small;
pub mod clip;

pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError>;
    fn dimension(&self) -> usize;
    fn model_name(&self) -> &str;
}

pub trait ImageEmbeddingProvider: Send + Sync {
    fn embed_image(&self, path: &std::path::Path) -> Result<Vec<f32>, EmbeddingError>;
    fn embed_text(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    fn dimension(&self) -> usize;
    fn model_name(&self) -> &str;
}

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),
    #[error("Inference error: {0}")]
    InferenceError(String),
    #[error("Tokenization error: {0}")]
    TokenizationError(String),
}
