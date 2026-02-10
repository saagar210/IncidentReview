use qir_core::error::AppError;

pub trait Llm {
    fn generate(&self, model: &str, prompt: &str) -> Result<String, AppError>;
}

pub mod ollama_llm;

