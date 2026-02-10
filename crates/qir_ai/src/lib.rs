pub mod evidence;
pub mod guardrails;
pub mod ollama;

#[cfg(test)]
mod tests {
    use super::ollama::OllamaClient;
    use super::{evidence::EvidenceStore, guardrails::enforce_citations};
    use std::path::PathBuf;

    #[test]
    fn enforces_localhost_only_base_url() {
        assert!(OllamaClient::new("http://127.0.0.1:11434").is_ok());
        assert!(OllamaClient::new("http://127.0.0.1").is_ok());

        assert!(OllamaClient::new("http://localhost:11434").is_err());
        assert!(OllamaClient::new("http://0.0.0.0:11434").is_err());
        assert!(OllamaClient::new("http://[::1]:11434").is_err());
        assert!(OllamaClient::new("https://example.com").is_err());

        // Harden against prefix-based bypasses.
        assert!(OllamaClient::new("http://127.0.0.1.evil.com:11434").is_err());
        assert!(OllamaClient::new("http://127.0.0.1@evil.com:11434").is_err());
        assert!(OllamaClient::new("http://127.0.0.1:").is_err());
        assert!(OllamaClient::new("http://127.0.0.1:0").is_err());
        assert!(OllamaClient::new("http://127.0.0.1:99999").is_err());
        assert!(OllamaClient::new("http://127.0.0.1:11434/").is_ok()); // trailing slash is trimmed
        assert!(OllamaClient::new("http://127.0.0.1:11434/api").is_err());
    }

    #[test]
    fn evidence_store_roundtrip() {
        let root = PathBuf::from(std::env::temp_dir()).join(format!(
            "incidentreview-evidence-test-{}",
            std::process::id()
        ));
        let store = EvidenceStore::new(root);
        let chunk = store
            .put_chunk("slack_sample.txt", "hello world")
            .expect("put");
        let got = store.get_chunk(&chunk.id).expect("get");
        assert_eq!(chunk, got);
    }

    #[test]
    fn citation_guard_rejects_missing_citations() {
        assert!(enforce_citations("no citations here").is_err());
        assert!(enforce_citations("supported [[chunk:abc123]] citation").is_ok());
    }
}
