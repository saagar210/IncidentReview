use std::fs;
use std::path::Path;

use tempfile::tempdir;

use qir_core::analytics::build_dashboard_payload_v2;
use qir_core::db;
use qir_core::demo::seed_demo_dataset;
use qir_core::ingest::slack_transcript::ingest_slack_transcript_text;
use qir_core::report::generate_qir_markdown;
use qir_core::sanitize::{export_sanitized_dataset, import_sanitized_dataset, inspect_sanitized_dataset};
use qir_core::validate::validate_all_incidents;

fn sum_counts(v: &[qir_core::analytics::SeverityCount]) -> i64 {
    v.iter().map(|b| b.count).sum()
}

#[test]
fn sanitized_export_import_round_trip_preserves_dashboard_and_report() {
    let tmp = tempdir().unwrap();

    // DB A: seed demo dataset and add a Slack event containing a sensitive marker.
    let db_a_path = tmp.path().join("a.sqlite");
    let mut conn_a = db::open(&db_a_path).expect("open a");
    db::migrate(&mut conn_a).expect("migrate a");

    let seed = seed_demo_dataset(&mut conn_a).expect("seed demo");
    assert!(seed.inserted > 0);

    let incidents = qir_core::repo::list_incidents(&conn_a).expect("list incidents");
    let first_id = incidents.first().map(|i| i.id).expect("has incidents");

    let sensitive = "SENSITIVE_DO_NOT_LEAK_67890";
    let transcript = format!("2026-01-01T00:00:00Z alice: hello {sensitive}\n");
    let _slack = ingest_slack_transcript_text(&mut conn_a, Some(first_id), None, &transcript)
        .expect("slack ingest");

    let dash_a = build_dashboard_payload_v2(&conn_a).expect("dashboard a");
    assert_eq!(dash_a.incident_count, seed.inserted as i64);
    assert_eq!(sum_counts(&dash_a.severity_counts), dash_a.incident_count);

    let report_a = generate_qir_markdown(&conn_a).expect("report a");
    assert!(report_a.contains("# Quarterly Incident Review (QIR)"));
    assert!(!report_a.contains(sensitive));

    // Export sanitized dataset deterministically (fixed export_time/app_version).
    let dest = tempdir().unwrap();
    let export_time = "2026-02-10T03:00:00Z";
    let app_version = "0.1.0-test";
    let exported = export_sanitized_dataset(&conn_a, dest.path(), export_time, app_version)
        .expect("export sanitized");

    // Inspect should validate hashes and sizes.
    let _manifest = inspect_sanitized_dataset(Path::new(&exported.export_dir)).expect("inspect");

    let exported_text = {
        let mut out = String::new();
        for name in ["incidents.json", "timeline_events.json", "warnings.json", "sanitized_manifest.json"] {
            let p = Path::new(&exported.export_dir).join(name);
            if p.is_file() {
                out.push_str(&fs::read_to_string(p).unwrap_or_default());
            }
        }
        out
    };
    assert!(
        !exported_text.contains(sensitive),
        "sanitized export must not contain raw Slack text"
    );

    // DB B: import into a fresh DB and ensure dashboards/report/validation succeed.
    let db_b_path = tmp.path().join("b.sqlite");
    let mut conn_b = db::open(&db_b_path).expect("open b");
    db::migrate(&mut conn_b).expect("migrate b");

    let import = import_sanitized_dataset(&mut conn_b, Path::new(&exported.export_dir))
        .expect("import sanitized");
    assert_eq!(import.inserted_incidents, seed.inserted as i64);

    let dash_b = build_dashboard_payload_v2(&conn_b).expect("dashboard b");
    assert_eq!(dash_b.incident_count, dash_a.incident_count);
    assert_eq!(sum_counts(&dash_b.severity_counts), dash_b.incident_count);

    // Validation and report generation must remain functional.
    let _validation = validate_all_incidents(&conn_b).expect("validate b");
    let report_b = generate_qir_markdown(&conn_b).expect("report b");
    assert!(report_b.contains("# Quarterly Incident Review (QIR)"));

    // Defense-in-depth: ensure the sensitive marker did not reappear via import.
    assert!(!report_b.contains(sensitive));
    let db_b_text: String = conn_b
        .prepare("SELECT text, COALESCE(raw_json,'') FROM timeline_events ORDER BY id ASC")
        .unwrap()
        .query_map([], |row| {
            let t: String = row.get(0)?;
            let r: String = row.get(1)?;
            Ok(format!("{t}\n{r}\n"))
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    assert!(
        !db_b_text.contains(sensitive),
        "imported DB must not contain raw Slack text"
    );
}

