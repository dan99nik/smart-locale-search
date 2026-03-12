use std::path::Path;
use std::sync::{Arc, Mutex};

use ndarray::{Array, Array4};
use ort::session::Session;
use ort::value::Tensor;

use super::{EmbeddingError, ImageEmbeddingProvider};

const CLIP_DIM: usize = 512;
const CLIP_IMAGE_SIZE: u32 = 224;

const CLIP_MEAN: [f32; 3] = [0.48145466, 0.4578275, 0.40821073];
const CLIP_STD: [f32; 3] = [0.26862954, 0.26130258, 0.27577711];

pub struct ClipProvider {
    visual_session: Mutex<Session>,
    textual_session: Mutex<Session>,
}

impl ClipProvider {
    pub fn new(model_dir: &Path) -> Result<Self, EmbeddingError> {
        let visual_path = model_dir.join("clip-visual.onnx");
        let textual_path = model_dir.join("clip-textual.onnx");

        if !visual_path.exists() {
            return Err(EmbeddingError::ModelNotLoaded(format!(
                "CLIP visual model not found at: {}", visual_path.display()
            )));
        }
        if !textual_path.exists() {
            return Err(EmbeddingError::ModelNotLoaded(format!(
                "CLIP textual model not found at: {}", textual_path.display()
            )));
        }

        let visual_session = Session::builder()
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?
            .with_intra_threads(4)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?
            .commit_from_file(&visual_path)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;

        let textual_session = Session::builder()
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?
            .with_intra_threads(4)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?
            .commit_from_file(&textual_path)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;

        Ok(Self {
            visual_session: Mutex::new(visual_session),
            textual_session: Mutex::new(textual_session),
        })
    }

    fn preprocess_image(path: &Path) -> Result<Array4<f32>, EmbeddingError> {
        let img = image::open(path)
            .map_err(|e| EmbeddingError::InferenceError(format!("Failed to open image: {}", e)))?;

        let resized = img.resize_exact(
            CLIP_IMAGE_SIZE,
            CLIP_IMAGE_SIZE,
            image::imageops::FilterType::Lanczos3,
        );
        let rgb = resized.to_rgb8();

        let mut pixel_data = Array4::<f32>::zeros((1, 3, CLIP_IMAGE_SIZE as usize, CLIP_IMAGE_SIZE as usize));

        for y in 0..CLIP_IMAGE_SIZE as usize {
            for x in 0..CLIP_IMAGE_SIZE as usize {
                let pixel = rgb.get_pixel(x as u32, y as u32);
                for c in 0..3 {
                    pixel_data[[0, c, y, x]] =
                        (pixel[c] as f32 / 255.0 - CLIP_MEAN[c]) / CLIP_STD[c];
                }
            }
        }

        Ok(pixel_data)
    }

    fn normalize_embedding(embedding: &mut [f32]) {
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in embedding.iter_mut() {
                *x /= norm;
            }
        }
    }
}

impl ImageEmbeddingProvider for ClipProvider {
    fn embed_image(&self, path: &Path) -> Result<Vec<f32>, EmbeddingError> {
        let pixel_data = Self::preprocess_image(path)?;

        let input_tensor = Tensor::from_array(pixel_data)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;

        let mut session = self.visual_session.lock()
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;

        let outputs = session
            .run(ort::inputs!["pixel_values" => input_tensor])
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;

        let output_value = &outputs[0];
        let (_shape, data) = output_value
            .try_extract_tensor::<f32>()
            .map_err(|e: ort::error::Error| EmbeddingError::InferenceError(e.to_string()))?;

        let mut embedding: Vec<f32> = data.iter().take(CLIP_DIM).copied().collect();
        Self::normalize_embedding(&mut embedding);

        Ok(embedding)
    }

    fn embed_text(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let tokens = simple_clip_tokenize(text);
        let seq_len = tokens.len();

        let token_array = Array::from_shape_vec(
            (1, seq_len),
            tokens.iter().map(|&t| t as i64).collect(),
        ).map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;

        let input_tensor = Tensor::from_array(token_array)
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;

        let mut session = self.textual_session.lock()
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;

        let outputs = session
            .run(ort::inputs!["input_ids" => input_tensor])
            .map_err(|e| EmbeddingError::InferenceError(e.to_string()))?;

        let output_value = &outputs[0];
        let (_shape, data) = output_value
            .try_extract_tensor::<f32>()
            .map_err(|e: ort::error::Error| EmbeddingError::InferenceError(e.to_string()))?;

        let mut embedding: Vec<f32> = data.iter().take(CLIP_DIM).copied().collect();
        Self::normalize_embedding(&mut embedding);

        Ok(embedding)
    }

    fn dimension(&self) -> usize {
        CLIP_DIM
    }

    fn model_name(&self) -> &str {
        "open-clip-ViT-B-32"
    }
}

/// Minimal CLIP tokenizer — CLIP uses a simple BPE but for the ONNX textual
/// model from Marqo/open_clip, the input is already token IDs. We use a basic
/// word-level tokenization that maps ASCII chars to token space. The Marqo
/// textual ONNX models accept raw input_ids with SOT=49406, EOT=49407.
fn simple_clip_tokenize(text: &str) -> Vec<i32> {
    let max_len = 77;
    let sot_token = 49406i32;
    let eot_token = 49407i32;

    let mut tokens = vec![sot_token];

    let cleaned = text.to_lowercase();
    for ch in cleaned.chars().take(max_len - 2) {
        let code = ch as u32;
        if code < 256 {
            tokens.push(code as i32 + 1);
        }
    }

    tokens.push(eot_token);

    while tokens.len() < max_len {
        tokens.push(0);
    }
    tokens.truncate(max_len);

    tokens
}

pub fn create_provider(app_data_dir: &Path) -> Option<Arc<dyn ImageEmbeddingProvider>> {
    let model_dir = app_data_dir.join("models").join("open-clip-vit-b-32");
    match ClipProvider::new(&model_dir) {
        Ok(provider) => {
            log::info!("Loaded CLIP vision model: open-clip-ViT-B-32");
            Some(Arc::new(provider))
        }
        Err(e) => {
            log::warn!("CLIP model not available: {}. Image semantic search disabled.", e);
            None
        }
    }
}
