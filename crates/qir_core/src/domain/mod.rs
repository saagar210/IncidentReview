use serde::{Deserialize, Serialize};

/// Canonical incident representation used by deterministic metrics and report generation.
///
/// Notes:
/// - Canonical timestamps are nullable RFC3339 UTC strings.
/// - When a non-RFC3339 timestamp is provided during ingest, the original value is preserved in
///   `*_ts_raw` and validators surface warnings (no silent guessing or defaults).
/// - Unknown values remain `None` and should surface as validation warnings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Incident {
    pub id: i64,
    pub external_id: Option<String>,
    pub fingerprint: String,
    pub title: String,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub detection_source: Option<String>,
    pub vendor: Option<String>,
    pub service: Option<String>,
    pub impact_pct: Option<i64>,
    pub service_health_pct: Option<i64>,

    pub start_ts: Option<String>,
    pub first_observed_ts: Option<String>,
    pub it_awareness_ts: Option<String>,
    pub ack_ts: Option<String>,
    pub mitigate_ts: Option<String>,
    pub resolve_ts: Option<String>,

    pub start_ts_raw: Option<String>,
    pub first_observed_ts_raw: Option<String>,
    pub it_awareness_ts_raw: Option<String>,
    pub ack_ts_raw: Option<String>,
    pub mitigate_ts_raw: Option<String>,
    pub resolve_ts_raw: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl ValidationWarning {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}
