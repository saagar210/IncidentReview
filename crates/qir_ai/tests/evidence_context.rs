use qir_ai::evidence::{EvidenceAddSourceInput, EvidenceOrigin, EvidenceQueryStore, EvidenceSourceType, EvidenceStore};
use qir_core::error::AppError;
use tempfile::tempdir;

#[test]
fn context_returns_stable_window_in_ordinal_order() {
    let dir = tempdir().unwrap();
    let store = EvidenceStore::open(dir.path().to_path_buf());

    let p1 = "a".repeat(900);
    let p2 = "b".repeat(900);
    let p3 = "c".repeat(900);
    let text = format!("{p1}\n\n{p2}\n\n{p3}");

    let source = store
        .add_source(EvidenceAddSourceInput {
            source_type: EvidenceSourceType::FreeformText,
            origin: EvidenceOrigin {
                kind: "paste".to_string(),
                path: None,
            },
            label: "freeform".to_string(),
            created_at: "2026-02-10T00:00:00Z".to_string(),
            text: Some(text),
        })
        .unwrap();

    store
        .build_chunks(Some(source.source_id.clone()), "2026-02-10T00:00:00Z")
        .unwrap();

    let chunks = store
        .list_chunks(EvidenceQueryStore {
            include_text: false,
            source_id: Some(source.source_id.clone()),
        })
        .unwrap();
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].ordinal, 0);
    assert_eq!(chunks[1].ordinal, 1);
    assert_eq!(chunks[2].ordinal, 2);

    let center_id = chunks[1].chunk_id.clone();
    let ctx = store.get_context(&center_id, 1).unwrap();
    assert_eq!(ctx.center_chunk_id, center_id);
    assert_eq!(ctx.chunks.len(), 3);
    assert_eq!(ctx.chunks[0].ordinal, 0);
    assert_eq!(ctx.chunks[1].ordinal, 1);
    assert_eq!(ctx.chunks[2].ordinal, 2);
}

#[test]
fn context_returns_not_found_for_unknown_chunk_id() {
    let dir = tempdir().unwrap();
    let store = EvidenceStore::open(dir.path().to_path_buf());

    let err: AppError = store.get_context("does_not_exist", 1).unwrap_err();
    assert_eq!(err.code, "AI_EVIDENCE_NOT_FOUND");
}

