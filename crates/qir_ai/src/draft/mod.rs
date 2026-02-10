use qir_core::error::AppError;
use serde::{Deserialize, Serialize};

use crate::evidence::{Citation, EvidenceStore};
use crate::guardrails::enforce_citations;
use crate::llm::Llm;

mod prompts;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SectionId {
    ExecSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDraftSectionRequest {
    pub section_id: SectionId,
    pub quarter_label: String,
    pub prompt: String,
    pub citation_chunk_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDraftResponse {
    pub section_id: SectionId,
    pub markdown: String,
    pub citations: Vec<Citation>,
}

pub fn draft_section_with_llm(
    evidence: &EvidenceStore,
    llm: &dyn Llm,
    model: &str,
    req: AiDraftSectionRequest,
) -> Result<AiDraftResponse, AppError> {
    if req.citation_chunk_ids.is_empty() {
        return Err(AppError::new(
            "AI_CITATION_REQUIRED",
            "At least one citation chunk must be selected",
        ));
    }

    // Validate that the selected chunks exist and build canonical citations.
    let mut citations: Vec<Citation> = Vec::new();
    for chunk_id in req.citation_chunk_ids.iter() {
        let chunk = evidence.get_chunk(chunk_id).map_err(|e| {
            if e.code == "AI_EVIDENCE_NOT_FOUND" {
                AppError::new("AI_CITATION_INVALID", "Citation chunk_id not found")
                    .with_details(format!("chunk_id={}", chunk_id))
            } else {
                e
            }
        })?;
        citations.push(evidence.citation_for_chunk(&chunk));
    }
    evidence.validate_citations(&citations)?;

    let evidence_blocks = build_evidence_blocks(evidence, &req.citation_chunk_ids)?;

    let prompt = match req.section_id {
        SectionId::ExecSummary => prompts::exec_summary_prompt(
            &req.quarter_label,
            &req.prompt,
            &evidence_blocks,
        ),
    };

    let markdown = llm.generate(model, &prompt)?;

    // Guardrails: require at least one citation marker in output.
    enforce_citations(&markdown).map_err(|e| {
        AppError::new("AI_CITATION_REQUIRED", "Draft missing citations")
            .with_details(e.to_string())
    })?;

    // Validate that any cited chunk IDs are within the allowed set.
    let cited_ids = extract_cited_chunk_ids(&markdown);
    if cited_ids.is_empty() {
        return Err(AppError::new(
            "AI_CITATION_REQUIRED",
            "Draft missing citations",
        ));
    }
    let allowed: std::collections::BTreeSet<String> =
        req.citation_chunk_ids.iter().cloned().collect();
    for cid in cited_ids.iter() {
        if !allowed.contains(cid) {
            return Err(AppError::new(
                "AI_CITATION_INVALID",
                "Draft cited an unapproved chunk_id",
            )
            .with_details(format!("chunk_id={}", cid)));
        }
    }

    // Return citations for the cited IDs (stable ordering).
    let mut out_citations: Vec<Citation> = Vec::new();
    let mut cited_sorted = cited_ids.into_iter().collect::<Vec<_>>();
    cited_sorted.sort();
    cited_sorted.dedup();
    for cid in cited_sorted {
        let chunk = evidence.get_chunk(&cid)?;
        out_citations.push(evidence.citation_for_chunk(&chunk));
    }

    Ok(AiDraftResponse {
        section_id: req.section_id,
        markdown,
        citations: out_citations,
    })
}

fn build_evidence_blocks(evidence: &EvidenceStore, chunk_ids: &[String]) -> Result<String, AppError> {
    let mut blocks: Vec<String> = Vec::new();
    for cid in chunk_ids {
        let chunk = evidence.get_chunk(cid)?;
        blocks.push(format!(
            "[[chunk:{cid}]] source_id={} ordinal={} text_sha256={}\n{}",
            chunk.source_id, chunk.ordinal, chunk.text_sha256, chunk.text
        ));
    }
    Ok(blocks.join("\n\n---\n\n"))
}

fn extract_cited_chunk_ids(markdown: &str) -> std::collections::BTreeSet<String> {
    // Parse `[[chunk:<id>]]` markers.
    let mut out = std::collections::BTreeSet::new();
    let bytes = markdown.as_bytes();
    let mut i = 0usize;
    while i + 8 < bytes.len() {
        if bytes[i..].starts_with(b"[[chunk:") {
            let start = i + 8;
            if let Some(end) = bytes[start..].iter().position(|&b| b == b']') {
                let s = &markdown[start..start + end];
                // Expect closing "]]" after the first ']'.
                let after = start + end;
                if markdown.get(after..after + 2) == Some("]]") {
                    let id = s.trim();
                    if !id.is_empty() {
                        out.insert(id.to_string());
                    }
                }
                i = after + 2;
                continue;
            }
        }
        i += 1;
    }
    out
}

