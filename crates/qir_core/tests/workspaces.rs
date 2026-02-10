use std::path::Path;

use tempfile::tempdir;

use qir_core::analytics::build_dashboard_payload_v2;
use qir_core::demo::seed_demo_dataset;
use qir_core::report::generate_qir_markdown;
use qir_core::sanitize::{export_sanitized_dataset, import_sanitized_dataset};
use qir_core::validate::validate_all_incidents;
use qir_core::workspace::{
    create_workspace, create_workspace_connection, db_is_empty, open_workspace_connection,
};

fn count_incidents(conn: &rusqlite::Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM incidents", [], |row| row.get(0))
        .unwrap()
}

#[test]
fn workspace_isolation_create_open_switch() {
    let tmp = tempdir().unwrap();
    let w1 = tmp.path().join("w1.sqlite");
    let w2 = tmp.path().join("w2.sqlite");

    // Create workspace A and seed it.
    let mut conn1 = create_workspace_connection(&w1).expect("create w1");
    seed_demo_dataset(&mut conn1).expect("seed w1");
    assert!(count_incidents(&conn1) > 0);

    // Create workspace B and ensure it's empty.
    let conn2 = create_workspace_connection(&w2).expect("create w2");
    assert_eq!(count_incidents(&conn2), 0);
    assert!(db_is_empty(&w2).expect("is_empty"));

    // Re-open A and confirm data is still there.
    let conn1b = open_workspace_connection(&w1).expect("open w1");
    assert!(count_incidents(&conn1b) > 0);
}

#[test]
fn migrations_run_on_open_and_create() {
    let tmp = tempdir().unwrap();
    let w = tmp.path().join("migrate.sqlite");

    let meta = create_workspace(&w).expect("create meta");
    assert_eq!(meta.is_empty, true);

    // Re-open: should still succeed and preserve emptiness.
    let conn = open_workspace_connection(&w).expect("open");
    assert_eq!(count_incidents(&conn), 0);
}

#[test]
fn workspace_db_is_empty_reports_correctly() {
    let tmp = tempdir().unwrap();
    let w = tmp.path().join("empty.sqlite");

    let mut conn = create_workspace_connection(&w).expect("create");
    assert!(db_is_empty(&w).expect("empty true"));
    seed_demo_dataset(&mut conn).expect("seed");
    assert!(!db_is_empty(&w).expect("empty false"));
}

#[test]
fn scenario_create_seed_export_create_import_and_run_outputs() {
    let tmp = tempdir().unwrap();
    let w1 = tmp.path().join("scenario_w1.sqlite");
    let w2 = tmp.path().join("scenario_w2.sqlite");

    // Workspace W1: seed demo, export sanitized.
    let mut conn1 = create_workspace_connection(&w1).expect("create w1");
    seed_demo_dataset(&mut conn1).expect("seed w1");
    let dest = tempdir().unwrap();
    let export_time = "2026-02-10T03:00:00Z";
    let app_version = "0.1.0-test";
    let exported = export_sanitized_dataset(&conn1, dest.path(), export_time, app_version).expect("export");

    // Workspace W2: create fresh DB, import sanitized dataset (must be empty).
    let mut conn2 = create_workspace_connection(&w2).expect("create w2");
    let import = import_sanitized_dataset(&mut conn2, Path::new(&exported.export_dir)).expect("import");
    assert!(import.inserted_incidents > 0);

    // Confirm incidents list/dashboards/validation/report succeed in W2.
    let dash = build_dashboard_payload_v2(&conn2).expect("dashboard");
    assert_eq!(dash.incident_count, import.inserted_incidents);
    let _val = validate_all_incidents(&conn2).expect("validate");
    let report = generate_qir_markdown(&conn2).expect("report");
    assert!(report.contains("# Quarterly Incident Review (QIR)"));
}

