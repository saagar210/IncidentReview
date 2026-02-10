use time::format_description::well_known::Rfc3339;
use time::{format_description, OffsetDateTime, PrimitiveDateTime, UtcOffset};

use crate::domain::ValidationWarning;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedTimestamp {
    /// Canonical RFC3339 UTC string, if deterministically parseable.
    pub canonical_rfc3339_utc: Option<String>,
    /// Raw input preserved for non-RFC3339 (or unparseable) inputs.
    pub raw: Option<String>,
}

fn canonicalize_rfc3339_utc(dt: OffsetDateTime) -> Option<String> {
    let utc = dt.to_offset(UtcOffset::UTC);
    utc.format(&Rfc3339).ok()
}

fn parse_primitive_assume_utc(
    raw: &str,
    fmt: &str,
    field: &str,
    warnings: &mut Vec<ValidationWarning>,
) -> Option<String> {
    let items = match format_description::parse(fmt) {
        Ok(i) => i,
        Err(e) => {
            warnings.push(
                ValidationWarning::new(
                    "INGEST_TS_FORMAT_CONFIG_FAILED",
                    format!("Timestamp format config error for {field}"),
                )
                .with_details(format!("fmt={fmt}; err={e}")),
            );
            return None;
        }
    };

    let pdt = match PrimitiveDateTime::parse(raw, &items) {
        Ok(p) => p,
        Err(_) => return None,
    };

    // This format is missing timezone. We assume UTC deterministically but MUST warn explicitly.
    warnings.push(
        ValidationWarning::new(
            "INGEST_TS_TZ_ASSUMED_UTC",
            format!("Assumed UTC timezone for {field}"),
        )
        .with_details(format!("value={raw}; fmt={fmt}")),
    );

    canonicalize_rfc3339_utc(pdt.assume_utc())
}

fn parse_allowlist(
    raw: &str,
    field: &str,
    warnings: &mut Vec<ValidationWarning>,
) -> Option<String> {
    // Deterministic allowlist only (no fuzzy parsing).
    //
    // Notes:
    // - We prefer formats that explicitly include a timezone.
    // - For formats without timezone, we assume UTC but emit an explicit warning.

    // ISO-like without timezone (assume UTC, warn).
    for fmt in [
        "[year]-[month]-[day] [hour]:[minute]:[second]",
        "[year]-[month]-[day] [hour]:[minute]",
        "[year]-[month]-[day]T[hour]:[minute]:[second]",
        "[year]-[month]-[day]T[hour]:[minute]",
    ] {
        if let Some(canon) = parse_primitive_assume_utc(raw, fmt, field, warnings) {
            return Some(canon);
        }
    }

    None
}

/// Normalize a user-provided timestamp into canonical RFC3339 UTC while preserving raw inputs.
///
/// Contract:
/// - If `raw_input` is RFC3339 parseable, we store canonical only and return `raw=None`.
/// - If `raw_input` is non-RFC3339 but deterministically parseable via allowlist,
///   we store canonical and preserve `raw` with an explicit warning.
/// - If unparseable, we preserve `raw`, keep canonical `None`, and emit an explicit warning.
pub fn normalize_timestamp(
    field: &str,
    raw_input: &str,
    warnings: &mut Vec<ValidationWarning>,
) -> NormalizedTimestamp {
    let trimmed = raw_input.trim();
    if trimmed.is_empty() {
        return NormalizedTimestamp {
            canonical_rfc3339_utc: None,
            raw: None,
        };
    }

    if let Ok(dt) = OffsetDateTime::parse(trimmed, &Rfc3339) {
        return NormalizedTimestamp {
            canonical_rfc3339_utc: canonicalize_rfc3339_utc(dt),
            raw: None,
        };
    }

    if let Some(canon) = parse_allowlist(trimmed, field, warnings) {
        warnings.push(
            ValidationWarning::new(
                "INGEST_TS_NORMALIZED",
                format!("Normalized non-RFC3339 timestamp for {field}"),
            )
            .with_details(format!("raw={trimmed}; canonical={canon}")),
        );
        return NormalizedTimestamp {
            canonical_rfc3339_utc: Some(canon),
            raw: Some(trimmed.to_string()),
        };
    }

    warnings.push(
        ValidationWarning::new(
            "INGEST_TS_UNPARSEABLE",
            format!("Unparseable timestamp for {field}; preserved raw"),
        )
        .with_details(format!("raw={trimmed}")),
    );

    NormalizedTimestamp {
        canonical_rfc3339_utc: None,
        raw: Some(trimmed.to_string()),
    }
}
