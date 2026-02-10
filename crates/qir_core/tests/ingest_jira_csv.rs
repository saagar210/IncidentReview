use qir_core::db;
use qir_core::ingest::jira_csv::{ingest_jira_csv, JiraCsvMapping};

#[test]
fn ingests_sample_jira_csv_into_db() {
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

    let summary = ingest_jira_csv(&mut conn, csv_text, &mapping).expect("ingest");
    assert_eq!(summary.inserted, 2);
    assert!(summary.warnings.is_empty());

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM incidents", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 2);
}
