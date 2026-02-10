use crate::domain::ValidationWarning;
use crate::error::AppError;
use crate::normalize::timestamps::normalize_timestamp;

use sha2::{Digest, Sha256};

use rusqlite::Connection;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JiraCsvMapping {
    /// CSV column header for external ID (e.g. Jira key). Optional.
    pub external_id: Option<String>,
    /// CSV column header for incident title/summary. Required.
    pub title: String,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub detection_source: Option<String>,
    pub vendor: Option<String>,
    pub service: Option<String>,
    pub impact_pct: Option<String>,
    pub service_health_pct: Option<String>,

    pub start_ts: Option<String>,
    pub first_observed_ts: Option<String>,
    pub it_awareness_ts: Option<String>,
    pub ack_ts: Option<String>,
    pub mitigate_ts: Option<String>,
    pub resolve_ts: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JiraIngestSummary {
    pub inserted: usize,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JiraCsvPreview {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JiraImportConflict {
    pub row: usize,
    pub reason: String,
    pub external_id: Option<String>,
    pub fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JiraImportSummary {
    pub inserted: usize,
    pub updated: usize,
    pub skipped: usize,
    pub conflicts: Vec<JiraImportConflict>,
    pub warnings: Vec<ValidationWarning>,
}

fn normalize_title_for_fingerprint(title: &str) -> String {
    title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_lowercase()
}

fn fingerprint(
    title: &str,
    start_ts: Option<&str>,
    first_observed_ts: Option<&str>,
    ack_ts: Option<&str>,
    mitigate_ts: Option<&str>,
    resolve_ts: Option<&str>,
) -> String {
    // Stable fingerprint derived from normalized title + primary timestamps (nullable).
    // This matches the repo rule: "normalized title + date + primary timestamps".
    // We omit a separate date field for now and rely on timestamps; if all are missing,
    // the fingerprint will be title-only and collisions will be surfaced as conflicts.
    let payload = format!(
        "title={}|start={}|first_observed={}|ack={}|mitigate={}|resolve={}",
        normalize_title_for_fingerprint(title),
        start_ts.unwrap_or(""),
        first_observed_ts.unwrap_or(""),
        ack_ts.unwrap_or(""),
        mitigate_ts.unwrap_or(""),
        resolve_ts.unwrap_or("")
    );
    let digest = Sha256::digest(payload.as_bytes());
    hex::encode(digest)
}

fn get<'a>(
    row: &'a csv::StringRecord,
    headers: &'a csv::StringRecord,
    header_name: &str,
) -> Option<&'a str> {
    headers
        .iter()
        .position(|h| h == header_name)
        .and_then(|idx| row.get(idx))
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
}

fn parse_pct(raw: Option<&str>, field: &str, warnings: &mut Vec<ValidationWarning>) -> Option<i64> {
    let Some(s) = raw else { return None };
    match s.parse::<i64>() {
        Ok(v) if (0..=100).contains(&v) => Some(v),
        Ok(v) => {
            warnings.push(
                ValidationWarning::new(
                    "VALIDATION_PCT_OUT_OF_RANGE",
                    format!("{field} out of range"),
                )
                .with_details(format!("value={v}")),
            );
            None
        }
        Err(e) => {
            warnings.push(
                ValidationWarning::new(
                    "VALIDATION_PCT_PARSE_FAILED",
                    format!("Failed to parse {field}"),
                )
                .with_details(format!("value={s}; err={e}")),
            );
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TimestampUpdate {
    // Preserve-on-empty semantics:
    // - provided=false: do not update stored canonical/raw.
    // - provided=true: overwrite stored canonical/raw exactly as computed, even if canonical is NULL.
    provided: bool,
    canonical: Option<String>,
    raw: Option<String>,
}

fn ts_update_from_cell(
    row_idx: usize,
    field: &str,
    cell: Option<String>,
    warnings: &mut Vec<ValidationWarning>,
) -> TimestampUpdate {
    let Some(v) = cell else {
        return TimestampUpdate {
            provided: false,
            canonical: None,
            raw: None,
        };
    };

    let norm = normalize_timestamp(field, &v, warnings);
    if norm.raw.is_some() && norm.canonical_rfc3339_utc.is_none() {
        warnings.push(
            ValidationWarning::new(
                "INGEST_TS_RAW_STORED",
                format!("Stored raw timestamp for {field} (no canonical)"),
            )
            .with_details(format!("row={row_idx}")),
        );
    }

    TimestampUpdate {
        provided: true,
        canonical: norm.canonical_rfc3339_utc,
        raw: norm.raw,
    }
}

pub fn preview_jira_csv(csv_text: &str, max_rows: usize) -> Result<JiraCsvPreview, AppError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| {
            AppError::new(
                "INGEST_JIRA_CSV_HEADERS_FAILED",
                "Failed to read Jira CSV headers",
            )
            .with_details(e.to_string())
        })?
        .iter()
        .map(|h| h.to_string())
        .collect::<Vec<_>>();

    let mut rows = Vec::new();
    for result in rdr.records().take(max_rows) {
        let row = result.map_err(|e| {
            AppError::new(
                "INGEST_JIRA_CSV_PARSE_FAILED",
                "Failed to parse Jira CSV row",
            )
            .with_details(e.to_string())
        })?;
        rows.push(row.iter().map(|v| v.to_string()).collect::<Vec<_>>());
    }

    Ok(JiraCsvPreview { headers, rows })
}

fn is_unique_constraint_error(err: &rusqlite::Error) -> bool {
    match err {
        // rusqlite versions differ in the granularity of constraint error codes.
        // We treat any constraint violation as a conflict that must be surfaced explicitly.
        rusqlite::Error::SqliteFailure(e, _) => e.code == rusqlite::ErrorCode::ConstraintViolation,
        _ => false,
    }
}

fn find_existing_by_external_id(
    conn: &Connection,
    external_id: &str,
) -> Result<Option<i64>, AppError> {
    conn.query_row(
        "SELECT id FROM incidents WHERE external_id = ?1",
        [external_id],
        |row| row.get::<_, i64>(0),
    )
    .optional()
    .map_err(|e| {
        AppError::new("DB_QUERY_FAILED", "Failed to query incident by external_id")
            .with_details(e.to_string())
    })
}

fn find_existing_by_fingerprint(conn: &Connection, fp: &str) -> Result<Option<i64>, AppError> {
    conn.query_row(
        "SELECT id FROM incidents WHERE fingerprint = ?1",
        [fp],
        |row| row.get::<_, i64>(0),
    )
    .optional()
    .map_err(|e| {
        AppError::new("DB_QUERY_FAILED", "Failed to query incident by fingerprint")
            .with_details(e.to_string())
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IncidentRecord {
    external_id: Option<String>,
    fingerprint: String,
    title: String,
    description: Option<String>,
    severity: Option<String>,
    detection_source: Option<String>,
    vendor: Option<String>,
    service: Option<String>,
    impact_pct: Option<i64>,
    service_health_pct: Option<i64>,
    start_ts: Option<String>,
    first_observed_ts: Option<String>,
    it_awareness_ts: Option<String>,
    ack_ts: Option<String>,
    mitigate_ts: Option<String>,
    resolve_ts: Option<String>,
    start_ts_raw: Option<String>,
    first_observed_ts_raw: Option<String>,
    it_awareness_ts_raw: Option<String>,
    ack_ts_raw: Option<String>,
    mitigate_ts_raw: Option<String>,
    resolve_ts_raw: Option<String>,
}

fn load_incident_record_for_compare(
    conn: &Connection,
    id: i64,
) -> Result<IncidentRecord, AppError> {
    conn.query_row(
        r#"
      SELECT external_id, fingerprint, title, description, severity,
             detection_source, vendor, service,
             impact_pct, service_health_pct,
             start_ts, first_observed_ts, it_awareness_ts, ack_ts, mitigate_ts, resolve_ts,
             start_ts_raw, first_observed_ts_raw, it_awareness_ts_raw, ack_ts_raw, mitigate_ts_raw, resolve_ts_raw
      FROM incidents WHERE id = ?1
      "#,
        [id],
        |row| {
            Ok(IncidentRecord {
                external_id: row.get(0)?,
                fingerprint: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                severity: row.get(4)?,
                detection_source: row.get(5)?,
                vendor: row.get(6)?,
                service: row.get(7)?,
                impact_pct: row.get(8)?,
                service_health_pct: row.get(9)?,
                start_ts: row.get(10)?,
                first_observed_ts: row.get(11)?,
                it_awareness_ts: row.get(12)?,
                ack_ts: row.get(13)?,
                mitigate_ts: row.get(14)?,
                resolve_ts: row.get(15)?,
                start_ts_raw: row.get(16)?,
                first_observed_ts_raw: row.get(17)?,
                it_awareness_ts_raw: row.get(18)?,
                ack_ts_raw: row.get(19)?,
                mitigate_ts_raw: row.get(20)?,
                resolve_ts_raw: row.get(21)?,
            })
        },
    )
    .map_err(|e| {
        AppError::new("DB_QUERY_FAILED", "Failed to load incident for compare")
            .with_details(e.to_string())
    })
}

fn merge_preserve_on_empty(
    existing: &IncidentRecord,
    incoming: IncidentRecord,
    start: TimestampUpdate,
    first_observed: TimestampUpdate,
    it_awareness: TimestampUpdate,
    ack: TimestampUpdate,
    mitigate: TimestampUpdate,
    resolve: TimestampUpdate,
) -> IncidentRecord {
    // Preserve-on-empty semantics:
    // - If the CSV cell is missing/empty, we do NOT overwrite the existing value with NULL.
    // - For required fields (title), the incoming value is always present by construction.
    //
    // Timestamp fields require special handling:
    // - provided=true with raw=None must clear any previously stored raw.
    // - provided=true with canonical=None must overwrite canonical to NULL (raw preserved).
    IncidentRecord {
        external_id: incoming
            .external_id
            .or_else(|| existing.external_id.clone()),
        fingerprint: incoming.fingerprint, // will be recomputed after merge
        title: incoming.title,
        description: incoming
            .description
            .or_else(|| existing.description.clone()),
        severity: incoming.severity.or_else(|| existing.severity.clone()),
        detection_source: incoming
            .detection_source
            .or_else(|| existing.detection_source.clone()),
        vendor: incoming.vendor.or_else(|| existing.vendor.clone()),
        service: incoming.service.or_else(|| existing.service.clone()),
        impact_pct: incoming.impact_pct.or(existing.impact_pct),
        service_health_pct: incoming.service_health_pct.or(existing.service_health_pct),
        start_ts: if start.provided {
            start.canonical
        } else {
            existing.start_ts.clone()
        },
        first_observed_ts: if first_observed.provided {
            first_observed.canonical
        } else {
            existing.first_observed_ts.clone()
        },
        it_awareness_ts: if it_awareness.provided {
            it_awareness.canonical
        } else {
            existing.it_awareness_ts.clone()
        },
        ack_ts: if ack.provided {
            ack.canonical
        } else {
            existing.ack_ts.clone()
        },
        mitigate_ts: if mitigate.provided {
            mitigate.canonical
        } else {
            existing.mitigate_ts.clone()
        },
        resolve_ts: if resolve.provided {
            resolve.canonical
        } else {
            existing.resolve_ts.clone()
        },

        start_ts_raw: if start.provided {
            start.raw
        } else {
            existing.start_ts_raw.clone()
        },
        first_observed_ts_raw: if first_observed.provided {
            first_observed.raw
        } else {
            existing.first_observed_ts_raw.clone()
        },
        it_awareness_ts_raw: if it_awareness.provided {
            it_awareness.raw
        } else {
            existing.it_awareness_ts_raw.clone()
        },
        ack_ts_raw: if ack.provided {
            ack.raw
        } else {
            existing.ack_ts_raw.clone()
        },
        mitigate_ts_raw: if mitigate.provided {
            mitigate.raw
        } else {
            existing.mitigate_ts_raw.clone()
        },
        resolve_ts_raw: if resolve.provided {
            resolve.raw
        } else {
            existing.resolve_ts_raw.clone()
        },
    }
}

pub fn import_jira_csv(
    conn: &mut Connection,
    csv_text: &str,
    mapping: &JiraCsvMapping,
) -> Result<JiraImportSummary, AppError> {
    let mut warnings = Vec::new();
    let mut conflicts = Vec::new();
    let mut inserted = 0usize;
    let mut updated = 0usize;
    let mut skipped = 0usize;

    let mut seen_external_ids = std::collections::HashSet::<String>::new();
    let mut seen_fps = std::collections::HashSet::<String>::new();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| {
            AppError::new(
                "INGEST_JIRA_CSV_HEADERS_FAILED",
                "Failed to read Jira CSV headers",
            )
            .with_details(e.to_string())
        })?
        .clone();

    for (row_idx, result) in rdr.records().enumerate() {
        let row = match result {
            Ok(r) => r,
            Err(e) => {
                warnings.push(
                    ValidationWarning::new(
                        "INGEST_JIRA_CSV_PARSE_FAILED",
                        "Failed to parse Jira CSV row",
                    )
                    .with_details(format!("row={row_idx}; err={e}")),
                );
                skipped += 1;
                continue;
            }
        };

        let title = get(&row, &headers, &mapping.title)
            .unwrap_or("")
            .to_string();
        if title.trim().is_empty() {
            warnings.push(
                ValidationWarning::new("INGEST_MISSING_TITLE", "Row missing required title")
                    .with_details(format!("row={row_idx}")),
            );
            skipped += 1;
            continue;
        }

        let external_id = mapping
            .external_id
            .as_deref()
            .and_then(|h| get(&row, &headers, h))
            .map(|s| s.to_string());

        if let Some(ref ext) = external_id {
            if !seen_external_ids.insert(ext.clone()) {
                conflicts.push(JiraImportConflict {
                    row: row_idx,
                    reason: "Duplicate external_id in import".to_string(),
                    external_id: Some(ext.clone()),
                    fingerprint: None,
                });
                skipped += 1;
                continue;
            }
        }

        let description = mapping
            .description
            .as_deref()
            .and_then(|h| get(&row, &headers, h))
            .map(|s| s.to_string());

        let severity = mapping
            .severity
            .as_deref()
            .and_then(|h| get(&row, &headers, h))
            .map(|s| s.to_string());

        let detection_source = mapping
            .detection_source
            .as_deref()
            .and_then(|h| get(&row, &headers, h))
            .map(|s| s.to_string());

        let vendor = mapping
            .vendor
            .as_deref()
            .and_then(|h| get(&row, &headers, h))
            .map(|s| s.to_string());

        let service = mapping
            .service
            .as_deref()
            .and_then(|h| get(&row, &headers, h))
            .map(|s| s.to_string());

        let start_update = ts_update_from_cell(
            row_idx,
            "start_ts",
            mapping
                .start_ts
                .as_deref()
                .and_then(|h| get(&row, &headers, h))
                .map(|s| s.to_string()),
            &mut warnings,
        );
        let first_observed_update = ts_update_from_cell(
            row_idx,
            "first_observed_ts",
            mapping
                .first_observed_ts
                .as_deref()
                .and_then(|h| get(&row, &headers, h))
                .map(|s| s.to_string()),
            &mut warnings,
        );
        let it_awareness_update = ts_update_from_cell(
            row_idx,
            "it_awareness_ts",
            mapping
                .it_awareness_ts
                .as_deref()
                .and_then(|h| get(&row, &headers, h))
                .map(|s| s.to_string()),
            &mut warnings,
        );
        let ack_update = ts_update_from_cell(
            row_idx,
            "ack_ts",
            mapping
                .ack_ts
                .as_deref()
                .and_then(|h| get(&row, &headers, h))
                .map(|s| s.to_string()),
            &mut warnings,
        );
        let mitigate_update = ts_update_from_cell(
            row_idx,
            "mitigate_ts",
            mapping
                .mitigate_ts
                .as_deref()
                .and_then(|h| get(&row, &headers, h))
                .map(|s| s.to_string()),
            &mut warnings,
        );
        let resolve_update = ts_update_from_cell(
            row_idx,
            "resolve_ts",
            mapping
                .resolve_ts
                .as_deref()
                .and_then(|h| get(&row, &headers, h))
                .map(|s| s.to_string()),
            &mut warnings,
        );

        let start_ts = start_update.canonical.clone();
        let start_ts_raw = start_update.raw.clone();
        let first_observed_ts = first_observed_update.canonical.clone();
        let first_observed_ts_raw = first_observed_update.raw.clone();
        let it_awareness_ts = it_awareness_update.canonical.clone();
        let it_awareness_ts_raw = it_awareness_update.raw.clone();
        let ack_ts = ack_update.canonical.clone();
        let ack_ts_raw = ack_update.raw.clone();
        let mitigate_ts = mitigate_update.canonical.clone();
        let mitigate_ts_raw = mitigate_update.raw.clone();
        let resolve_ts = resolve_update.canonical.clone();
        let resolve_ts_raw = resolve_update.raw.clone();

        let impact_pct = parse_pct(
            mapping
                .impact_pct
                .as_deref()
                .and_then(|h| get(&row, &headers, h)),
            "impact_pct",
            &mut warnings,
        );
        let service_health_pct = parse_pct(
            mapping
                .service_health_pct
                .as_deref()
                .and_then(|h| get(&row, &headers, h)),
            "service_health_pct",
            &mut warnings,
        );

        let fp = fingerprint(
            &title,
            start_ts.as_deref().or(start_ts_raw.as_deref()),
            first_observed_ts
                .as_deref()
                .or(first_observed_ts_raw.as_deref()),
            ack_ts.as_deref().or(ack_ts_raw.as_deref()),
            mitigate_ts.as_deref().or(mitigate_ts_raw.as_deref()),
            resolve_ts.as_deref().or(resolve_ts_raw.as_deref()),
        );

        if !seen_fps.insert(fp.clone()) {
            conflicts.push(JiraImportConflict {
                row: row_idx,
                reason: "Duplicate fingerprint in import".to_string(),
                external_id: external_id.clone(),
                fingerprint: Some(fp),
            });
            skipped += 1;
            continue;
        }

        let existing_id = if let Some(ref ext) = external_id {
            find_existing_by_external_id(conn, ext)?
        } else {
            find_existing_by_fingerprint(conn, &fp)?
        };

        if let Some(id) = existing_id {
            let existing = load_incident_record_for_compare(conn, id)?;
            let incoming = IncidentRecord {
                external_id: external_id.clone(),
                fingerprint: fp.clone(), // placeholder; recomputed after merge
                title: title.clone(),
                description: description.clone(),
                severity: severity.clone(),
                detection_source: detection_source.clone(),
                vendor: vendor.clone(),
                service: service.clone(),
                impact_pct,
                service_health_pct,
                start_ts: start_ts.clone(),
                first_observed_ts: first_observed_ts.clone(),
                it_awareness_ts: it_awareness_ts.clone(),
                ack_ts: ack_ts.clone(),
                mitigate_ts: mitigate_ts.clone(),
                resolve_ts: resolve_ts.clone(),
                start_ts_raw: start_ts_raw.clone(),
                first_observed_ts_raw: first_observed_ts_raw.clone(),
                it_awareness_ts_raw: it_awareness_ts_raw.clone(),
                ack_ts_raw: ack_ts_raw.clone(),
                mitigate_ts_raw: mitigate_ts_raw.clone(),
                resolve_ts_raw: resolve_ts_raw.clone(),
            };

            let mut desired = merge_preserve_on_empty(
                &existing,
                incoming,
                start_update.clone(),
                first_observed_update.clone(),
                it_awareness_update.clone(),
                ack_update.clone(),
                mitigate_update.clone(),
                resolve_update.clone(),
            );
            // Recompute fingerprint based on the merged record so dedupe remains stable.
            desired.fingerprint = fingerprint(
                &desired.title,
                desired
                    .start_ts
                    .as_deref()
                    .or(desired.start_ts_raw.as_deref()),
                desired
                    .first_observed_ts
                    .as_deref()
                    .or(desired.first_observed_ts_raw.as_deref()),
                desired.ack_ts.as_deref().or(desired.ack_ts_raw.as_deref()),
                desired
                    .mitigate_ts
                    .as_deref()
                    .or(desired.mitigate_ts_raw.as_deref()),
                desired
                    .resolve_ts
                    .as_deref()
                    .or(desired.resolve_ts_raw.as_deref()),
            );

            if existing == desired {
                skipped += 1;
                continue;
            }

            let res = conn.execute(
                r#"
        UPDATE incidents SET
          external_id = ?1,
          fingerprint = ?2,
          title = ?3,
          description = ?4,
          severity = ?5,
          detection_source = ?6,
          vendor = ?7,
          service = ?8,
          impact_pct = ?9,
          service_health_pct = ?10,
          start_ts = ?11,
          first_observed_ts = ?12,
          it_awareness_ts = ?13,
          ack_ts = ?14,
          mitigate_ts = ?15,
          resolve_ts = ?16,
          start_ts_raw = ?17,
          first_observed_ts_raw = ?18,
          it_awareness_ts_raw = ?19,
          ack_ts_raw = ?20,
          mitigate_ts_raw = ?21,
          resolve_ts_raw = ?22,
          ingested_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
        WHERE id = ?23
        "#,
                rusqlite::params![
                    desired.external_id,
                    desired.fingerprint,
                    desired.title,
                    desired.description,
                    desired.severity,
                    desired.detection_source,
                    desired.vendor,
                    desired.service,
                    desired.impact_pct,
                    desired.service_health_pct,
                    desired.start_ts,
                    desired.first_observed_ts,
                    desired.it_awareness_ts,
                    desired.ack_ts,
                    desired.mitigate_ts,
                    desired.resolve_ts,
                    desired.start_ts_raw,
                    desired.first_observed_ts_raw,
                    desired.it_awareness_ts_raw,
                    desired.ack_ts_raw,
                    desired.mitigate_ts_raw,
                    desired.resolve_ts_raw,
                    id
                ],
            );

            match res {
                Ok(_) => updated += 1,
                Err(e) if is_unique_constraint_error(&e) => {
                    conflicts.push(JiraImportConflict {
                        row: row_idx,
                        reason: "Uniqueness constraint conflict while updating incident"
                            .to_string(),
                        external_id: desired.external_id.clone(),
                        fingerprint: Some(desired.fingerprint.clone()),
                    });
                    skipped += 1;
                }
                Err(e) => {
                    return Err(AppError::new(
                        "INGEST_JIRA_CSV_UPDATE_FAILED",
                        "Failed to update incident from Jira CSV",
                    )
                    .with_details(format!("row={row_idx}; err={e}")));
                }
            }
            continue;
        }

        let res = conn.execute(
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
        ?17, ?18, ?19, ?20, ?21, ?22,
        strftime('%Y-%m-%dT%H:%M:%fZ','now')
      )
      "#,
            rusqlite::params![
                external_id,
                fp,
                title,
                description,
                severity,
                detection_source,
                vendor,
                service,
                impact_pct,
                service_health_pct,
                start_ts,
                first_observed_ts,
                it_awareness_ts,
                ack_ts,
                mitigate_ts,
                resolve_ts,
                start_ts_raw,
                first_observed_ts_raw,
                it_awareness_ts_raw,
                ack_ts_raw,
                mitigate_ts_raw,
                resolve_ts_raw,
            ],
        );

        match res {
            Ok(_) => inserted += 1,
            Err(e) if is_unique_constraint_error(&e) => {
                conflicts.push(JiraImportConflict {
                    row: row_idx,
                    reason: "Uniqueness constraint conflict while inserting incident".to_string(),
                    external_id: external_id.clone(),
                    fingerprint: Some(fp.clone()),
                });
                skipped += 1;
            }
            Err(e) => {
                return Err(AppError::new(
                    "INGEST_JIRA_CSV_INSERT_FAILED",
                    "Failed to insert incident from Jira CSV",
                )
                .with_details(format!("row={row_idx}; err={e}")));
            }
        }
    }

    Ok(JiraImportSummary {
        inserted,
        updated,
        skipped,
        conflicts,
        warnings,
    })
}

pub fn ingest_jira_csv(
    conn: &mut Connection,
    csv_text: &str,
    mapping: &JiraCsvMapping,
) -> Result<JiraIngestSummary, AppError> {
    let summary = import_jira_csv(conn, csv_text, mapping)?;
    Ok(JiraIngestSummary {
        inserted: summary.inserted,
        warnings: summary.warnings,
    })
}
