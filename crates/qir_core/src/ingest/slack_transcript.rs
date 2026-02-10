use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime, UtcOffset};

use crate::domain::ValidationWarning;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlackIngestSummary {
    pub incident_id: i64,
    pub incident_created: bool,
    pub detected_format: String,
    pub inserted_events: usize,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlackPreview {
    pub detected_format: String,
    pub line_count: usize,
    pub message_count: usize,
    pub warnings: Vec<ValidationWarning>,
}

fn canonicalize_rfc3339_utc(dt: OffsetDateTime) -> Option<String> {
    let utc = dt.to_offset(UtcOffset::UTC);
    utc.format(&Rfc3339).ok()
}

fn split_line_rfc3339ish(line: &str) -> (Option<String>, Option<String>, String) {
    // Supported minimal formats:
    // - "<rfc3339> - <author>: <text>"
    // - "<rfc3339> <author>: <text>"
    //
    // If no RFC3339 prefix is found, ts/author may be None.
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return (None, None, String::new());
    }

    // Try tokenizing by first whitespace, treating the first token as timestamp.
    let mut parts = trimmed.splitn(2, ' ');
    let first = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("").trim();

    let ts = OffsetDateTime::parse(first, &Rfc3339)
        .ok()
        .and_then(canonicalize_rfc3339_utc);
    let payload = if ts.is_some() { rest } else { trimmed };

    // Remove optional leading "- " after timestamp.
    let payload = payload.strip_prefix("- ").unwrap_or(payload).trim();

    // Parse "author: text"
    if let Some((author, text)) = payload.split_once(':') {
        let author = author.trim();
        let text = text.trim();
        let author = if author.is_empty() {
            None
        } else {
            Some(author.to_string())
        };
        (ts, author, text.to_string())
    } else {
        (ts, None, payload.to_string())
    }
}

fn parse_slack_ts_seconds_to_rfc3339(ts: &str) -> Option<String> {
    // Slack JSON export typically uses a string seconds-with-fraction, e.g. "1700000000.000100".
    // We parse deterministically without floats.
    let trimmed = ts.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (secs_s, frac_s) = trimmed.split_once('.').unwrap_or((trimmed, "0"));
    let secs: i64 = secs_s.parse().ok()?;

    let mut frac = frac_s.chars().take(9).collect::<String>();
    while frac.len() < 9 {
        frac.push('0');
    }
    let nanos: i64 = frac.parse().ok()?;

    let base = OffsetDateTime::from_unix_timestamp(secs).ok()?;
    let dt = base + Duration::nanoseconds(nanos);
    canonicalize_rfc3339_utc(dt)
}

fn detect_format(transcript: &str) -> String {
    let s = transcript.trim_start();
    if s.starts_with('[') || s.starts_with('{') {
        // If it parses as JSON array, treat it as Slack JSON export.
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
            if v.is_array() {
                return "slack_json_export".to_string();
            }
        }
    }

    // If first non-empty line starts with an RFC3339 timestamp token, treat as line-oriented.
    for line in transcript.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let first = t.split_whitespace().next().unwrap_or("");
        if OffsetDateTime::parse(first, &Rfc3339).is_ok() {
            return "line_rfc3339".to_string();
        }
        break;
    }

    "raw_lines".to_string()
}

fn ensure_target_incident(
    conn: &mut Connection,
    incident_id: Option<i64>,
    new_incident_title: Option<&str>,
    transcript: &str,
) -> Result<(i64, bool), AppError> {
    match (incident_id, new_incident_title) {
        (Some(_), Some(_)) => Err(AppError::new(
            "INGEST_SLACK_TARGET_AMBIGUOUS",
            "Provide either incident_id or new_incident_title, not both",
        )),
        (Some(id), None) => {
            let exists: Option<i64> = conn
                .query_row("SELECT id FROM incidents WHERE id = ?1", [id], |row| {
                    row.get(0)
                })
                .optional()
                .map_err(|e| {
                    AppError::new("DB_QUERY_FAILED", "Failed to check incident exists")
                        .with_details(e.to_string())
                })?;
            if exists.is_none() {
                return Err(AppError::new(
                    "INGEST_SLACK_INCIDENT_NOT_FOUND",
                    "Incident not found for provided incident_id",
                )
                .with_details(format!("incident_id={id}")));
            }
            Ok((id, false))
        }
        (None, Some(title)) => {
            let title = title.trim();
            if title.is_empty() {
                return Err(AppError::new(
                    "INGEST_SLACK_TITLE_REQUIRED",
                    "New incident title is required",
                ));
            }

            // Deterministic Slack-only shell fingerprint: title + transcript hash.
            let transcript_hash = hex::encode(Sha256::digest(transcript.as_bytes()));
            let payload = format!(
                "slack_shell|title={}|sha={}",
                title.to_lowercase(),
                transcript_hash
            );
            let fp = hex::encode(Sha256::digest(payload.as_bytes()));

            conn.execute(
                r#"
          INSERT INTO incidents(
            external_id, fingerprint, title, description, severity,
            impact_pct, service_health_pct,
            start_ts, first_observed_ts, it_awareness_ts, ack_ts, mitigate_ts, resolve_ts,
            ingested_at
          ) VALUES (
            NULL, ?1, ?2, NULL, NULL,
            NULL, NULL,
            NULL, NULL, NULL, NULL, NULL, NULL,
            strftime('%Y-%m-%dT%H:%M:%fZ','now')
          )
          "#,
                rusqlite::params![fp, title],
            )
            .map_err(|e| {
                AppError::new(
                    "INGEST_SLACK_INCIDENT_CREATE_FAILED",
                    "Failed to create Slack-only incident shell",
                )
                .with_details(e.to_string())
            })?;

            let id = conn.last_insert_rowid();
            Ok((id, true))
        }
        (None, None) => Err(AppError::new(
            "INGEST_SLACK_TARGET_REQUIRED",
            "Select an incident or provide a new incident title",
        )),
    }
}

pub fn preview_slack_transcript_text(transcript: &str) -> SlackPreview {
    let mut warnings = Vec::new();
    let detected_format = detect_format(transcript);
    let line_count = transcript.lines().count();

    let message_count = match detected_format.as_str() {
        "slack_json_export" => serde_json::from_str::<serde_json::Value>(transcript)
            .ok()
            .and_then(|v| v.as_array().map(|a| a.len()))
            .unwrap_or(0),
        _ => transcript.lines().filter(|l| !l.trim().is_empty()).count(),
    };

    if detected_format == "raw_lines" {
        warnings.push(
            ValidationWarning::new(
                "INGEST_SLACK_FORMAT_UNKNOWN",
                "Unknown Slack transcript format; ingest will preserve raw lines",
            )
            .with_details("detected_format=raw_lines"),
        );
    }

    SlackPreview {
        detected_format,
        line_count,
        message_count,
        warnings,
    }
}

pub fn ingest_slack_transcript_text(
    conn: &mut Connection,
    incident_id: Option<i64>,
    new_incident_title: Option<&str>,
    transcript: &str,
) -> Result<SlackIngestSummary, AppError> {
    let mut warnings = Vec::new();
    let detected_format = detect_format(transcript);

    let (target_incident_id, incident_created) =
        ensure_target_incident(conn, incident_id, new_incident_title, transcript)?;

    let mut inserted_events = 0usize;
    match detected_format.as_str() {
        "slack_json_export" => {
            let v: serde_json::Value = serde_json::from_str(transcript).map_err(|e| {
                AppError::new(
                    "INGEST_SLACK_JSON_PARSE_FAILED",
                    "Failed to parse Slack JSON export",
                )
                .with_details(e.to_string())
            })?;
            let arr = v.as_array().ok_or_else(|| {
                AppError::new(
                    "INGEST_SLACK_JSON_INVALID",
                    "Slack JSON export must be a JSON array",
                )
            })?;

            for (idx, item) in arr.iter().enumerate() {
                let obj = match item.as_object() {
                    Some(o) => o,
                    None => {
                        warnings.push(
                            ValidationWarning::new(
                                "INGEST_SLACK_JSON_ROW_SKIPPED",
                                "Skipped non-object entry in Slack JSON export",
                            )
                            .with_details(format!("index={idx}")),
                        );
                        continue;
                    }
                };

                let text = obj
                    .get("text")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .trim();
                if text.is_empty() {
                    continue;
                }

                let author = obj
                    .get("user")
                    .and_then(|u| u.as_str())
                    .or_else(|| obj.get("username").and_then(|u| u.as_str()))
                    .map(|s| s.to_string());

                let ts = obj
                    .get("ts")
                    .and_then(|t| t.as_str())
                    .and_then(parse_slack_ts_seconds_to_rfc3339);

                if ts.is_none() {
                    warnings.push(
                        ValidationWarning::new(
                            "INGEST_SLACK_TS_UNKNOWN",
                            "Slack JSON message missing/invalid timestamp",
                        )
                        .with_details(format!("index={idx}")),
                    );
                }

                let raw_json = serde_json::to_string(item).ok();

                conn.execute(
                    r#"
              INSERT INTO timeline_events(
                incident_id, source, ts, author, kind, text, raw_json, created_at
              ) VALUES (
                ?1, 'slack', ?2, ?3, 'message', ?4, ?5,
                strftime('%Y-%m-%dT%H:%M:%fZ','now')
              )
              "#,
                    rusqlite::params![target_incident_id, ts, author, text, raw_json],
                )
                .map_err(|e| {
                    AppError::new(
                        "INGEST_SLACK_INSERT_FAILED",
                        "Failed to insert Slack timeline event",
                    )
                    .with_details(format!("index={idx}; err={e}"))
                })?;
                inserted_events += 1;
            }
        }
        "line_rfc3339" => {
            for (idx, line) in transcript.lines().enumerate() {
                let (ts, author, text) = split_line_rfc3339ish(line);
                if text.trim().is_empty() {
                    continue;
                }
                if ts.is_none() {
                    warnings.push(
                        ValidationWarning::new(
                            "INGEST_SLACK_TS_UNKNOWN",
                            "Slack line missing RFC3339 timestamp",
                        )
                        .with_details(format!("line={idx}")),
                    );
                }

                conn.execute(
                    r#"
              INSERT INTO timeline_events(
                incident_id, source, ts, author, kind, text, raw_json, created_at
              ) VALUES (
                ?1, 'slack', ?2, ?3, 'message', ?4, NULL,
                strftime('%Y-%m-%dT%H:%M:%fZ','now')
              )
              "#,
                    rusqlite::params![target_incident_id, ts, author, text],
                )
                .map_err(|e| {
                    AppError::new(
                        "INGEST_SLACK_INSERT_FAILED",
                        "Failed to insert Slack timeline event",
                    )
                    .with_details(format!("line={idx}; err={e}"))
                })?;
                inserted_events += 1;
            }
        }
        _ => {
            warnings.push(
                ValidationWarning::new(
                    "INGEST_SLACK_FORMAT_UNKNOWN",
                    "Unknown Slack transcript format; ingest preserved raw lines without timestamps",
                )
                .with_details("detected_format=raw_lines"),
            );

            for (idx, line) in transcript.lines().enumerate() {
                let t = line.trim();
                if t.is_empty() {
                    continue;
                }

                let (author, text) = if let Some((a, rest)) = t.split_once(':') {
                    let a = a.trim();
                    let rest = rest.trim();
                    let a = if a.is_empty() {
                        None
                    } else {
                        Some(a.to_string())
                    };
                    (a, rest.to_string())
                } else {
                    (None, t.to_string())
                };

                conn.execute(
                    r#"
              INSERT INTO timeline_events(
                incident_id, source, ts, author, kind, text, raw_json, created_at
              ) VALUES (
                ?1, 'slack', NULL, ?2, 'message', ?3, NULL,
                strftime('%Y-%m-%dT%H:%M:%fZ','now')
              )
              "#,
                    rusqlite::params![target_incident_id, author, text],
                )
                .map_err(|e| {
                    AppError::new(
                        "INGEST_SLACK_INSERT_FAILED",
                        "Failed to insert Slack timeline event",
                    )
                    .with_details(format!("line={idx}; err={e}"))
                })?;
                inserted_events += 1;
            }
        }
    }

    Ok(SlackIngestSummary {
        incident_id: target_incident_id,
        incident_created,
        detected_format,
        inserted_events,
        warnings,
    })
}
