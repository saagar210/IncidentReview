use qir_core::error::AppError;

pub trait Embedder {
    fn embed(&self, model: &str, input: &str) -> Result<Vec<f32>, AppError>;
}

pub mod ollama_embed;

