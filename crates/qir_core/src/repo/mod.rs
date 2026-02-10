use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::domain::Incident;
use crate::error::AppError;
use crate::metrics::{compute_incident_metrics, IncidentMetrics};
use crate::validate::validate_incident;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Artifact {
    pub id: i64,
    pub incident_id: Option<i64>,
    pub kind: String,
    pub sha256: String,
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimelineEvent {
    pub id: i64,
    pub incident_id: Option<i64>,
    pub source: String,
    pub ts: Option<String>,
    pub author: Option<String>,
    pub kind: Option<String>,
    pub text: String,
    pub raw_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncidentDetail {
    pub incident: Incident,
    pub metrics: IncidentMetrics,
    pub warnings: Vec<crate::domain::ValidationWarning>,
    pub artifacts: Vec<Artifact>,
    pub timeline_events: Vec<TimelineEvent>,
}

pub fn list_incidents(conn: &Connection) -> Result<Vec<Incident>, AppError> {
    let mut stmt = conn
    .prepare(
      r#"
      SELECT
        id, external_id, fingerprint, title, description, severity,
        detection_source, vendor, service,
        impact_pct, service_health_pct,
        start_ts, first_observed_ts, it_awareness_ts, ack_ts, mitigate_ts, resolve_ts,
        start_ts_raw, first_observed_ts_raw, it_awareness_ts_raw, ack_ts_raw, mitigate_ts_raw, resolve_ts_raw
      FROM incidents
      "#,
    )
    .map_err(|e| {
      AppError::new("DB_QUERY_FAILED", "Failed to prepare incidents query").with_details(e.to_string())
    })?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Incident {
                id: row.get(0)?,
                external_id: row.get(1)?,
                fingerprint: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                severity: row.get(5)?,
                detection_source: row.get(6)?,
                vendor: row.get(7)?,
                service: row.get(8)?,
                impact_pct: row.get(9)?,
                service_health_pct: row.get(10)?,
                start_ts: row.get(11)?,
                first_observed_ts: row.get(12)?,
                it_awareness_ts: row.get(13)?,
                ack_ts: row.get(14)?,
                mitigate_ts: row.get(15)?,
                resolve_ts: row.get(16)?,
                start_ts_raw: row.get(17)?,
                first_observed_ts_raw: row.get(18)?,
                it_awareness_ts_raw: row.get(19)?,
                ack_ts_raw: row.get(20)?,
                mitigate_ts_raw: row.get(21)?,
                resolve_ts_raw: row.get(22)?,
            })
        })
        .map_err(|e| {
            AppError::new("DB_QUERY_FAILED", "Failed to query incidents")
                .with_details(e.to_string())
        })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| {
            AppError::new("DB_QUERY_FAILED", "Failed to decode incident row")
                .with_details(e.to_string())
        })?);
    }

    Ok(out)
}

pub fn count_incidents(conn: &Connection) -> Result<i64, AppError> {
    conn.query_row("SELECT COUNT(*) FROM incidents", [], |row| row.get(0))
        .map_err(|e| {
            AppError::new("DB_QUERY_FAILED", "Failed to count incidents")
                .with_details(e.to_string())
        })
}

pub fn get_incident(conn: &Connection, id: i64) -> Result<Incident, AppError> {
    let mut stmt = conn
        .prepare(
            r#"
      SELECT
        id, external_id, fingerprint, title, description, severity,
        detection_source, vendor, service,
        impact_pct, service_health_pct,
        start_ts, first_observed_ts, it_awareness_ts, ack_ts, mitigate_ts, resolve_ts,
        start_ts_raw, first_observed_ts_raw, it_awareness_ts_raw, ack_ts_raw, mitigate_ts_raw, resolve_ts_raw
      FROM incidents
      WHERE id = ?1
      "#,
        )
        .map_err(|e| {
            AppError::new("DB_QUERY_FAILED", "Failed to prepare incident query")
                .with_details(e.to_string())
        })?;

    stmt.query_row([id], |row| {
        Ok(Incident {
            id: row.get(0)?,
            external_id: row.get(1)?,
            fingerprint: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            severity: row.get(5)?,
            detection_source: row.get(6)?,
            vendor: row.get(7)?,
            service: row.get(8)?,
            impact_pct: row.get(9)?,
            service_health_pct: row.get(10)?,
            start_ts: row.get(11)?,
            first_observed_ts: row.get(12)?,
            it_awareness_ts: row.get(13)?,
            ack_ts: row.get(14)?,
            mitigate_ts: row.get(15)?,
            resolve_ts: row.get(16)?,
            start_ts_raw: row.get(17)?,
            first_observed_ts_raw: row.get(18)?,
            it_awareness_ts_raw: row.get(19)?,
            ack_ts_raw: row.get(20)?,
            mitigate_ts_raw: row.get(21)?,
            resolve_ts_raw: row.get(22)?,
        })
    })
    .map_err(|e| {
        AppError::new("DB_NOT_FOUND", "Incident not found").with_details(e.to_string())
    })
}

pub fn list_artifacts_for_incident(conn: &Connection, incident_id: i64) -> Result<Vec<Artifact>, AppError> {
    let mut stmt = conn
        .prepare(
            r#"
      SELECT
        id, incident_id, kind, sha256, filename, mime_type, text, created_at
      FROM artifacts
      WHERE incident_id = ?1
      ORDER BY created_at ASC, id ASC
      "#,
        )
        .map_err(|e| {
            AppError::new("DB_QUERY_FAILED", "Failed to prepare artifacts query")
                .with_details(e.to_string())
        })?;

    let rows = stmt
        .query_map([incident_id], |row| {
            Ok(Artifact {
                id: row.get(0)?,
                incident_id: row.get(1)?,
                kind: row.get(2)?,
                sha256: row.get(3)?,
                filename: row.get(4)?,
                mime_type: row.get(5)?,
                text: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| {
            AppError::new("DB_QUERY_FAILED", "Failed to query artifacts")
                .with_details(e.to_string())
        })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| {
            AppError::new("DB_QUERY_FAILED", "Failed to decode artifact row")
                .with_details(e.to_string())
        })?);
    }

    Ok(out)
}

pub fn list_timeline_events_for_incident(
    conn: &Connection,
    incident_id: i64,
) -> Result<Vec<TimelineEvent>, AppError> {
    let mut stmt = conn
        .prepare(
            r#"
      SELECT
        id, incident_id, source, ts, author, kind, text, raw_json, created_at
      FROM timeline_events
      WHERE incident_id = ?1
      ORDER BY (ts IS NULL) ASC, ts ASC, id ASC
      "#,
        )
        .map_err(|e| {
            AppError::new("DB_QUERY_FAILED", "Failed to prepare timeline events query")
                .with_details(e.to_string())
        })?;

    let rows = stmt
        .query_map([incident_id], |row| {
            Ok(TimelineEvent {
                id: row.get(0)?,
                incident_id: row.get(1)?,
                source: row.get(2)?,
                ts: row.get(3)?,
                author: row.get(4)?,
                kind: row.get(5)?,
                text: row.get(6)?,
                raw_json: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| {
            AppError::new("DB_QUERY_FAILED", "Failed to query timeline events")
                .with_details(e.to_string())
        })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| {
            AppError::new("DB_QUERY_FAILED", "Failed to decode timeline event row")
                .with_details(e.to_string())
        })?);
    }

    Ok(out)
}

pub fn get_incident_detail(conn: &Connection, incident_id: i64) -> Result<IncidentDetail, AppError> {
    let incident = get_incident(conn, incident_id)?;
    let (metrics, metric_warnings) = compute_incident_metrics(&incident);
    let mut warnings = validate_incident(&incident);
    warnings.extend(metric_warnings);
    warnings.sort_by(|a, b| a.code.cmp(&b.code));

    let artifacts = list_artifacts_for_incident(conn, incident_id)?;
    let timeline_events = list_timeline_events_for_incident(conn, incident_id)?;

    Ok(IncidentDetail {
        incident,
        metrics,
        warnings,
        artifacts,
        timeline_events,
    })
}
