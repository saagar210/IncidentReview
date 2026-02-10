use qir_core::error::AppError;
use serde::{Deserialize, Serialize};

use crate::evidence::{Citation, EvidenceStore};
use crate::guardrails::enforce_citations;
use crate::llm::Llm;
use sha2::{Digest, Sha256};

mod prompts;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SectionId {
    ExecSummary,
    IncidentHighlightsTopN,
    ThemeAnalysis,
    ActionPlanNextQuarter,
    QuarterNarrativeRecap,
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
    pub model_name: String,
    pub model_params_hash: String,
    pub prompt_template_version: String,
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
        SectionId::IncidentHighlightsTopN => prompts::incident_highlights_top_n_prompt(
            &req.quarter_label,
            &req.prompt,
            &evidence_blocks,
        ),
        SectionId::ThemeAnalysis => prompts::theme_analysis_prompt(
            &req.quarter_label,
            &req.prompt,
            &evidence_blocks,
        ),
        SectionId::ActionPlanNextQuarter => prompts::action_plan_next_quarter_prompt(
            &req.quarter_label,
            &req.prompt,
            &evidence_blocks,
        ),
        SectionId::QuarterNarrativeRecap => prompts::quarter_narrative_recap_prompt(
            &req.quarter_label,
            &req.prompt,
            &evidence_blocks,
        ),
    };

    let prompt_template_version = match req.section_id {
        SectionId::ExecSummary => "exec_summary_v1".to_string(),
        SectionId::IncidentHighlightsTopN => "incident_highlights_top_n_v1".to_string(),
        SectionId::ThemeAnalysis => "theme_analysis_v1".to_string(),
        SectionId::ActionPlanNextQuarter => "action_plan_next_quarter_v1".to_string(),
        SectionId::QuarterNarrativeRecap => "quarter_narrative_recap_v1".to_string(),
    };
    let model_params_hash = compute_model_params_hash(model)?;

    let markdown = llm.generate(model, &prompt)?;

    validate_section_citations(req.section_id.clone(), &markdown).map_err(|e| {
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
        model_name: model.to_string(),
        model_params_hash,
        prompt_template_version,
    })
}

fn validate_section_citations(section_id: SectionId, markdown: &str) -> Result<(), AppError> {
    // Always require at least one citation marker somewhere.
    enforce_citations(markdown)?;

    match section_id {
        SectionId::ExecSummary => Ok(()),
        SectionId::IncidentHighlightsTopN | SectionId::ThemeAnalysis | SectionId::ActionPlanNextQuarter => {
            validate_each_list_item_has_citation(markdown)
        }
        SectionId::QuarterNarrativeRecap => validate_each_paragraph_has_citation(markdown),
    }
}

fn validate_each_list_item_has_citation(markdown: &str) -> Result<(), AppError> {
    let mut missing: Vec<usize> = Vec::new();
    for (idx, line) in markdown.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with("- ") || t.starts_with("* ") {
            if !t.contains("[[chunk:") {
                missing.push(idx + 1);
            }
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    Err(AppError::new(
        "AI_CITATION_REQUIRED",
        "Each list item must include at least one citation marker",
    )
    .with_details(format!("missing_citation_lines={missing:?}")))
}

fn validate_each_paragraph_has_citation(markdown: &str) -> Result<(), AppError> {
    let mut missing: Vec<usize> = Vec::new();
    // Split on blank lines; each paragraph must cite.
    let paras = markdown.split("\n\n").collect::<Vec<_>>();
    for (idx, p) in paras.iter().enumerate() {
        let t = p.trim();
        if t.is_empty() {
            continue;
        }
        if !t.contains("[[chunk:") {
            missing.push(idx + 1);
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    Err(AppError::new(
        "AI_CITATION_REQUIRED",
        "Each paragraph must include at least one citation marker",
    )
    .with_details(format!("missing_citation_paragraphs={missing:?}")))
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

fn compute_model_params_hash(model: &str) -> Result<String, AppError> {
    // Minimal stable hash of the generation parameters that affect output.
    // Currently: model + stream=false (no other params exposed yet).
    #[derive(Serialize)]
    struct Params<'a> {
        v: u32,
        api: &'a str,
        model: &'a str,
        stream: bool,
    }
    let p = Params {
        v: 1,
        api: "/api/generate",
        model,
        stream: false,
    };
    let json = serde_json::to_string(&p).map_err(|e| {
        AppError::new("AI_DRAFT_FAILED", "Failed to encode model params for hashing")
            .with_details(e.to_string())
    })?;
    let digest = Sha256::digest(json.as_bytes());
    Ok(hex::encode(digest))
}
