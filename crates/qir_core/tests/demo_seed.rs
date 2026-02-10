use tempfile::tempdir;

use qir_core::db;
use qir_core::demo::seed_demo_dataset;

#[test]
fn seeds_demo_dataset_with_enough_incidents_for_dashboards() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("demo.sqlite");
    let mut conn = db::open(&db_path).expect("open");
    db::migrate(&mut conn).expect("migrate");

    let res = seed_demo_dataset(&mut conn).expect("seed");
    assert!(res.inserted >= 40, "expected >= 40 demo incidents");
    assert!(
        res.warnings.is_empty() && res.conflicts.is_empty(),
        "expected clean demo seed"
    );
}

