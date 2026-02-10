use qir_core::db;
use qir_core::ingest::jira_csv::{import_jira_csv, JiraCsvMapping};
use qir_core::profiles::jira::{
    delete_profile, get_profile, list_profiles, upsert_profile, JiraMappingProfileUpsert,
};
use qir_core::repo::count_incidents;

fn demo_mapping() -> JiraCsvMapping {
    JiraCsvMapping {
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
    }
}

#[test]
fn mapping_profile_crud_roundtrip() {
    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let created = upsert_profile(
        &mut conn,
        JiraMappingProfileUpsert {
            id: None,
            name: "Default Jira".to_string(),
            mapping: demo_mapping(),
        },
    )
    .expect("create");

    let listed = list_profiles(&conn).expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, created.id);

    let updated = upsert_profile(
        &mut conn,
        JiraMappingProfileUpsert {
            id: Some(created.id),
            name: "Default Jira Updated".to_string(),
            mapping: demo_mapping(),
        },
    )
    .expect("update");
    assert_eq!(updated.name, "Default Jira Updated");

    let got = get_profile(&conn, updated.id).expect("get");
    assert_eq!(got.name, "Default Jira Updated");

    delete_profile(&mut conn, got.id).expect("delete");
    let listed2 = list_profiles(&conn).expect("list2");
    assert!(listed2.is_empty());
}

#[test]
fn import_surfaces_duplicate_external_id_conflict() {
    let csv_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/demo/jira_duplicate_external_id.csv"
    ));

    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let summary = import_jira_csv(&mut conn, csv_text, &demo_mapping()).expect("import");
    assert_eq!(summary.inserted, 1);
    assert_eq!(summary.updated, 0);
    assert_eq!(summary.skipped, 1);
    assert_eq!(summary.conflicts.len(), 1);

    let count = count_incidents(&conn).expect("count");
    assert_eq!(count, 1);
}

#[test]
fn import_warns_on_non_rfc3339_timestamps_but_preserves_raw() {
    let csv_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/demo/jira_non_rfc3339.csv"
    ));

    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let summary = import_jira_csv(&mut conn, csv_text, &demo_mapping()).expect("import");
    assert_eq!(summary.inserted, 1);
    assert!(
        summary
            .warnings
            .iter()
            .any(|w| w.code == "INGEST_TS_UNPARSEABLE"),
        "expected unparseable timestamp warning, got: {:?}",
        summary.warnings
    );

    let (start_ts, start_ts_raw, ack_ts, ack_ts_raw, resolve_ts, resolve_ts_raw): (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(
            r#"
      SELECT start_ts, start_ts_raw, ack_ts, ack_ts_raw, resolve_ts, resolve_ts_raw
      FROM incidents WHERE external_id = ?1
      "#,
            ["INC-301"],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .unwrap();

    // Contract: canonical columns are RFC3339-only; non-RFC3339 inputs are preserved as raw with warnings.
    assert_eq!(start_ts, None);
    assert_eq!(ack_ts, None);
    assert_eq!(resolve_ts, None);
    assert_eq!(start_ts_raw.as_deref(), Some("01/05/2026 12:00"));
    assert_eq!(ack_ts_raw.as_deref(), Some("01/05/2026 12:05"));
    assert_eq!(resolve_ts_raw.as_deref(), Some("01/05/2026 13:00"));
}

#[test]
fn import_normalizes_allowlisted_timestamps_and_preserves_raw() {
    let csv_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/demo/jira_allowlisted_ts.csv"
    ));

    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let summary = import_jira_csv(&mut conn, csv_text, &demo_mapping()).expect("import");
    assert_eq!(summary.inserted, 1);
    assert!(
        summary
            .warnings
            .iter()
            .any(|w| w.code == "INGEST_TS_NORMALIZED"),
        "expected normalization warning, got: {:?}",
        summary.warnings
    );
    assert!(
        summary
            .warnings
            .iter()
            .any(|w| w.code == "INGEST_TS_TZ_ASSUMED_UTC"),
        "expected tz assumption warning, got: {:?}",
        summary.warnings
    );

    let (start_ts, start_ts_raw, ack_ts, ack_ts_raw): (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(
            r#"
      SELECT start_ts, start_ts_raw, ack_ts, ack_ts_raw
      FROM incidents WHERE external_id = ?1
      "#,
            ["INC-302"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();

    assert_eq!(start_ts.as_deref(), Some("2026-02-01T00:00:00Z"));
    assert_eq!(start_ts_raw.as_deref(), Some("2026-02-01 00:00:00"));
    assert_eq!(ack_ts.as_deref(), Some("2026-02-01T00:05:00Z"));
    assert_eq!(ack_ts_raw.as_deref(), Some("2026-02-01 00:05"));
}

#[test]
fn import_updates_existing_incident_by_external_id() {
    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let csv1 =
        "Key,Summary,Description,Severity,ImpactPct,ServiceHealthPct,StartTs,AckTs,ResolveTs\n\
INC-401,Title A,Desc A,SEV2,10,90,2026-02-01T00:00:00Z,2026-02-01T00:02:00Z,2026-02-01T00:10:00Z\n";
    // Preserve-on-empty semantics: empty cells must not overwrite existing values.
    // Here we change title/impact/health but leave description + severity empty.
    let csv2 =
        "Key,Summary,Description,Severity,ImpactPct,ServiceHealthPct,StartTs,AckTs,ResolveTs\n\
INC-401,Title B,,,20,80,2026-02-01T00:00:00Z,2026-02-01T00:02:00Z,2026-02-01T00:10:00Z\n";

    let s1 = import_jira_csv(&mut conn, csv1, &demo_mapping()).expect("import1");
    assert_eq!(s1.inserted, 1);

    let s2 = import_jira_csv(&mut conn, csv2, &demo_mapping()).expect("import2");
    assert_eq!(s2.updated, 1);
    assert_eq!(s2.inserted, 0);
    assert_eq!(s2.skipped, 0);

    let (title, description, severity, impact, health): (String, Option<String>, Option<String>, Option<i64>, Option<i64>) = conn
    .query_row(
      "SELECT title, description, severity, impact_pct, service_health_pct FROM incidents WHERE external_id = ?1",
      ["INC-401"],
      |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
    )
    .unwrap();
    assert_eq!(title, "Title B");
    assert_eq!(description.as_deref(), Some("Desc A"));
    assert_eq!(severity.as_deref(), Some("SEV2"));
    assert_eq!(impact, Some(20));
    assert_eq!(health, Some(80));
}

#[test]
fn import_skips_when_no_changes() {
    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let csv =
        "Key,Summary,Description,Severity,ImpactPct,ServiceHealthPct,StartTs,AckTs,ResolveTs\n\
INC-501,Same,Desc,SEV3,1,99,2026-02-02T00:00:00Z,2026-02-02T00:01:00Z,2026-02-02T00:02:00Z\n";

    let s1 = import_jira_csv(&mut conn, csv, &demo_mapping()).expect("import1");
    assert_eq!(s1.inserted, 1);

    let s2 = import_jira_csv(&mut conn, csv, &demo_mapping()).expect("import2");
    assert_eq!(s2.skipped, 1);
    assert_eq!(s2.inserted, 0);
    assert_eq!(s2.updated, 0);
}

#[test]
fn profile_can_be_used_for_import() {
    let mut conn = db::open_in_memory().expect("open");
    db::migrate(&mut conn).expect("migrate");

    let profile = upsert_profile(
        &mut conn,
        JiraMappingProfileUpsert {
            id: None,
            name: "ProfileForImport".to_string(),
            mapping: demo_mapping(),
        },
    )
    .expect("create profile");

    let csv = "Key,Summary,Description,Severity,ImpactPct,ServiceHealthPct,StartTs,AckTs,ResolveTs\n\
INC-601,FromProfile,Desc,SEV2,3,97,2026-02-03T00:00:00Z,2026-02-03T00:01:00Z,2026-02-03T00:02:00Z\n";

    let loaded = get_profile(&conn, profile.id).expect("load profile");
    let summary = import_jira_csv(&mut conn, csv, &loaded.mapping).expect("import");
    assert_eq!(summary.inserted, 1);
    assert_eq!(count_incidents(&conn).unwrap(), 1);
}
