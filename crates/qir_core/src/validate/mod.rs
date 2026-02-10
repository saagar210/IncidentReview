use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::domain::{Incident, ValidationWarning};
use crate::error::AppError;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

fn parse_ts(
    field: &str,
    canonical: &Option<String>,
    warnings: &mut Vec<ValidationWarning>,
) -> Option<OffsetDateTime> {
    let Some(s) = canonical.as_deref() else {
        return None;
    };
    match OffsetDateTime::parse(s, &Rfc3339) {
        Ok(dt) => Some(dt),
        Err(e) => {
            warnings.push(
                ValidationWarning::new(
                    "VALIDATION_TS_PARSE_FAILED",
                    format!("Failed to parse {field}"),
                )
                .with_details(format!("value={s}; err={e}")),
            );
            None
        }
    }
}

fn order_check(
    a_field: &str,
    a: Option<OffsetDateTime>,
    b_field: &str,
    b: Option<OffsetDateTime>,
    warnings: &mut Vec<ValidationWarning>,
) {
    let (Some(a), Some(b)) = (a, b) else { return };
    if a > b {
        warnings.push(
            ValidationWarning::new(
                "VALIDATION_TS_ORDER_VIOLATION",
                format!("Timestamp order violation: {a_field} must be <= {b_field}"),
            )
            .with_details(format!("{a_field}={a}; {b_field}={b}")),
        );
    }
}

/// Validate an incident according to repo rules:
/// start <= first_observed <= it_awareness <= ack <= mitigate <= resolve (when present).
pub fn validate_incident(incident: &Incident) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();

    // If raw values exist but canonical is missing, surface that explicitly so the UI can show
    // "provided but unparseable/non-canonical" without guessing.
    for (field, canonical, raw) in [
        ("start_ts", &incident.start_ts, &incident.start_ts_raw),
        (
            "first_observed_ts",
            &incident.first_observed_ts,
            &incident.first_observed_ts_raw,
        ),
        (
            "it_awareness_ts",
            &incident.it_awareness_ts,
            &incident.it_awareness_ts_raw,
        ),
        ("ack_ts", &incident.ack_ts, &incident.ack_ts_raw),
        (
            "mitigate_ts",
            &incident.mitigate_ts,
            &incident.mitigate_ts_raw,
        ),
        ("resolve_ts", &incident.resolve_ts, &incident.resolve_ts_raw),
    ] {
        if canonical.is_none() {
            if let Some(raw) = raw.as_deref() {
                warnings.push(
                    ValidationWarning::new(
                        "VALIDATION_TS_RAW_PRESENT",
                        format!(
                            "Non-canonical timestamp preserved for {field}; canonical is UNKNOWN"
                        ),
                    )
                    .with_details(format!("raw={raw}")),
                );
            }
        }
    }

    let start = parse_ts("start_ts", &incident.start_ts, &mut warnings);
    let first_observed = parse_ts(
        "first_observed_ts",
        &incident.first_observed_ts,
        &mut warnings,
    );
    let it_awareness = parse_ts("it_awareness_ts", &incident.it_awareness_ts, &mut warnings);
    let ack = parse_ts("ack_ts", &incident.ack_ts, &mut warnings);
    let mitigate = parse_ts("mitigate_ts", &incident.mitigate_ts, &mut warnings);
    let resolve = parse_ts("resolve_ts", &incident.resolve_ts, &mut warnings);

    order_check(
        "start_ts",
        start,
        "first_observed_ts",
        first_observed,
        &mut warnings,
    );
    order_check(
        "first_observed_ts",
        first_observed,
        "it_awareness_ts",
        it_awareness,
        &mut warnings,
    );
    order_check(
        "it_awareness_ts",
        it_awareness,
        "ack_ts",
        ack,
        &mut warnings,
    );
    order_check("ack_ts", ack, "mitigate_ts", mitigate, &mut warnings);
    order_check(
        "mitigate_ts",
        mitigate,
        "resolve_ts",
        resolve,
        &mut warnings,
    );

    // Percent fields: nullable 0..100 (warnings instead of silent defaults).
    for (field, v) in [
        ("impact_pct", incident.impact_pct),
        ("service_health_pct", incident.service_health_pct),
    ] {
        if let Some(v) = v {
            if !(0..=100).contains(&v) {
                warnings.push(
                    ValidationWarning::new(
                        "VALIDATION_PCT_OUT_OF_RANGE",
                        format!("{field} out of range"),
                    )
                    .with_details(format!("value={v}")),
                );
            }
        }
    }

    warnings
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncidentValidationReportItem {
    pub id: i64,
    pub external_id: Option<String>,
    pub title: String,
    pub warnings: Vec<ValidationWarning>,
}

pub fn validate_all_incidents(
    conn: &Connection,
) -> Result<Vec<IncidentValidationReportItem>, AppError> {
    let incidents = crate::repo::list_incidents(conn)?;
    let mut out = Vec::new();

    for inc in incidents {
        let warnings = validate_incident(&inc);
        out.push(IncidentValidationReportItem {
            id: inc.id,
            external_id: inc.external_id,
            title: inc.title,
            warnings,
        });
    }

    // Deterministic ordering.
    out.sort_by(|a, b| {
        (
            a.external_id.clone().unwrap_or_default(),
            a.title.clone(),
            a.id,
        )
            .cmp(&(
                b.external_id.clone().unwrap_or_default(),
                b.title.clone(),
                b.id,
            ))
    });

    Ok(out)
}
