use std::collections::BTreeSet;

use qir_core::analytics::build_dashboard_payload_v2;
use qir_core::db;
use qir_core::ingest::jira_csv::{ingest_jira_csv, JiraCsvMapping};

fn story_mapping() -> JiraCsvMapping {
    JiraCsvMapping {
        external_id: Some("Key".to_string()),
        title: "Summary".to_string(),
        description: Some("Description".to_string()),
        severity: Some("Severity".to_string()),
        detection_source: Some("DetectionSource".to_string()),
        vendor: Some("Vendor".to_string()),
        service: Some("Service".to_string()),
        impact_pct: Some("ImpactPct".to_string()),
        service_health_pct: Some("ServiceHealthPct".to_string()),
        start_ts: Some("StartTs".to_string()),
        first_observed_ts: Some("FirstObservedTs".to_string()),
        it_awareness_ts: Some("ItAwarenessTs".to_string()),
        ack_ts: Some("AckTs".to_string()),
        mitigate_ts: Some("MitigateTs".to_string()),
        resolve_ts: Some("ResolveTs".to_string()),
    }
}

fn assert_reconciles_to_total(buckets: Vec<(i64, Vec<i64>)>, total: i64) {
    let sum: i64 = buckets.iter().map(|(c, _)| *c).sum();
    assert_eq!(sum, total, "expected bucket counts to reconcile to total");

    let mut all = BTreeSet::new();
    for (_, ids) in buckets {
        for id in ids {
            all.insert(id);
        }
    }
    assert_eq!(
        all.len() as i64,
        total,
        "expected union of incident_ids == total"
    );
}

#[test]
fn dashboard_v2_buckets_reconcile_and_support_drilldown() {
    let csv_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/demo/jira_story.csv"
    ));

    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");
    ingest_jira_csv(&mut conn, csv_text, &story_mapping()).expect("ingest");

    let dash = build_dashboard_payload_v2(&conn).expect("dash");
    assert_eq!(dash.version, 2);
    assert_eq!(dash.incident_count, 6);

    assert_reconciles_to_total(
        dash.detection_story
            .detection_source_mix
            .iter()
            .map(|b| (b.count, b.incident_ids.clone()))
            .collect(),
        dash.incident_count,
    );

    assert_reconciles_to_total(
        dash.detection_story
            .it_awareness_lag_buckets
            .iter()
            .map(|b| (b.count, b.incident_ids.clone()))
            .collect(),
        dash.incident_count,
    );

    assert_reconciles_to_total(
        dash.response_story
            .time_to_mitigation_buckets
            .iter()
            .map(|b| (b.count, b.incident_ids.clone()))
            .collect(),
        dash.incident_count,
    );

    assert_reconciles_to_total(
        dash.response_story
            .time_to_resolve_buckets
            .iter()
            .map(|b| (b.count, b.incident_ids.clone()))
            .collect(),
        dash.incident_count,
    );

    assert_reconciles_to_total(
        dash.vendor_service_story
            .top_vendors_by_count
            .iter()
            .map(|b| (b.count, b.incident_ids.clone()))
            .collect(),
        dash.incident_count,
    );

    assert_reconciles_to_total(
        dash.vendor_service_story
            .top_services_by_count
            .iter()
            .map(|b| (b.count, b.incident_ids.clone()))
            .collect(),
        dash.incident_count,
    );

    // Pain buckets must still cover all incidents deterministically (count/id reconciliation),
    // even though pain may be unknown for some incidents.
    assert_reconciles_to_total(
        dash.vendor_service_story
            .top_vendors_by_pain
            .iter()
            .map(|b| (b.count, b.incident_ids.clone()))
            .collect(),
        dash.incident_count,
    );
    assert_reconciles_to_total(
        dash.vendor_service_story
            .top_services_by_pain
            .iter()
            .map(|b| (b.count, b.incident_ids.clone()))
            .collect(),
        dash.incident_count,
    );
}
