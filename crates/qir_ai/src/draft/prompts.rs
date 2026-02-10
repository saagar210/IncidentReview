pub fn exec_summary_prompt(
    quarter_label: &str,
    user_prompt: &str,
    evidence_blocks: &str,
) -> String {
    // Keep the contract explicit:
    // - Use ONLY evidence provided.
    // - Must include inline citations as [[chunk:<chunk_id>]].
    // - If evidence is insufficient, write UNKNOWN.
    format!(
        r#"You are drafting a Quarterly Incident Review executive summary for quarter "{quarter_label}".

Rules (non-negotiable):
1) Use ONLY the evidence chunks provided below. Do not invent facts.
2) Every concrete claim MUST include an inline citation marker in the form [[chunk:<chunk_id>]].
3) If you cannot support a claim with evidence, write UNKNOWN (and do not cite).
4) Do not compute or infer metrics; treat any metrics in evidence as already computed.

User prompt:
{user_prompt}

Evidence chunks:
{evidence_blocks}

Output:
- Return Markdown only.
- Include inline citations as specified.
"#
    )
}

