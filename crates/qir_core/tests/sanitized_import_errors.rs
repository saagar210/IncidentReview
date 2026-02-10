use std::fs;
use std::path::Path;

use tempfile::tempdir;

use qir_core::db;
use qir_core::demo::seed_demo_dataset;
use qir_core::sanitize::{export_sanitized_dataset, import_sanitized_dataset, inspect_sanitized_dataset};
use qir_core::sanitize::SanitizedExportManifest;

fn write_manifest(dir: &Path, manifest: &SanitizedExportManifest) {
    let p = dir.join("sanitized_manifest.json");
    let json = serde_json::to_string_pretty(manifest).unwrap();
    fs::write(p, json.as_bytes()).unwrap();
}

#[test]
fn import_refuses_non_empty_db_with_stable_code() {
    let tmp = tempdir().unwrap();

    // Create a source dataset via export.
    let db_a_path = tmp.path().join("a.sqlite");
    let mut conn_a = db::open(&db_a_path).expect("open a");
    db::migrate(&mut conn_a).expect("migrate a");
    seed_demo_dataset(&mut conn_a).expect("seed");

    let dest = tempdir().unwrap();
    let export_time = "2026-02-10T03:00:00Z";
    let app_version = "0.1.0-test";
    let exported = export_sanitized_dataset(&conn_a, dest.path(), export_time, app_version).expect("export");

    // Create a target DB that is non-empty.
    let db_b_path = tmp.path().join("b.sqlite");
    let mut conn_b = db::open(&db_b_path).expect("open b");
    db::migrate(&mut conn_b).expect("migrate b");
    conn_b
        .execute(
            r#"
            INSERT INTO incidents(
              external_id, fingerprint, title, description, severity,
              detection_source, vendor, service,
              impact_pct, service_health_pct,
              start_ts, first_observed_ts, it_awareness_ts, ack_ts, mitigate_ts, resolve_ts,
              start_ts_raw, first_observed_ts_raw, it_awareness_ts_raw, ack_ts_raw, mitigate_ts_raw, resolve_ts_raw,
              ingested_at
            ) VALUES (
              NULL, 'fp', 'some title', NULL, NULL,
              NULL, NULL, NULL,
              NULL, NULL,
              NULL, NULL, NULL, NULL, NULL, NULL,
              NULL, NULL, NULL, NULL, NULL, NULL,
              ?1
            )
            "#,
            [export_time],
        )
        .expect("insert incident");

    let err = import_sanitized_dataset(&mut conn_b, Path::new(&exported.export_dir))
        .expect_err("must refuse non-empty db");
    assert_eq!(err.code, "INGEST_SANITIZED_DB_NOT_EMPTY");
}

#[test]
fn inspect_fails_on_manifest_version_mismatch_with_stable_code() {
    let tmp = tempdir().unwrap();
    let dir = tmp.path();
    let manifest = SanitizedExportManifest {
        manifest_version: 2,
        app_version: "0.1.0-test".to_string(),
        export_time: "2026-02-10T03:00:00Z".to_string(),
        incident_count: 0,
        files: Vec::new(),
    };
    write_manifest(dir, &manifest);

    let err = inspect_sanitized_dataset(dir).expect_err("must fail");
    assert_eq!(err.code, "INGEST_SANITIZED_MANIFEST_VERSION_MISMATCH");
}

#[test]
fn inspect_fails_on_manifest_hash_mismatch_with_stable_code() {
    let tmp = tempdir().unwrap();

    // Export a valid dataset first.
    let db_path = tmp.path().join("db.sqlite");
    let mut conn = db::open(&db_path).expect("open");
    db::migrate(&mut conn).expect("migrate");
    seed_demo_dataset(&mut conn).expect("seed");

    let dest = tempdir().unwrap();
    let export_time = "2026-02-10T03:00:00Z";
    let app_version = "0.1.0-test";
    let exported = export_sanitized_dataset(&conn, dest.path(), export_time, app_version).expect("export");
    let dir = Path::new(&exported.export_dir);

    // Mutate a file without updating the manifest.
    let incidents_path = dir.join("incidents.json");
    let mut s = fs::read_to_string(&incidents_path).unwrap();
    s.push_str("\n ");
    fs::write(&incidents_path, s.as_bytes()).unwrap();

    let err = inspect_sanitized_dataset(dir).expect_err("must fail");
    assert_eq!(err.code, "INGEST_SANITIZED_MANIFEST_HASH_MISMATCH");
}

#[test]
fn inspect_fails_on_manifest_bytes_mismatch_with_stable_code() {
    let tmp = tempdir().unwrap();

    // Export a valid dataset first.
    let db_path = tmp.path().join("db.sqlite");
    let mut conn = db::open(&db_path).expect("open");
    db::migrate(&mut conn).expect("migrate");
    seed_demo_dataset(&mut conn).expect("seed");

    let dest = tempdir().unwrap();
    let export_time = "2026-02-10T03:00:00Z";
    let app_version = "0.1.0-test";
    let exported = export_sanitized_dataset(&conn, dest.path(), export_time, app_version).expect("export");
    let dir = Path::new(&exported.export_dir);

    // Load manifest, mutate the expected bytes for a file, and write it back.
    let manifest_path = dir.join("sanitized_manifest.json");
    let mut manifest: SanitizedExportManifest =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    let mut changed = false;
    for f in manifest.files.iter_mut() {
        if f.filename == "incidents.json" {
            f.bytes = f.bytes.saturating_add(1);
            changed = true;
        }
    }
    assert!(changed);
    write_manifest(dir, &manifest);

    let err = inspect_sanitized_dataset(dir).expect_err("must fail");
    assert_eq!(err.code, "INGEST_SANITIZED_MANIFEST_BYTES_MISMATCH");
}

#[test]
fn import_fails_on_metrics_mismatch_with_stable_code() {
    let tmp = tempdir().unwrap();

    // Export a valid dataset first.
    let db_path = tmp.path().join("db.sqlite");
    let mut conn = db::open(&db_path).expect("open");
    db::migrate(&mut conn).expect("migrate");
    seed_demo_dataset(&mut conn).expect("seed");

    let dest = tempdir().unwrap();
    let export_time = "2026-02-10T03:00:00Z";
    let app_version = "0.1.0-test";
    let exported = export_sanitized_dataset(&conn, dest.path(), export_time, app_version).expect("export");
    let dir = Path::new(&exported.export_dir);

    // Modify incidents.json metrics but keep manifest integrity by recomputing hash/bytes.
    let incidents_path = dir.join("incidents.json");
    let mut incidents: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&incidents_path).unwrap()).unwrap();
    let arr = incidents.as_array_mut().expect("incidents array");
    let first = arr.first_mut().expect("non-empty");
    // Force a mismatch: set mttr_seconds to an impossible value.
    first["metrics"]["mttr_seconds"] = serde_json::Value::Number(serde_json::Number::from(999999999i64));
    fs::write(
        &incidents_path,
        serde_json::to_string_pretty(&incidents).unwrap().as_bytes(),
    )
    .unwrap();

    // Recompute sha/bytes for incidents.json and update manifest accordingly.
    let manifest_path = dir.join("sanitized_manifest.json");
    let mut manifest: SanitizedExportManifest =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    let (sha, bytes) = {
        use sha2::{Digest, Sha256};
        let data = fs::read(&incidents_path).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&data);
        (hex::encode(hasher.finalize()), data.len() as u64)
    };
    for f in manifest.files.iter_mut() {
        if f.filename == "incidents.json" {
            f.sha256 = sha.clone();
            f.bytes = bytes;
        }
    }
    write_manifest(dir, &manifest);

    // Import into an empty DB should now fail due to metrics mismatch (not manifest mismatch).
    let db_b_path = tmp.path().join("b.sqlite");
    let mut conn_b = db::open(&db_b_path).expect("open b");
    db::migrate(&mut conn_b).expect("migrate b");

    let err = import_sanitized_dataset(&mut conn_b, dir).expect_err("metrics mismatch");
    assert_eq!(err.code, "INGEST_SANITIZED_METRICS_MISMATCH");
}
