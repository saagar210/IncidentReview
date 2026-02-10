use std::path::Path;

use pretty_assertions::assert_eq;
use rusqlite::Connection;
use tempfile::tempdir;

use qir_core::backup::{create_backup, read_manifest, restore_from_backup};
use qir_core::db;
use qir_core::ingest::jira_csv::{import_jira_csv, JiraCsvMapping};

fn seed_db(db_path: &Path) -> Connection {
    let mut conn = db::open(db_path).expect("open");
    db::migrate(&mut conn).expect("migrate");

    let csv_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/demo/jira_story.csv"
    ));

    let mapping = JiraCsvMapping {
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
    };

    import_jira_csv(&mut conn, csv_text, &mapping).expect("import");
    conn
}

#[test]
fn backup_creates_expected_folder_contents_and_manifest() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("source.sqlite");
    let conn = seed_db(&db_path);

    let dest_root = tmp.path().join("backups");
    std::fs::create_dir_all(&dest_root).unwrap();

    let res = create_backup(
        &conn,
        &db_path,
        None,
        &dest_root,
        "2026-02-10T01:00:00Z",
        "0.1.0-test",
    )
    .expect("backup");

    let backup_dir = Path::new(&res.backup_dir);
    assert!(backup_dir.is_dir());

    let manifest_path = backup_dir.join("manifest.json");
    let db_copy_path = backup_dir.join("incidentreview.sqlite");
    assert!(manifest_path.is_file());
    assert!(db_copy_path.is_file());

    let manifest = read_manifest(backup_dir).expect("read manifest");
    assert_eq!(manifest.app_version, "0.1.0-test");
    assert_eq!(manifest.export_time, "2026-02-10T01:00:00Z");
    assert_eq!(manifest.db.filename, "incidentreview.sqlite");
    assert!(manifest.db.sha256.len() >= 64);

    let conn2 = db::open(&db_copy_path).expect("open copied");
    let count: i64 = conn2
        .query_row("SELECT COUNT(*) FROM incidents", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, manifest.counts.incidents);
}

#[test]
fn restore_rehydrates_db_when_overwrite_is_allowed() {
    let tmp = tempdir().unwrap();
    let source_db_path = tmp.path().join("source.sqlite");
    let conn = seed_db(&source_db_path);

    let dest_root = tmp.path().join("backups");
    std::fs::create_dir_all(&dest_root).unwrap();
    let backup = create_backup(
        &conn,
        &source_db_path,
        None,
        &dest_root,
        "2026-02-10T02:00:00Z",
        "0.1.0-test",
    )
    .expect("backup");

    // Create a different target DB to ensure we actually overwrite.
    let target_db_path = tmp.path().join("target.sqlite");
    {
        let mut c = db::open(&target_db_path).expect("open target");
        db::migrate(&mut c).expect("migrate");
        let csv_text = "Key,Summary\nIR-FAKE,fake incident\n";
        let mapping = JiraCsvMapping {
            external_id: Some("Key".to_string()),
            title: "Summary".to_string(),
            description: None,
            severity: None,
            detection_source: None,
            vendor: None,
            service: None,
            impact_pct: None,
            service_health_pct: None,
            start_ts: None,
            first_observed_ts: None,
            it_awareness_ts: None,
            ack_ts: None,
            mitigate_ts: None,
            resolve_ts: None,
        };
        import_jira_csv(&mut c, csv_text, &mapping).expect("seed target");
    }

    restore_from_backup(
        Path::new(&backup.backup_dir),
        &target_db_path,
        None,
        true,
    )
    .expect("restore");

    let conn3 = db::open(&target_db_path).expect("open restored");
    let count: i64 = conn3
        .query_row("SELECT COUNT(*) FROM incidents", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, qir_core::backup::read_manifest(Path::new(&backup.backup_dir)).unwrap().counts.incidents);
}
