use qir_core::error::AppError;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
}

impl OllamaClient {
    /// Create a client for Ollama. This is strictly limited to `127.0.0.1`.
    pub fn new(base_url: &str) -> Result<Self, AppError> {
        let base_url = base_url.trim_end_matches('/').to_string();

        // Binding constraint: local-only via 127.0.0.1.
        if !base_url.starts_with("http://127.0.0.1:") && base_url != "http://127.0.0.1" {
            return Err(AppError::new(
                "AI_REMOTE_NOT_ALLOWED",
                "Ollama base URL must be localhost (127.0.0.1)",
            )
            .with_details(format!("base_url={base_url}")));
        }

        Ok(Self { base_url })
    }

    pub fn health_check(&self) -> Result<(), AppError> {
        let url = format!("{}/api/tags", self.base_url);
        let resp = ureq::get(&url)
            .timeout(std::time::Duration::from_millis(800))
            .call();

        match resp {
            Ok(r) if r.status() == 200 => Ok(()),
            Ok(r) => Err(
                AppError::new("AI_OLLAMA_UNHEALTHY", "Ollama health check failed")
                    .with_details(format!("status={}", r.status())),
            ),
            Err(e) => Err(AppError::new(
                "AI_OLLAMA_UNREACHABLE",
                "Failed to reach Ollama on 127.0.0.1",
            )
            .with_details(e.to_string())
            .with_retryable(true)),
        }
    }
}
