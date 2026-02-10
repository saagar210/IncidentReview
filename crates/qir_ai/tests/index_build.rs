use std::path::PathBuf;

use qir_ai::embeddings::Embedder;
use qir_ai::evidence::{AiIndexBuildInput, EvidenceAddSourceInput, EvidenceOrigin, EvidenceSourceType, EvidenceStore, IndexStore};

use qir_core::error::AppError;

use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingEmbedder {
    calls: AtomicUsize,
}

impl CountingEmbedder {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
        }
    }

    fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl Embedder for CountingEmbedder {
    fn embed(&self, _model: &str, input: &str) -> Result<Vec<f32>, AppError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        // Deterministic embedding: [len, first_byte, last_byte]
        let bytes = input.as_bytes();
        let first = bytes.first().copied().unwrap_or(0) as f32;
        let last = bytes.last().copied().unwrap_or(0) as f32;
        Ok(vec![bytes.len() as f32, first, last])
    }
}

#[test]
fn builds_index_incrementally_and_embeds_only_changed_chunks() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = PathBuf::from(std::env::temp_dir()).join(format!("incidentreview-ai-index-test-{nanos}"));
    let evidence = EvidenceStore::open(root.clone());

    let p1 = "a".repeat(900);
    let p2 = "b".repeat(900);
    let text_v1 = format!("{p1}\n\n{p2}");

    let source = evidence
        .add_source(EvidenceAddSourceInput {
            source_type: EvidenceSourceType::FreeformText,
            origin: EvidenceOrigin {
                kind: "paste".to_string(),
                path: None,
            },
            label: "test".to_string(),
            created_at: "2026-02-10T00:00:00Z".to_string(),
            text: Some(text_v1),
        })
        .expect("add_source");
    evidence
        .build_chunks(Some(source.source_id.clone()), "2026-02-10T00:00:00Z")
        .expect("build_chunks");

    let index = IndexStore::open(root);
    let embedder = CountingEmbedder::new();
    let st = index
        .build_with_embedder(
            &evidence,
            &embedder,
            AiIndexBuildInput {
                model: "mock".to_string(),
                source_id: Some(source.source_id),
                updated_at: "2026-02-10T00:00:00Z".to_string(),
            },
        )
        .expect("build_index");

    assert!(st.ready);
    assert_eq!(st.chunks_total, 2);
    assert_eq!(st.chunk_count, 2);
    assert_eq!(st.model.as_deref(), Some("mock"));
    assert_eq!(st.dims, Some(3));
    assert_eq!(embedder.call_count(), 2);

    // Rebuild without changes: should not call embedder again.
    let st2 = index
        .build_with_embedder(
            &evidence,
            &embedder,
            AiIndexBuildInput {
                model: "mock".to_string(),
                source_id: st.source_id.clone(),
                updated_at: "2026-02-10T01:00:00Z".to_string(),
            },
        )
        .expect("build_index_2");
    assert!(st2.ready);
    assert_eq!(embedder.call_count(), 2);

    // Change only the first paragraph -> only one chunk should require embedding.
    let p1b = format!("{}x", "a".repeat(900));
    let text_v2 = format!("{p1b}\n\n{p2}");
    let source2 = evidence
        .add_source(EvidenceAddSourceInput {
            source_type: EvidenceSourceType::FreeformText,
            origin: EvidenceOrigin {
                kind: "paste".to_string(),
                path: None,
            },
            label: "test".to_string(),
            created_at: "2026-02-10T00:00:00Z".to_string(),
            text: Some(text_v2),
        })
        .expect("add_source_2");
    evidence
        .build_chunks(Some(source2.source_id.clone()), "2026-02-10T02:00:00Z")
        .expect("build_chunks_2");

    let st3 = index
        .build_with_embedder(
            &evidence,
            &embedder,
            AiIndexBuildInput {
                model: "mock".to_string(),
                source_id: Some(source2.source_id),
                updated_at: "2026-02-10T02:00:00Z".to_string(),
            },
        )
        .expect("build_index_3");
    assert!(st3.ready);
    assert_eq!(st3.chunks_total, 2);
    assert_eq!(embedder.call_count(), 3);
}
