use std::path::PathBuf;

use qir_ai::draft::{draft_section_with_llm, AiDraftSectionRequest, SectionId};
use qir_ai::evidence::{EvidenceAddSourceInput, EvidenceOrigin, EvidenceSourceType, EvidenceStore};
use qir_ai::llm::Llm;
use qir_core::error::AppError;

struct MockLlm {
    out: String,
}

impl Llm for MockLlm {
    fn generate(&self, _model: &str, _prompt: &str) -> Result<String, AppError> {
        Ok(self.out.clone())
    }
}

fn setup_one_chunk_store() -> (EvidenceStore, String) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = PathBuf::from(std::env::temp_dir()).join(format!("incidentreview-ai-draft-test-{nanos}"));
    let evidence = EvidenceStore::open(root);
    let source = evidence
        .add_source(EvidenceAddSourceInput {
            source_type: EvidenceSourceType::FreeformText,
            origin: EvidenceOrigin {
                kind: "paste".to_string(),
                path: None,
            },
            label: "draft".to_string(),
            created_at: "2026-02-10T00:00:00Z".to_string(),
            text: Some("evidence chunk text".to_string()),
        })
        .expect("add_source");
    evidence
        .build_chunks(Some(source.source_id.clone()), "2026-02-10T00:00:00Z")
        .expect("build_chunks");
    let chunks = evidence
        .list_chunks(qir_ai::evidence::EvidenceQueryStore {
            include_text: false,
            source_id: Some(source.source_id),
        })
        .expect("list_chunks");
    assert_eq!(chunks.len(), 1);
    (evidence, chunks[0].chunk_id.clone())
}

#[test]
fn draft_requires_citations() {
    let (evidence, chunk_id) = setup_one_chunk_store();
    let llm = MockLlm {
        out: format!("Hello [[chunk:{chunk_id}]]"),
    };
    let err = draft_section_with_llm(
        &evidence,
        &llm,
        "mock",
        AiDraftSectionRequest {
            section_id: SectionId::ExecSummary,
            quarter_label: "Q1 2026".to_string(),
            prompt: "test".to_string(),
            citation_chunk_ids: vec![],
        },
    )
    .expect_err("should error");
    assert_eq!(err.code, "AI_CITATION_REQUIRED");
}

#[test]
fn draft_rejects_unknown_chunk_id() {
    let (evidence, _chunk_id) = setup_one_chunk_store();
    let llm = MockLlm {
        out: "Hello [[chunk:doesnotmatter]]".to_string(),
    };
    let err = draft_section_with_llm(
        &evidence,
        &llm,
        "mock",
        AiDraftSectionRequest {
            section_id: SectionId::ExecSummary,
            quarter_label: "Q1 2026".to_string(),
            prompt: "test".to_string(),
            citation_chunk_ids: vec!["missing".to_string()],
        },
    )
    .expect_err("should error");
    assert_eq!(err.code, "AI_CITATION_INVALID");
}

#[test]
fn draft_fails_when_model_output_has_no_citations() {
    let (evidence, chunk_id) = setup_one_chunk_store();
    let llm = MockLlm {
        out: "Hello with no citations".to_string(),
    };
    let err = draft_section_with_llm(
        &evidence,
        &llm,
        "mock",
        AiDraftSectionRequest {
            section_id: SectionId::ExecSummary,
            quarter_label: "Q1 2026".to_string(),
            prompt: "test".to_string(),
            citation_chunk_ids: vec![chunk_id],
        },
    )
    .expect_err("should error");
    assert_eq!(err.code, "AI_CITATION_REQUIRED");
}

#[test]
fn draft_fails_when_model_cites_unapproved_chunk_id() {
    let (evidence, chunk_id) = setup_one_chunk_store();
    let llm = MockLlm {
        out: "Hello [[chunk:other]]".to_string(),
    };
    let err = draft_section_with_llm(
        &evidence,
        &llm,
        "mock",
        AiDraftSectionRequest {
            section_id: SectionId::ExecSummary,
            quarter_label: "Q1 2026".to_string(),
            prompt: "test".to_string(),
            citation_chunk_ids: vec![chunk_id],
        },
    )
    .expect_err("should error");
    assert_eq!(err.code, "AI_CITATION_INVALID");
}

#[test]
fn draft_succeeds_with_valid_citation_marker() {
    let (evidence, chunk_id) = setup_one_chunk_store();
    let llm = MockLlm {
        out: format!("Executive summary [[chunk:{chunk_id}]]"),
    };
    let res = draft_section_with_llm(
        &evidence,
        &llm,
        "mock",
        AiDraftSectionRequest {
            section_id: SectionId::ExecSummary,
            quarter_label: "Q1 2026".to_string(),
            prompt: "test".to_string(),
            citation_chunk_ids: vec![chunk_id.clone()],
        },
    )
    .expect("should succeed");
    assert!(res.markdown.contains("[[chunk:"));
    assert_eq!(res.citations.len(), 1);
    assert_eq!(res.citations[0].chunk_id, chunk_id);
}
