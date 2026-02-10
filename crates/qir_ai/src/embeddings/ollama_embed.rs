use qir_core::error::AppError;
use serde::{Deserialize, Serialize};

use crate::ollama::OllamaClient;
use super::Embedder;

#[derive(Debug, Clone)]
pub struct OllamaEmbedder {
    client: OllamaClient,
}

impl OllamaEmbedder {
    pub fn new(client: OllamaClient) -> Self {
        Self { client }
    }
}

#[derive(Debug, Clone, Serialize)]
struct EmbeddingsRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Debug, Clone, Deserialize)]
struct EmbeddingsResponse {
    embedding: Vec<f32>,
}

impl Embedder for OllamaEmbedder {
    fn embed(&self, model: &str, input: &str) -> Result<Vec<f32>, AppError> {
        // Keep requests bounded. Chunking enforces reasonable sizes, but guard anyway.
        let prompt = if input.len() > 12_000 { &input[..12_000] } else { input };

        let url = format!("{}/api/embeddings", self.client.base_url());
        let req = EmbeddingsRequest { model, prompt };
        let resp = ureq::post(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send_json(serde_json::to_value(req).map_err(|e| {
                AppError::new("AI_EMBEDDINGS_FAILED", "Failed to encode embeddings request")
                    .with_details(e.to_string())
            })?);

        match resp {
            Ok(r) if r.status() == 200 => {
                let v: EmbeddingsResponse = r.into_json().map_err(|e| {
                    AppError::new("AI_EMBEDDINGS_FAILED", "Failed to decode embeddings response")
                        .with_details(e.to_string())
                })?;
                if v.embedding.is_empty() {
                    return Err(AppError::new(
                        "AI_EMBEDDINGS_FAILED",
                        "Embeddings response was empty",
                    ));
                }
                Ok(v.embedding)
            }
            Ok(r) => Err(
                AppError::new("AI_EMBEDDINGS_FAILED", "Embeddings request failed")
                    .with_details(format!("status={}", r.status())),
            ),
            Err(e) => Err(
                AppError::new("AI_EMBEDDINGS_FAILED", "Failed to call embeddings endpoint")
                    .with_details(e.to_string())
                    .with_retryable(true),
            ),
        }
    }
}

