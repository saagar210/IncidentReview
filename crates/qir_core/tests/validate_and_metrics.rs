use qir_core::domain::Incident;
use qir_core::metrics::compute_incident_metrics;
use qir_core::validate::validate_incident;

fn incident_with_ts(start: &str, ack: &str, resolve: &str) -> Incident {
    Incident {
        id: 1,
        external_id: Some("INC-1".to_string()),
        fingerprint: "fp".to_string(),
        title: "Test".to_string(),
        description: None,
        severity: None,
        detection_source: None,
        vendor: None,
        service: None,
        impact_pct: None,
        service_health_pct: None,
        start_ts: Some(start.to_string()),
        first_observed_ts: Some(start.to_string()),
        it_awareness_ts: Some(start.to_string()),
        ack_ts: Some(ack.to_string()),
        mitigate_ts: None,
        resolve_ts: Some(resolve.to_string()),
        start_ts_raw: None,
        first_observed_ts_raw: None,
        it_awareness_ts_raw: None,
        ack_ts_raw: None,
        mitigate_ts_raw: None,
        resolve_ts_raw: None,
    }
}

#[test]
fn validator_flags_ordering_violations() {
    let incident = incident_with_ts(
        "2026-01-01T00:00:00Z",
        "2025-12-31T23:59:00Z",
        "2026-01-01T00:10:00Z",
    );
    let warnings = validate_incident(&incident);
    assert!(
        warnings
            .iter()
            .any(|w| w.code == "VALIDATION_TS_ORDER_VIOLATION"),
        "expected ordering warning"
    );
}

#[test]
fn metrics_compute_durations_when_possible() {
    let incident = incident_with_ts(
        "2026-01-01T00:00:00Z",
        "2026-01-01T00:05:00Z",
        "2026-01-01T00:40:00Z",
    );
    let (m, warnings) = compute_incident_metrics(&incident);
    assert!(
        warnings.is_empty(),
        "expected no warnings, got: {warnings:?}"
    );
    assert_eq!(m.mtta_seconds, Some(5 * 60));
    assert_eq!(m.mttr_seconds, Some(40 * 60));
}
