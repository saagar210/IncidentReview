use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::domain::{Incident, ValidationWarning};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct IncidentMetrics {
    pub mttd_seconds: Option<i64>,
    pub it_awareness_lag_seconds: Option<i64>,
    pub mtta_seconds: Option<i64>,
    pub time_to_mitigation_seconds: Option<i64>,
    pub mttr_seconds: Option<i64>,
}

fn parse_ts(
    field: &str,
    raw: &Option<String>,
    warnings: &mut Vec<ValidationWarning>,
) -> Option<OffsetDateTime> {
    let Some(s) = raw.as_deref() else { return None };
    match OffsetDateTime::parse(s, &Rfc3339) {
        Ok(dt) => Some(dt),
        Err(e) => {
            warnings.push(
                ValidationWarning::new(
                    "METRICS_TS_PARSE_FAILED",
                    format!("Failed to parse {field} for metrics"),
                )
                .with_details(format!("value={s}; err={e}")),
            );
            None
        }
    }
}

fn diff_seconds(a: OffsetDateTime, b: OffsetDateTime) -> Option<i64> {
    let dur = b - a;
    let secs = dur.whole_seconds();
    if secs < 0 {
        None
    } else {
        Some(secs)
    }
}

fn compute_pair(
    a_field: &str,
    a: Option<OffsetDateTime>,
    b_field: &str,
    b: Option<OffsetDateTime>,
    warnings: &mut Vec<ValidationWarning>,
) -> Option<i64> {
    let (Some(a), Some(b)) = (a, b) else {
        return None;
    };
    match diff_seconds(a, b) {
        Some(s) => Some(s),
        None => {
            warnings.push(
                ValidationWarning::new(
                    "METRICS_TS_ORDER_VIOLATION",
                    format!("Cannot compute metric: {a_field} must be <= {b_field}"),
                )
                .with_details(format!("{a_field}={a}; {b_field}={b}")),
            );
            None
        }
    }
}

/// Compute deterministic per-incident metrics.
///
/// Metrics are computed only when the relevant timestamps are present and parseable.
/// Ordering violations produce warnings and yield `None` metrics (no silent correction).
pub fn compute_incident_metrics(incident: &Incident) -> (IncidentMetrics, Vec<ValidationWarning>) {
    let mut warnings = Vec::new();

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

    let mttd_seconds = compute_pair(
        "start_ts",
        start,
        "first_observed_ts",
        first_observed,
        &mut warnings,
    );
    let it_awareness_lag_seconds = compute_pair(
        "first_observed_ts",
        first_observed,
        "it_awareness_ts",
        it_awareness,
        &mut warnings,
    );
    let mtta_seconds = compute_pair(
        "it_awareness_ts",
        it_awareness,
        "ack_ts",
        ack,
        &mut warnings,
    );
    let time_to_mitigation_seconds =
        compute_pair("ack_ts", ack, "mitigate_ts", mitigate, &mut warnings);

    // MTTR: start -> resolve when start present, else first_observed -> resolve if present.
    let mttr_seconds = match (start, first_observed, resolve) {
        (Some(s), _, Some(r)) => {
            compute_pair("start_ts", Some(s), "resolve_ts", Some(r), &mut warnings)
        }
        (None, Some(o), Some(r)) => compute_pair(
            "first_observed_ts",
            Some(o),
            "resolve_ts",
            Some(r),
            &mut warnings,
        ),
        _ => None,
    };

    (
        IncidentMetrics {
            mttd_seconds,
            it_awareness_lag_seconds,
            mtta_seconds,
            time_to_mitigation_seconds,
            mttr_seconds,
        },
        warnings,
    )
}
