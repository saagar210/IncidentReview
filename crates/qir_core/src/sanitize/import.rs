use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::domain::{Incident, ValidationWarning};
use crate::error::AppError;
use crate::{metrics, validate};

use super::{SanitizedExportManifest, SanitizedIncident, SanitizedTimelineEvent, SanitizedWarning};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SanitizedImportSummary {
    pub inserted_incidents: i64,
    pub inserted_timeline_events: i64,
    pub import_warnings: Vec<ValidationWarning>,
}

fn sha256_file_hex_ingest(path: &Path) -> Result<(String, u64), AppError> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut f = fs::File::open(path).map_err(|e| {
        AppError::new(
            "INGEST_SANITIZED_FILE_OPEN_FAILED",
            "Failed to open sanitized dataset file for hashing",
        )
        .with_details(format!("path={}: {}", path.display(), e))
    })?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    let mut total: u64 = 0;
    loop {
        let n = f.read(&mut buf).map_err(|e| {
            AppError::new(
                "INGEST_SANITIZED_FILE_READ_FAILED",
                "Failed to read sanitized dataset file for hashing",
            )
            .with_details(format!("path={}: {}", path.display(), e))
        })?;
        if n == 0 {
            break;
        }
        total += n as u64;
        hasher.update(&buf[..n]);
    }
    Ok((hex::encode(hasher.finalize()), total))
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, AppError> {
    let bytes = fs::read(path).map_err(|e| {
        AppError::new(
            "INGEST_SANITIZED_READ_FAILED",
            "Failed to read sanitized dataset file",
        )
        .with_details(format!("path={}: {}", path.display(), e))
    })?;
    serde_json::from_slice(&bytes).map_err(|e| {
        AppError::new(
            "INGEST_SANITIZED_DECODE_FAILED",
            "Failed to decode sanitized dataset JSON",
        )
        .with_details(format!("path={}: {}", path.display(), e))
    })
}

pub fn read_sanitized_manifest(dataset_dir: &Path) -> Result<SanitizedExportManifest, AppError> {
    if !dataset_dir.is_dir() {
        return Err(AppError::new(
            "INGEST_SANITIZED_NOT_DIR",
            "Sanitized dataset path must be a directory",
        )
        .with_details(dataset_dir.display().to_string()));
    }
    let manifest_path = dataset_dir.join("sanitized_manifest.json");
    read_json_file::<SanitizedExportManifest>(manifest_path.as_path())
}

fn verify_manifest_and_files(
    dataset_dir: &Path,
    manifest: &SanitizedExportManifest,
) -> Result<(), AppError> {
    if manifest.manifest_version != 1 {
        return Err(AppError::new(
            "INGEST_SANITIZED_MANIFEST_VERSION_MISMATCH",
            "Sanitized manifest version mismatch (expected 1)",
        )
        .with_details(format!("manifest_version={}", manifest.manifest_version)));
    }

    // For DS5, require these exact JSON files to exist and be referenced in the manifest file list.
    let required = ["incidents.json", "timeline_events.json", "warnings.json"];
    for r in required {
        let mentioned = manifest.files.iter().any(|f| f.filename == r);
        if !mentioned {
            return Err(AppError::new(
                "INGEST_SANITIZED_MANIFEST_MISSING_FILE",
                "Sanitized manifest missing required file entry",
            )
            .with_details(format!("missing={r}")));
        }
        let path = dataset_dir.join(r);
        if !path.is_file() {
            return Err(AppError::new(
                "INGEST_SANITIZED_FILE_MISSING",
                "Required sanitized dataset file is missing",
            )
            .with_details(format!("path={}", path.display())));
        }
    }

    // Verify hashes and byte sizes for every file listed in the manifest.
    for f in &manifest.files {
        let path = dataset_dir.join(&f.filename);
        if !path.is_file() {
            return Err(AppError::new(
                "INGEST_SANITIZED_FILE_MISSING",
                "Sanitized dataset file listed in manifest is missing",
            )
            .with_details(format!("path={}", path.display())));
        }
        let (sha, bytes) = sha256_file_hex_ingest(&path)?;
        if sha != f.sha256 {
            return Err(AppError::new(
                "INGEST_SANITIZED_MANIFEST_HASH_MISMATCH",
                "Sanitized dataset file hash mismatch",
            )
            .with_details(format!(
                "file={}; expected_sha256={}; actual_sha256={}",
                f.filename, f.sha256, sha
            )));
        }
        if bytes != f.bytes {
            return Err(AppError::new(
                "INGEST_SANITIZED_MANIFEST_BYTES_MISMATCH",
                "Sanitized dataset file size mismatch",
            )
            .with_details(format!(
                "file={}; expected_bytes={}; actual_bytes={}",
                f.filename, f.bytes, bytes
            )));
        }
    }

    Ok(())
}

pub fn inspect_sanitized_dataset(dataset_dir: &Path) -> Result<SanitizedExportManifest, AppError> {
    let manifest = read_sanitized_manifest(dataset_dir)?;
    verify_manifest_and_files(dataset_dir, &manifest)?;
    Ok(manifest)
}

fn db_is_empty(conn: &Connection) -> Result<(bool, String), AppError> {
    let incidents: i64 = conn
        .query_row("SELECT COUNT(*) FROM incidents", [], |row| row.get(0))
        .map_err(|e| {
            AppError::new(
                "DB_QUERY_FAILED",
                "Failed to count incidents during sanitized import",
            )
            .with_details(e.to_string())
        })?;
    let events: i64 = conn
        .query_row("SELECT COUNT(*) FROM timeline_events", [], |row| row.get(0))
        .map_err(|e| {
            AppError::new(
                "DB_QUERY_FAILED",
                "Failed to count timeline events during sanitized import",
            )
            .with_details(e.to_string())
        })?;
    let artifacts: i64 = conn
        .query_row("SELECT COUNT(*) FROM artifacts", [], |row| row.get(0))
        .map_err(|e| {
            AppError::new(
                "DB_QUERY_FAILED",
                "Failed to count artifacts during sanitized import",
            )
            .with_details(e.to_string())
        })?;

    let ok = incidents == 0 && events == 0 && artifacts == 0;
    Ok((ok, format!("incidents={incidents}; timeline_events={events}; artifacts={artifacts}")))
}

fn sanitized_fingerprint(incident_key: &str) -> String {
    use sha2::{Digest, Sha256};
    let payload = format!("sanitized|incident_key={incident_key}");
    hex::encode(Sha256::digest(payload.as_bytes()))
}

fn metrics_equal(a: &super::SanitizedMetrics, b: &super::SanitizedMetrics) -> bool {
    a.mttd_seconds == b.mttd_seconds
        && a.it_awareness_lag_seconds == b.it_awareness_lag_seconds
        && a.mtta_seconds == b.mtta_seconds
        && a.time_to_mitigation_seconds == b.time_to_mitigation_seconds
        && a.mttr_seconds == b.mttr_seconds
}

pub fn import_sanitized_dataset(
    conn: &mut Connection,
    dataset_dir: &Path,
) -> Result<SanitizedImportSummary, AppError> {
    let manifest = inspect_sanitized_dataset(dataset_dir)?;

    let (empty, details) = db_is_empty(conn)?;
    if !empty {
        return Err(AppError::new(
            "INGEST_SANITIZED_DB_NOT_EMPTY",
            "Refusing to import sanitized dataset into a non-empty database",
        )
        .with_details(details));
    }

    let incidents_path = dataset_dir.join("incidents.json");
    let events_path = dataset_dir.join("timeline_events.json");
    let warnings_path = dataset_dir.join("warnings.json");

    let mut incidents = read_json_file::<Vec<SanitizedIncident>>(incidents_path.as_path())?;
    let mut events = read_json_file::<Vec<SanitizedTimelineEvent>>(events_path.as_path())?;
    let warnings_flat = read_json_file::<Vec<SanitizedWarning>>(warnings_path.as_path())?;

    if incidents.len() as i64 != manifest.incident_count {
        // No silent defaults: this is a contract mismatch; treat as a hard failure for DS5.
        return Err(AppError::new(
            "INGEST_SANITIZED_INCIDENT_COUNT_MISMATCH",
            "Sanitized manifest incident_count does not match incidents.json length",
        )
        .with_details(format!(
            "manifest_incident_count={}; incidents_len={}",
            manifest.incident_count,
            incidents.len()
        )));
    }

    // Ensure deterministic ordering and stable ID assignment in a fresh DB.
    incidents.sort_by(|a, b| a.incident_key.cmp(&b.incident_key));
    events.sort_by(|a, b| {
        a.incident_key
            .cmp(&b.incident_key)
            .then_with(|| a.ts.cmp(&b.ts))
            .then_with(|| a.source.cmp(&b.source))
    });

    let export_time = manifest.export_time.clone();

    let mut import_warnings: Vec<ValidationWarning> = Vec::new();
    import_warnings.push(
        ValidationWarning::new(
            "INGEST_SANITIZED_TITLE_REDACTED",
            "Incident titles were deterministically replaced during sanitized import",
        )
        .with_details(format!("incidents={}", incidents.len())),
    );
    import_warnings.push(
        ValidationWarning::new(
            "INGEST_SANITIZED_TIMELINE_TEXT_REDACTED",
            "Timeline event text was deterministically replaced during sanitized import",
        )
        .with_details(format!("timeline_events={}", events.len())),
    );

    // Insert incidents and build mapping from incident_key -> inserted id.
    let mut id_by_key: BTreeMap<String, i64> = BTreeMap::new();
    for inc in &incidents {
        let incident_key = inc.incident_key.clone();

        let title = format!("Incident {incident_key}");
        let fp = sanitized_fingerprint(&incident_key);

        // Metrics are deterministic truth; validate the dataset wasn't tampered with.
        let candidate = Incident {
            id: 0,
            external_id: Some(incident_key.clone()),
            fingerprint: fp.clone(),
            title: title.clone(),
            description: None,
            severity: inc.severity.clone(),
            detection_source: inc.detection_source.clone(),
            vendor: inc.vendor.clone(),
            service: inc.service.clone(),
            impact_pct: inc.impact_pct,
            service_health_pct: inc.service_health_pct,
            start_ts: inc.start_ts.clone(),
            first_observed_ts: inc.first_observed_ts.clone(),
            it_awareness_ts: inc.it_awareness_ts.clone(),
            ack_ts: inc.ack_ts.clone(),
            mitigate_ts: inc.mitigate_ts.clone(),
            resolve_ts: inc.resolve_ts.clone(),
            start_ts_raw: None,
            first_observed_ts_raw: None,
            it_awareness_ts_raw: None,
            ack_ts_raw: None,
            mitigate_ts_raw: None,
            resolve_ts_raw: None,
        };

        let (m, _metric_warnings) = metrics::compute_incident_metrics(&candidate);
        let computed = super::SanitizedMetrics {
            mttd_seconds: m.mttd_seconds,
            it_awareness_lag_seconds: m.it_awareness_lag_seconds,
            mtta_seconds: m.mtta_seconds,
            time_to_mitigation_seconds: m.time_to_mitigation_seconds,
            mttr_seconds: m.mttr_seconds,
        };
        if !metrics_equal(&computed, &inc.metrics) {
            return Err(AppError::new(
                "INGEST_SANITIZED_METRICS_MISMATCH",
                "Sanitized incident metrics do not match deterministic recomputation",
            )
            .with_details(format!(
                "incident_key={}; expected={:?}; computed={:?}",
                incident_key, inc.metrics, computed
            )));
        }

        conn.execute(
            r#"
            INSERT INTO incidents(
              external_id, fingerprint, title, description, severity,
              detection_source, vendor, service,
              impact_pct, service_health_pct,
              start_ts, first_observed_ts, it_awareness_ts, ack_ts, mitigate_ts, resolve_ts,
              start_ts_raw, first_observed_ts_raw, it_awareness_ts_raw, ack_ts_raw, mitigate_ts_raw, resolve_ts_raw,
              ingested_at
            ) VALUES (
              ?1, ?2, ?3, ?4, ?5,
              ?6, ?7, ?8,
              ?9, ?10,
              ?11, ?12, ?13, ?14, ?15, ?16,
              NULL, NULL, NULL, NULL, NULL, NULL,
              ?17
            )
            "#,
            rusqlite::params![
                Some(incident_key.clone()),
                fp,
                title,
                Option::<String>::None,
                inc.severity.clone(),
                inc.detection_source.clone(),
                inc.vendor.clone(),
                inc.service.clone(),
                inc.impact_pct,
                inc.service_health_pct,
                inc.start_ts.clone(),
                inc.first_observed_ts.clone(),
                inc.it_awareness_ts.clone(),
                inc.ack_ts.clone(),
                inc.mitigate_ts.clone(),
                inc.resolve_ts.clone(),
                export_time.as_str(),
            ],
        )
        .map_err(|e| {
            AppError::new(
                "INGEST_SANITIZED_INSERT_INCIDENT_FAILED",
                "Failed to insert incident during sanitized import",
            )
            .with_details(format!("incident_key={}; err={}", incident_key, e))
        })?;

        let id = conn.last_insert_rowid();
        id_by_key.insert(incident_key, id);
    }

    // Insert timeline events with explicit redaction markers.
    let mut inserted_events: i64 = 0;
    for e in &events {
        if !e.text_redacted {
            return Err(AppError::new(
                "INGEST_SANITIZED_EVENT_NOT_REDACTED",
                "Sanitized timeline event indicates text_redacted=false (refusing import)",
            )
            .with_details(format!(
                "incident_key={}; source={}; ts={:?}",
                e.incident_key, e.source, e.ts
            )));
        }

        let incident_id = id_by_key.get(&e.incident_key).copied().ok_or_else(|| {
            AppError::new(
                "INGEST_SANITIZED_EVENT_INCIDENT_UNKNOWN",
                "Timeline event references unknown incident_key",
            )
            .with_details(format!("incident_key={}", e.incident_key))
        })?;

        let raw_json = r#"{"text_redacted":true}"#;

        conn.execute(
            r#"
            INSERT INTO timeline_events(
              incident_id, source, ts, author, kind, text, raw_json, created_at
            ) VALUES (
              ?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7
            )
            "#,
            rusqlite::params![
                incident_id,
                e.source.as_str(),
                e.ts.as_deref(),
                e.kind.as_deref(),
                "[REDACTED]",
                raw_json,
                export_time.as_str(),
            ],
        )
        .map_err(|err| {
            AppError::new(
                "INGEST_SANITIZED_INSERT_EVENT_FAILED",
                "Failed to insert timeline event during sanitized import",
            )
            .with_details(format!(
                "incident_key={}; source={}; ts={:?}; err={}",
                e.incident_key, e.source, e.ts, err
            ))
        })?;
        inserted_events += 1;
    }

    // Warning reconciliation policy (DS5): do not fail; surface explicit warning if mismatched.
    let mut expected_by_key: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for w in &warnings_flat {
        expected_by_key
            .entry(w.incident_key.clone())
            .or_default()
            .push(w.code.clone());
    }
    for (_, v) in expected_by_key.iter_mut() {
        v.sort();
    }

    let actual_items = validate::validate_all_incidents(conn)?;
    let mut actual_by_key: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for item in actual_items {
        let key = item
            .external_id
            .clone()
            .unwrap_or_else(|| "INC_UNKNOWN".to_string());
        let mut codes = item.warnings.into_iter().map(|w| w.code).collect::<Vec<_>>();
        codes.sort();
        actual_by_key.insert(key, codes);
    }

    fn freq(codes: &[String]) -> BTreeMap<String, i64> {
        let mut m = BTreeMap::new();
        for c in codes {
            *m.entry(c.clone()).or_insert(0) += 1;
        }
        m
    }

    let mut expected_total = 0i64;
    let mut actual_total = 0i64;
    let mut missing: BTreeMap<String, i64> = BTreeMap::new();
    let mut extra: BTreeMap<String, i64> = BTreeMap::new();

    let mut all_keys: Vec<String> = expected_by_key
        .keys()
        .chain(actual_by_key.keys())
        .cloned()
        .collect();
    all_keys.sort();
    all_keys.dedup();

    for k in &all_keys {
        let exp = expected_by_key.get(k).cloned().unwrap_or_default();
        let act = actual_by_key.get(k).cloned().unwrap_or_default();
        expected_total += exp.len() as i64;
        actual_total += act.len() as i64;

        let exp_f = freq(&exp);
        let act_f = freq(&act);
        for (code, n_exp) in exp_f.iter() {
            let n_act = *act_f.get(code).unwrap_or(&0);
            if *n_exp > n_act {
                *missing.entry(code.clone()).or_insert(0) += *n_exp - n_act;
            }
        }
        for (code, n_act) in act_f.iter() {
            let n_exp = *exp_f.get(code).unwrap_or(&0);
            if *n_act > n_exp {
                *extra.entry(code.clone()).or_insert(0) += *n_act - n_exp;
            }
        }
    }

    if expected_total != actual_total || !missing.is_empty() || !extra.is_empty() {
        let mut missing_items = missing.into_iter().collect::<Vec<_>>();
        missing_items.sort_by(|a, b| (-(a.1), a.0.clone()).cmp(&(-(b.1), b.0.clone())));
        let mut extra_items = extra.into_iter().collect::<Vec<_>>();
        extra_items.sort_by(|a, b| (-(a.1), a.0.clone()).cmp(&(-(b.1), b.0.clone())));

        let fmt_top = |items: &[(String, i64)]| -> String {
            items
                .iter()
                .take(5)
                .map(|(c, n)| format!("{c}x{n}"))
                .collect::<Vec<_>>()
                .join(",")
        };

        import_warnings.push(
            ValidationWarning::new(
                "INGEST_SANITIZED_WARNINGS_MISMATCH",
                "Sanitized warnings do not match validator output after import",
            )
            .with_details(format!(
                "expected_total={}; actual_total={}; missing_top=[{}]; extra_top=[{}]",
                expected_total,
                actual_total,
                fmt_top(&missing_items),
                fmt_top(&extra_items)
            )),
        );
    }

    Ok(SanitizedImportSummary {
        inserted_incidents: incidents.len() as i64,
        inserted_timeline_events: inserted_events,
        import_warnings,
    })
}
