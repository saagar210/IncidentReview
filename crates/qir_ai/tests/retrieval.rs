use std::path::PathBuf;

use qir_ai::embeddings::Embedder;
use qir_ai::evidence::{
    AiIndexBuildInput, EvidenceAddSourceInput, EvidenceOrigin, EvidenceSourceType, EvidenceStore, IndexStore,
};
use qir_ai::retrieve::query_with_embedder;
use qir_core::error::AppError;

struct CountABEmbedder;

impl Embedder for CountABEmbedder {
    fn embed(&self, _model: &str, input: &str) -> Result<Vec<f32>, AppError> {
        let mut a = 0u32;
        let mut b = 0u32;
        for ch in input.chars() {
            if ch == 'a' {
                a += 1;
            } else if ch == 'b' {
                b += 1;
            }
        }
        Ok(vec![a as f32, b as f32])
    }
}

#[test]
fn retrieval_returns_stable_topk_and_tie_breaks_by_chunk_id() {
    let root = PathBuf::from(std::env::temp_dir()).join(format!(
        "incidentreview-ai-retrieval-test-{}",
        std::process::id()
    ));
    let evidence = EvidenceStore::open(root.clone());

    let para_a = "a".repeat(1000);
    let para_b = "b".repeat(1000);
    let text = format!("{para_a}\n\n{para_b}");

    let source = evidence
        .add_source(EvidenceAddSourceInput {
            source_type: EvidenceSourceType::FreeformText,
            origin: EvidenceOrigin {
                kind: "paste".to_string(),
                path: None,
            },
            label: "ab".to_string(),
            created_at: "2026-02-10T00:00:00Z".to_string(),
            text: Some(text),
        })
        .expect("add_source");
    evidence
        .build_chunks(Some(source.source_id.clone()), "2026-02-10T00:00:00Z")
        .expect("build_chunks");

    let index = IndexStore::open(root);
    index
        .build_with_embedder(
            &evidence,
            &CountABEmbedder,
            AiIndexBuildInput {
                model: "mock".to_string(),
                source_id: Some(source.source_id.clone()),
                updated_at: "2026-02-10T00:00:00Z".to_string(),
            },
        )
        .expect("build_index");

    // Query biased toward 'a' should rank the 'a' chunk first.
    let res = query_with_embedder(
        &evidence,
        &index,
        &CountABEmbedder,
        "aaaa",
        2,
        Some(&[source.source_id.clone()]),
    )
    .expect("query");
    assert_eq!(res.hits.len(), 2);
    assert!(res.hits[0].snippet.starts_with('a'));
    assert!(res.hits[1].snippet.starts_with('b'));

    // Tie query should order by chunk_id asc as a deterministic tie-breaker.
    let tie = query_with_embedder(
        &evidence,
        &index,
        &CountABEmbedder,
        "ab",
        2,
        Some(&[source.source_id]),
    )
    .expect("query");
    assert_eq!(tie.hits.len(), 2);
    assert!(tie.hits[0].chunk_id < tie.hits[1].chunk_id);
}

