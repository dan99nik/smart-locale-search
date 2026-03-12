use std::path::Path;
use std::sync::{Arc, Mutex};

use ndarray::Array2;
use ort::session::Session;
use ort::value::Tensor;
use tokenizers::Tokenizer;

use super::{EmbeddingError, EmbeddingProvider};

const MODEL_DIM: usize = 384;

pub struct E5SmallProvider {
    session: Mutex<Session>,
    tokenizer: Tokenizer,
}

impl E5SmallProvider {
    pub fn new(model_dir: &Path) -> Result<Self, EmbeddingError> {
        let model_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");

        if !model_path.exists() {
            return Err(EmbeddingError::ModelNotLoaded(format!(
                "ONNX model not found at: {}",
                model_path.display()
            )));
        }
        if !tokenizer_path.exists() {
            return Err(EmbeddingError::ModelNotLoaded(format!(
                "Tokenizer not found at: {}",
                tokenizer_path.display()
            )));
        }

        let session = Session::builder()
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?
            .with_intra_threads(4)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?
            .commit_from_file(&model_path)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| EmbeddingError::TokenizationError(e.to_string()))?;

        Ok(Self {
            session: Mutex::new(session),
            tokenizer,
        })
    }

    fn run_inference(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| EmbeddingError::TokenizationError(e.to_string()))?;

        let ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&m| m as i64)
            .collect();
        let token_type: Vec<i64> = encoding
            .get_type_ids()
            .iter()
            .map(|&t| t as i64)
            .collect();

        let seq_len = ids.len();

        let input_ids_array = Array2::from_shape_vec((1, seq_len), ids)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;
        let attention_mask_array = Array2::from_shape_vec((1, seq_len), mask)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;
        let token_type_array = Array2::from_shape_vec((1, seq_len), token_type)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;

        let input_ids_tensor = Tensor::from_array(input_ids_array)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;
        let attention_mask_tensor = Tensor::from_array(attention_mask_array)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;
        let token_type_tensor = Tensor::from_array(token_type_array)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;

        let mut session = self.session.lock()
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;
        let outputs = session
            .run(ort::inputs![
                "input_ids" => input_ids_tensor,
                "attention_mask" => attention_mask_tensor,
                "token_type_ids" => token_type_tensor,
            ])
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;

        let output_value = &outputs[0];
        let (shape, data) = output_value
            .try_extract_tensor::<f32>()
            .map_err(|e: ort::error::Error| EmbeddingError::InferenceError(e.to_string()))?;

        // shape is [batch=1, seq_len, hidden_dim] (Shape derefs to [i64])
        if shape.len() != 3 {
            return Err(EmbeddingError::InferenceError(format!(
                "Expected 3D tensor, got {}D",
                shape.len()
            )));
        }

        let seq_len_out = shape[1] as usize;
        let hidden_dim = shape[2] as usize;

        // Mean pooling over sequence dimension
        let mut pooled = vec![0.0f32; hidden_dim];
        for s in 0..seq_len_out {
            for h in 0..hidden_dim {
                pooled[h] += data[s * hidden_dim + h];
            }
        }
        for h in 0..hidden_dim {
            pooled[h] /= seq_len_out as f32;
        }

        // L2 normalize
        let norm: f32 = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in pooled.iter_mut() {
                *x /= norm;
            }
        }

        Ok(pooled)
    }
}

impl EmbeddingProvider for E5SmallProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.run_inference(text)
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        texts.iter().map(|t| self.run_inference(t)).collect()
    }

    fn dimension(&self) -> usize {
        MODEL_DIM
    }

    fn model_name(&self) -> &str {
        "multilingual-e5-small"
    }
}

pub fn create_provider(app_data_dir: &Path) -> Option<Arc<dyn EmbeddingProvider>> {
    let model_dir = app_data_dir.join("models").join("multilingual-e5-small");
    match E5SmallProvider::new(&model_dir) {
        Ok(provider) => {
            log::info!("Loaded embedding model: multilingual-e5-small");
            Some(Arc::new(provider))
        }
        Err(e) => {
            log::warn!("Embedding model not available: {}. Semantic search disabled.", e);
            None
        }
    }
}
