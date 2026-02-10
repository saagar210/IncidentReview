use qir_core::error::AppError;
use serde::{Deserialize, Serialize};

use crate::ollama::OllamaClient;
use super::Llm;

#[derive(Debug, Clone)]
pub struct OllamaLlm {
    client: OllamaClient,
}

impl OllamaLlm {
    pub fn new(client: OllamaClient) -> Self {
        Self { client }
    }
}

#[derive(Debug, Clone, Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct GenerateResponse {
    response: String,
}

impl Llm for OllamaLlm {
    fn generate(&self, model: &str, prompt: &str) -> Result<String, AppError> {
        let url = format!("{}/api/generate", self.client.base_url());
        let req = GenerateRequest {
            model,
            prompt,
            stream: false,
        };

        let resp = ureq::post(&url)
            .timeout(std::time::Duration::from_secs(30))
            .send_json(serde_json::to_value(req).map_err(|e| {
                AppError::new("AI_DRAFT_FAILED", "Failed to encode draft request")
                    .with_details(e.to_string())
            })?);

        match resp {
            Ok(r) if r.status() == 200 => {
                let v: GenerateResponse = r.into_json().map_err(|e| {
                    AppError::new("AI_DRAFT_FAILED", "Failed to decode draft response")
                        .with_details(e.to_string())
                })?;
                if v.response.trim().is_empty() {
                    return Err(AppError::new("AI_DRAFT_FAILED", "Draft response was empty"));
                }
                Ok(v.response)
            }
            Ok(r) => Err(
                AppError::new("AI_DRAFT_FAILED", "Draft request failed")
                    .with_details(format!("status={}", r.status())),
            ),
            Err(e) => Err(
                AppError::new("AI_DRAFT_FAILED", "Failed to call draft endpoint")
                    .with_details(e.to_string())
                    .with_retryable(true),
            ),
        }
    }
}

