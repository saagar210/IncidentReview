use qir_core::error::AppError;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
}

impl OllamaClient {
    /// Create a client for Ollama. This is strictly limited to `127.0.0.1`.
    pub fn new(base_url: &str) -> Result<Self, AppError> {
        let base_url = base_url.trim().trim_end_matches('/').to_string();

        // Binding constraint: local-only via 127.0.0.1.
        // Be strict: parse and validate host/port rather than using prefix matching.
        // Reject localhost/0.0.0.0/remote hosts and any URL with userinfo or path/query fragments.
        fn validate_base_url(url: &str) -> bool {
            let Some(rest) = url.strip_prefix("http://") else {
                return false;
            };
            // Disallow path/query/fragment; base_url must be host[:port] only.
            if rest.contains('/') || rest.contains('?') || rest.contains('#') || rest.contains('@') {
                return false;
            }
            let (host, port_opt) = match rest.split_once(':') {
                Some((h, p)) => (h, Some(p)),
                None => (rest, None),
            };
            if host != "127.0.0.1" {
                return false;
            }
            if let Some(p) = port_opt {
                if p.is_empty() {
                    return false;
                }
                if let Ok(v) = p.parse::<u16>() {
                    if v == 0 {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        }

        if !validate_base_url(&base_url) {
            return Err(AppError::new(
                "AI_REMOTE_NOT_ALLOWED",
                "Ollama base URL must be http://127.0.0.1[:port]",
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
            Err(e) => Err(
                AppError::new("AI_OLLAMA_UNHEALTHY", "Failed to reach Ollama on 127.0.0.1")
                    .with_details(e.to_string())
                    .with_retryable(true),
            ),
        }
    }

    pub fn base_url(&self) -> &str {
        self.base_url.as_str()
    }
}
