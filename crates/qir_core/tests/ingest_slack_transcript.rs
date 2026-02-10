use qir_core::db;
use qir_core::ingest::slack_transcript::{
    ingest_slack_transcript_text, preview_slack_transcript_text,
};

#[test]
fn ingests_line_oriented_slack_transcript_with_warnings_and_attaches_to_incident() {
    let text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/demo/slack_sample.txt"
    ));

    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let prev = preview_slack_transcript_text(text);
    assert_eq!(prev.detected_format, "line_rfc3339");

    let summary =
        ingest_slack_transcript_text(&mut conn, None, Some("Slack Shell"), text).expect("ingest");
    assert!(summary.incident_created);
    assert_eq!(summary.detected_format, "line_rfc3339");
    assert_eq!(summary.inserted_events, 3);
    assert!(
        summary
            .warnings
            .iter()
            .any(|w| w.code == "INGEST_SLACK_TS_UNKNOWN"),
        "expected missing timestamp warning"
    );

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM timeline_events", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 3);

    let attached_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM timeline_events WHERE incident_id = ?1",
            [summary.incident_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(attached_count, 3);
}

#[test]
fn ingests_slack_json_export_and_normalizes_timestamps() {
    let text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/demo/slack_export_sample.json"
    ));

    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let prev = preview_slack_transcript_text(text);
    assert_eq!(prev.detected_format, "slack_json_export");

    let summary = ingest_slack_transcript_text(&mut conn, None, Some("Slack JSON Shell"), text)
        .expect("ingest");
    assert_eq!(summary.detected_format, "slack_json_export");
    assert_eq!(summary.inserted_events, 2);
    assert!(
        summary.warnings.is_empty(),
        "expected no warnings: {:?}",
        summary.warnings
    );

    let ts_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM timeline_events WHERE incident_id = ?1 AND ts IS NOT NULL",
            [summary.incident_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(ts_count, 2);
}

#[test]
fn ingests_unknown_slack_format_as_raw_lines_with_warnings() {
    let text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/demo/slack_paste_sample.txt"
    ));

    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let prev = preview_slack_transcript_text(text);
    assert_eq!(prev.detected_format, "raw_lines");
    assert!(prev
        .warnings
        .iter()
        .any(|w| w.code == "INGEST_SLACK_FORMAT_UNKNOWN"));

    let summary = ingest_slack_transcript_text(&mut conn, None, Some("Slack Paste Shell"), text)
        .expect("ingest");
    assert_eq!(summary.detected_format, "raw_lines");
    assert_eq!(summary.inserted_events, 2);
    assert!(summary
        .warnings
        .iter()
        .any(|w| w.code == "INGEST_SLACK_FORMAT_UNKNOWN"));

    let null_ts_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM timeline_events WHERE incident_id = ?1 AND ts IS NULL",
            [summary.incident_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(null_ts_count, 2);
}
