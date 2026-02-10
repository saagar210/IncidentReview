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

pub fn incident_highlights_top_n_prompt(
    quarter_label: &str,
    user_prompt: &str,
    evidence_blocks: &str,
) -> String {
    format!(
        r#"You are drafting the "Incident Highlights" section for a Quarterly Incident Review for quarter "{quarter_label}".

Rules (non-negotiable):
1) Use ONLY the evidence chunks provided below. Do not invent facts.
2) Each bullet MUST include at least one inline citation marker in the form [[chunk:<chunk_id>]] on the same line.
3) Do not compute or infer metrics; treat any metrics in evidence as already computed.
4) If you cannot support a highlight with evidence, omit it entirely.

User prompt:
{user_prompt}

Evidence chunks:
{evidence_blocks}

Output:
- Return Markdown only.
- Use a bulleted list. Each bullet must contain >= 1 citation marker on the same line.
"#
    )
}

pub fn theme_analysis_prompt(
    quarter_label: &str,
    user_prompt: &str,
    evidence_blocks: &str,
) -> String {
    format!(
        r#"You are drafting the "Theme Analysis" section for a Quarterly Incident Review for quarter "{quarter_label}".

Rules (non-negotiable):
1) Use ONLY the evidence chunks provided below. Do not invent facts.
2) Each theme MUST include at least one inline citation marker in the form [[chunk:<chunk_id>]] on the same line.
3) Do not compute or infer metrics; treat any metrics in evidence as already computed.
4) If you cannot support a theme with evidence, omit it entirely.

User prompt:
{user_prompt}

Evidence chunks:
{evidence_blocks}

Output:
- Return Markdown only.
- Use a bulleted list of themes (one theme per bullet). Each bullet must contain >= 1 citation marker on the same line.
"#
    )
}

pub fn action_plan_next_quarter_prompt(
    quarter_label: &str,
    user_prompt: &str,
    evidence_blocks: &str,
) -> String {
    format!(
        r#"You are drafting the "Action Plan (Next Quarter)" section for a Quarterly Incident Review for quarter "{quarter_label}".

Rules (non-negotiable):
1) Use ONLY the evidence chunks provided below. Do not invent facts.
2) Each action MUST include at least one inline citation marker in the form [[chunk:<chunk_id>]] on the same line.
3) Do not compute or infer metrics; treat any metrics in evidence as already computed.
4) If you cannot support an action with evidence, omit it entirely.

User prompt:
{user_prompt}

Evidence chunks:
{evidence_blocks}

Output:
- Return Markdown only.
- Use a bulleted list of actions (one action per bullet). Each bullet must contain >= 1 citation marker on the same line.
"#
    )
}

pub fn quarter_narrative_recap_prompt(
    quarter_label: &str,
    user_prompt: &str,
    evidence_blocks: &str,
) -> String {
    format!(
        r#"You are drafting the "Quarter Narrative Recap" section for a Quarterly Incident Review for quarter "{quarter_label}".

Rules (non-negotiable):
1) Use ONLY the evidence chunks provided below. Do not invent facts.
2) Each paragraph MUST include at least one inline citation marker in the form [[chunk:<chunk_id>]] within that paragraph.
3) Do not compute or infer metrics; treat any metrics in evidence as already computed.
4) If you cannot support a paragraph with evidence, omit it entirely.

User prompt:
{user_prompt}

Evidence chunks:
{evidence_blocks}

Output:
- Return Markdown only.
- Write 2-6 short paragraphs (separated by blank lines). Each paragraph must contain >= 1 citation marker.
"#
    )
}
