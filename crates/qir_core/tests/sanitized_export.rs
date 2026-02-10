use std::fs;
use std::path::Path;

use pretty_assertions::assert_eq;
use tempfile::tempdir;

use qir_core::db;
use qir_core::ingest::slack_transcript::ingest_slack_transcript_text;
use qir_core::sanitize::export_sanitized_dataset;

fn read_dir_text(dir: &Path) -> String {
    let mut out = String::new();
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_file() {
            out.push_str(&fs::read_to_string(&p).unwrap_or_default());
        }
    }
    out
}

#[test]
fn sanitized_export_does_not_include_raw_slack_text_and_is_deterministic() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("db.sqlite");
    let mut conn = db::open(&db_path).expect("open");
    db::migrate(&mut conn).expect("migrate");

    // Seed Slack content with a unique marker we can search for.
    let sensitive = "SENSITIVE_DO_NOT_LEAK_12345";
    let transcript = format!("2026-01-01T00:00:00Z alice: hello {sensitive}\n");
    let _summary = ingest_slack_transcript_text(&mut conn, None, Some("Slack-only incident"), &transcript)
        .expect("ingest");

    // Export twice to different destination roots but with the same export_time/app_version
    // so the output directory name and manifests are deterministic.
    let dest1 = tempdir().unwrap();
    let dest2 = tempdir().unwrap();

    let export_time = "2026-02-10T03:00:00Z";
    let app_version = "0.1.0-test";

    let r1 = export_sanitized_dataset(&conn, dest1.path(), export_time, app_version).expect("export1");
    let r2 = export_sanitized_dataset(&conn, dest2.path(), export_time, app_version).expect("export2");

    let text1 = read_dir_text(Path::new(&r1.export_dir));
    assert!(
        !text1.contains(sensitive),
        "sanitized export must not contain raw Slack text"
    );

    let incidents1 = fs::read_to_string(Path::new(&r1.export_dir).join("incidents.json")).unwrap();
    let incidents2 = fs::read_to_string(Path::new(&r2.export_dir).join("incidents.json")).unwrap();
    assert_eq!(incidents1, incidents2);

    let events1 = fs::read_to_string(Path::new(&r1.export_dir).join("timeline_events.json")).unwrap();
    let events2 = fs::read_to_string(Path::new(&r2.export_dir).join("timeline_events.json")).unwrap();
    assert_eq!(events1, events2);

    let warnings1 = fs::read_to_string(Path::new(&r1.export_dir).join("warnings.json")).unwrap();
    let warnings2 = fs::read_to_string(Path::new(&r2.export_dir).join("warnings.json")).unwrap();
    assert_eq!(warnings1, warnings2);
}

