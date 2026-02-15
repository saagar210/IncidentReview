use qir_core::ai_drafts::{create_ai_draft, list_ai_drafts, AiDraftSectionType, CreateAiDraftInput};
use qir_core::db;
use sha2::Digest;

#[test]
fn ai_draft_persists_and_hash_matches_expected_sha256() {
    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let input = CreateAiDraftInput {
        quarter_label: "Q1 2026".to_string(),
        section_type: AiDraftSectionType::ExecSummary,
        draft_text: "Hello [[chunk:abc]]".to_string(),
        citation_chunk_ids: vec!["abc".to_string()],
        model_name: "llama3.2:latest".to_string(),
        model_params_hash: "params_hash".to_string(),
        prompt_template_version: "exec_summary_v1".to_string(),
        created_at: "2026-02-10T00:00:00Z".to_string(),
        parent_draft_id: None,
        revision_notes: None,
        branch_label: None,
    };

    let a = create_ai_draft(&conn, input.clone()).expect("create");

    #[derive(serde::Serialize)]
    struct HashPayload<'a> {
        quarter_label: &'a str,
        section_type: &'a str,
        draft_text: &'a str,
        citation_chunk_ids: &'a [String],
        model_name: &'a str,
        model_params_hash: &'a str,
        prompt_template_version: &'a str,
        created_at: &'a str,
    }

    let payload = HashPayload {
        quarter_label: "Q1 2026",
        section_type: "exec_summary",
        draft_text: "Hello [[chunk:abc]]",
        citation_chunk_ids: &vec!["abc".to_string()],
        model_name: "llama3.2:latest",
        model_params_hash: "params_hash",
        prompt_template_version: "exec_summary_v1",
        created_at: "2026-02-10T00:00:00Z",
    };

    let json = serde_json::to_string(&payload).expect("json");
    let digest = sha2::Sha256::digest(json.as_bytes());
    let expected = hex::encode(digest);
    assert_eq!(a.artifact_hash, expected);

    let all = list_ai_drafts(&conn, Some("Q1 2026")).expect("list");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].artifact_hash, expected);
}

#[test]
fn storing_without_citations_fails_and_does_not_insert() {
    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let err = create_ai_draft(
        &conn,
        CreateAiDraftInput {
            quarter_label: "Q1 2026".to_string(),
            section_type: AiDraftSectionType::ExecSummary,
            draft_text: "Hello".to_string(),
            citation_chunk_ids: vec![],
            model_name: "llama3.2:latest".to_string(),
            model_params_hash: "params_hash".to_string(),
            prompt_template_version: "exec_summary_v1".to_string(),
            created_at: "2026-02-10T00:00:00Z".to_string(),
            parent_draft_id: None,
            revision_notes: None,
            branch_label: None,
        },
    )
    .expect_err("should fail");

    assert_eq!(err.code, "AI_CITATION_REQUIRED");

    let all = list_ai_drafts(&conn, None).expect("list");
    assert!(all.is_empty());
}
