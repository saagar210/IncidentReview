use qir_core::error::AppError;

/// Enforce that AI outputs include citations referencing evidence chunk IDs.
///
/// Expected citation format: `[[chunk:<id>]]` anywhere in the output.
pub fn enforce_citations(output: &str) -> Result<(), AppError> {
    // Minimal parser: look for token prefix.
    let has_citation = output.contains("[[chunk:");
    if !has_citation {
        return Err(AppError::new(
            "AI_CITATION_REQUIRED",
            "AI output must include evidence chunk citations",
        ));
    }
    Ok(())
}
