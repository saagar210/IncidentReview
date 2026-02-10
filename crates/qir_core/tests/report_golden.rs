use qir_core::analytics::build_dashboard_payload_v2;
use qir_core::db;
use qir_core::ingest::jira_csv::{ingest_jira_csv, JiraCsvMapping};
use qir_core::report::generate_qir_markdown;

#[test]
fn dashboard_payload_reconciles_to_incident_total() {
    let csv_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/demo/jira_sample.csv"
    ));

    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let mapping = JiraCsvMapping {
        external_id: Some("Key".to_string()),
        title: "Summary".to_string(),
        description: Some("Description".to_string()),
        severity: Some("Severity".to_string()),
        detection_source: None,
        vendor: None,
        service: None,
        impact_pct: Some("ImpactPct".to_string()),
        service_health_pct: Some("ServiceHealthPct".to_string()),
        start_ts: Some("StartTs".to_string()),
        first_observed_ts: None,
        it_awareness_ts: None,
        ack_ts: Some("AckTs".to_string()),
        mitigate_ts: None,
        resolve_ts: Some("ResolveTs".to_string()),
    };

    ingest_jira_csv(&mut conn, csv_text, &mapping).expect("ingest");
    let dash = build_dashboard_payload_v2(&conn).expect("dash");

    let sum: i64 = dash.severity_counts.iter().map(|s| s.count).sum();
    assert_eq!(sum, dash.incident_count);

    let sum_det: i64 = dash
        .detection_story
        .detection_source_mix
        .iter()
        .map(|b| b.count)
        .sum();
    assert_eq!(sum_det, dash.incident_count);

    let sum_lag: i64 = dash
        .detection_story
        .it_awareness_lag_buckets
        .iter()
        .map(|b| b.count)
        .sum();
    assert_eq!(sum_lag, dash.incident_count);

    let sum_ttm: i64 = dash
        .response_story
        .time_to_mitigation_buckets
        .iter()
        .map(|b| b.count)
        .sum();
    assert_eq!(sum_ttm, dash.incident_count);

    let sum_ttr: i64 = dash
        .response_story
        .time_to_resolve_buckets
        .iter()
        .map(|b| b.count)
        .sum();
    assert_eq!(sum_ttr, dash.incident_count);

    let sum_vendor_count: i64 = dash
        .vendor_service_story
        .top_vendors_by_count
        .iter()
        .map(|b| b.count)
        .sum();
    assert_eq!(sum_vendor_count, dash.incident_count);

    let sum_service_count: i64 = dash
        .vendor_service_story
        .top_services_by_count
        .iter()
        .map(|b| b.count)
        .sum();
    assert_eq!(sum_service_count, dash.incident_count);

    let sum_vendor_pain: i64 = dash
        .vendor_service_story
        .top_vendors_by_pain
        .iter()
        .map(|b| b.count)
        .sum();
    assert_eq!(sum_vendor_pain, dash.incident_count);

    let sum_service_pain: i64 = dash
        .vendor_service_story
        .top_services_by_pain
        .iter()
        .map(|b| b.count)
        .sum();
    assert_eq!(sum_service_pain, dash.incident_count);
}

#[test]
fn report_matches_golden_fixture() {
    let csv_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/demo/jira_sample.csv"
    ));
    let golden = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/golden/qir_report_demo.md"
    ));

    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let mapping = JiraCsvMapping {
        external_id: Some("Key".to_string()),
        title: "Summary".to_string(),
        description: Some("Description".to_string()),
        severity: Some("Severity".to_string()),
        detection_source: None,
        vendor: None,
        service: None,
        impact_pct: Some("ImpactPct".to_string()),
        service_health_pct: Some("ServiceHealthPct".to_string()),
        start_ts: Some("StartTs".to_string()),
        first_observed_ts: None,
        it_awareness_ts: None,
        ack_ts: Some("AckTs".to_string()),
        mitigate_ts: None,
        resolve_ts: Some("ResolveTs".to_string()),
    };

    ingest_jira_csv(&mut conn, csv_text, &mapping).expect("ingest");

    let md = generate_qir_markdown(&conn).expect("report");
    assert_eq!(md, golden);
}
