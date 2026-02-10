pub mod evidence;
pub mod embeddings;
pub mod draft;
pub mod guardrails;
pub mod llm;
pub mod ollama;
pub mod retrieve;

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
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from(std::env::temp_dir()).join(format!("incidentreview-evidence-test-{nanos}"));
        let store = EvidenceStore::open(root);
        let source = store
            .add_source(super::evidence::EvidenceAddSourceInput {
                source_type: super::evidence::EvidenceSourceType::FreeformText,
                origin: super::evidence::EvidenceOrigin {
                    kind: "paste".to_string(),
                    path: None,
                },
                label: "test".to_string(),
                created_at: "2026-02-10T00:00:00Z".to_string(),
                text: Some("hello world".to_string()),
            })
            .expect("add_source");
        store
            .build_chunks(Some(source.source_id.clone()), "2026-02-10T00:00:00Z")
            .expect("build_chunks");
        let chunks = store
            .list_chunks(super::evidence::EvidenceQueryStore {
                include_text: false,
                source_id: Some(source.source_id.clone()),
            })
            .expect("list");
        assert_eq!(chunks.len(), 1);
        let chunk = store.get_chunk(&chunks[0].chunk_id).expect("get");
        assert_eq!(chunk.source_id, source.source_id);
        assert!(chunk.text.contains("hello world"));
    }

    #[test]
    fn citation_guard_rejects_missing_citations() {
        assert!(enforce_citations("no citations here").is_err());
        assert!(enforce_citations("supported [[chunk:abc123]] citation").is_ok());
    }
}
