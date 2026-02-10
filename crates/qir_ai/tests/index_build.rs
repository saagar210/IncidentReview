use std::path::PathBuf;

use qir_ai::embeddings::Embedder;
use qir_ai::evidence::{AiIndexBuildInput, EvidenceAddSourceInput, EvidenceOrigin, EvidenceSourceType, EvidenceStore, IndexStore};

use qir_core::error::AppError;

struct MockEmbedder;

impl Embedder for MockEmbedder {
    fn embed(&self, _model: &str, input: &str) -> Result<Vec<f32>, AppError> {
        // Deterministic embedding: [len, first_byte, last_byte]
        let bytes = input.as_bytes();
        let first = bytes.first().copied().unwrap_or(0) as f32;
        let last = bytes.last().copied().unwrap_or(0) as f32;
        Ok(vec![bytes.len() as f32, first, last])
    }
}

#[test]
fn builds_index_from_existing_chunks_with_mock_embedder() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = PathBuf::from(std::env::temp_dir()).join(format!("incidentreview-ai-index-test-{nanos}"));
    let evidence = EvidenceStore::open(root.clone());
    let source = evidence
        .add_source(EvidenceAddSourceInput {
            source_type: EvidenceSourceType::FreeformText,
            origin: EvidenceOrigin {
                kind: "paste".to_string(),
                path: None,
            },
            label: "test".to_string(),
            created_at: "2026-02-10T00:00:00Z".to_string(),
            text: Some("alpha\n\nbeta".to_string()),
        })
        .expect("add_source");
    evidence
        .build_chunks(Some(source.source_id.clone()), "2026-02-10T00:00:00Z")
        .expect("build_chunks");

    let index = IndexStore::open(root);
    let st = index
        .build_with_embedder(
            &evidence,
            &MockEmbedder,
            AiIndexBuildInput {
                model: "mock".to_string(),
                source_id: Some(source.source_id),
                updated_at: "2026-02-10T00:00:00Z".to_string(),
            },
        )
        .expect("build_index");

    assert!(st.ready);
    assert_eq!(st.chunk_count, 1);
    assert_eq!(st.model.as_deref(), Some("mock"));
    assert_eq!(st.dims, Some(3));
}
