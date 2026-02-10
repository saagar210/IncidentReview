use std::fs;
use std::path::PathBuf;

use qir_core::error::AppError;

use super::model::{EvidenceChunkMeta, EvidenceSourceType, EvidenceTimeRange};
use super::store::{normalize_text, EvidenceSourceRecord, EvidenceStore};

#[derive(Debug, Clone)]
pub struct ChunkDraft {
    pub ordinal: u32,
    pub text: String,
    pub token_count_est: u32,
    pub meta: EvidenceChunkMeta,
}

fn chunk_text_by_paragraphs(text: &str, kind: &str, max_chars: usize) -> Vec<ChunkDraft> {
    let normalized = normalize_text(text);
    let mut paras: Vec<String> = normalized
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect();
    if paras.is_empty() {
        paras.push(normalized.trim().to_string());
    }

    let mut out = Vec::new();
    let mut ordinal: u32 = 0;
    let mut buf = String::new();
    for p in paras {
        let add_len = if buf.is_empty() { p.len() } else { 2 + p.len() };
        if !buf.is_empty() && buf.len() + add_len > max_chars {
            let text = buf.clone();
            out.push(ChunkDraft {
                ordinal,
                token_count_est: text.len().min(u32::MAX as usize) as u32,
                text,
                meta: EvidenceChunkMeta {
                    kind: kind.to_string(),
                    incident_keys: None,
                    time_range: None,
                },
            });
            ordinal += 1;
            buf.clear();
        }
        if !buf.is_empty() {
            buf.push_str("\n\n");
        }
        buf.push_str(&p);
    }
    if !buf.trim().is_empty() {
        let text = buf;
        out.push(ChunkDraft {
            ordinal,
            token_count_est: text.len().min(u32::MAX as usize) as u32,
            text,
            meta: EvidenceChunkMeta {
                kind: kind.to_string(),
                incident_keys: None,
                time_range: None,
            },
        });
    }

    out
}

pub(crate) fn build_chunks_for_source(
    store: &EvidenceStore,
    rec: &EvidenceSourceRecord,
) -> Result<Vec<ChunkDraft>, AppError> {
    match rec.source.source_type {
        EvidenceSourceType::FreeformText | EvidenceSourceType::SlackTranscript | EvidenceSourceType::IncidentReportMd => {
            let text = store.read_source_text_for_chunking(rec)?;
            Ok(chunk_text_by_paragraphs(&text, "paragraph", 1600))
        }
        EvidenceSourceType::SanitizedExport => {
            let dir = rec
                .source
                .origin
                .path
                .as_ref()
                .ok_or_else(|| AppError::new("AI_EVIDENCE_SOURCE_INVALID", "Sanitized export requires a directory path"))?;
            let root = PathBuf::from(dir);
            if !root.is_dir() {
                return Err(AppError::new(
                    "AI_EVIDENCE_SOURCE_INVALID",
                    "Sanitized export origin must be a directory",
                )
                .with_details(format!("path={}", root.display())));
            }

            // Read the sanitized export artifacts produced by qir_core.
            let incidents_path = root.join("incidents.json");
            let events_path = root.join("timeline_events.json");
            let warnings_path = root.join("warnings.json");

            let incidents_raw = fs::read_to_string(&incidents_path).map_err(|e| {
                AppError::new(
                    "AI_EVIDENCE_SOURCE_INVALID",
                    "Failed to read incidents.json from sanitized export",
                )
                .with_details(format!("path={}; err={}", incidents_path.display(), e))
            })?;
            let events_raw = fs::read_to_string(&events_path).map_err(|e| {
                AppError::new(
                    "AI_EVIDENCE_SOURCE_INVALID",
                    "Failed to read timeline_events.json from sanitized export",
                )
                .with_details(format!("path={}; err={}", events_path.display(), e))
            })?;
            let warnings_raw = fs::read_to_string(&warnings_path).map_err(|e| {
                AppError::new(
                    "AI_EVIDENCE_SOURCE_INVALID",
                    "Failed to read warnings.json from sanitized export",
                )
                .with_details(format!("path={}; err={}", warnings_path.display(), e))
            })?;

            let mut incidents: Vec<qir_core::sanitize::SanitizedIncident> =
                serde_json::from_str(&incidents_raw).map_err(|e| {
                    AppError::new(
                        "AI_EVIDENCE_SOURCE_INVALID",
                        "Failed to decode incidents.json from sanitized export",
                    )
                    .with_details(e.to_string())
                })?;
            let events: Vec<qir_core::sanitize::SanitizedTimelineEvent> =
                serde_json::from_str(&events_raw).map_err(|e| {
                    AppError::new(
                        "AI_EVIDENCE_SOURCE_INVALID",
                        "Failed to decode timeline_events.json from sanitized export",
                    )
                    .with_details(e.to_string())
                })?;
            let warnings: Vec<qir_core::sanitize::SanitizedWarning> =
                serde_json::from_str(&warnings_raw).map_err(|e| {
                    AppError::new(
                        "AI_EVIDENCE_SOURCE_INVALID",
                        "Failed to decode warnings.json from sanitized export",
                    )
                    .with_details(e.to_string())
                })?;

            // Stable ordering by incident_key.
            incidents.sort_by(|a, b| a.incident_key.cmp(&b.incident_key));

            let mut out = Vec::new();
            for (i, inc) in incidents.iter().enumerate() {
                let key = inc.incident_key.clone();
                let evs = events
                    .iter()
                    .filter(|e| e.incident_key == key)
                    .collect::<Vec<_>>();
                let ws = warnings
                    .iter()
                    .filter(|w| w.incident_key == key)
                    .collect::<Vec<_>>();

                let mut lines = Vec::new();
                lines.push(format!("Incident Key: {}", inc.incident_key));
                if let Some(s) = inc.severity.as_ref() {
                    lines.push(format!("Severity: {s}"));
                }
                if let Some(s) = inc.detection_source.as_ref() {
                    lines.push(format!("Detection Source: {s}"));
                }
                if let Some(s) = inc.vendor.as_ref() {
                    lines.push(format!("Vendor: {s}"));
                }
                if let Some(s) = inc.service.as_ref() {
                    lines.push(format!("Service: {s}"));
                }
                if let Some(v) = inc.impact_pct {
                    lines.push(format!("Impact %: {v}"));
                }
                if let Some(v) = inc.service_health_pct {
                    lines.push(format!("Service Health %: {v}"));
                }

                lines.push("Timestamps (RFC3339, nullable):".to_string());
                lines.push(format!("  start_ts: {}", inc.start_ts.clone().unwrap_or_else(|| "NULL".to_string())));
                lines.push(format!(
                    "  first_observed_ts: {}",
                    inc.first_observed_ts.clone().unwrap_or_else(|| "NULL".to_string())
                ));
                lines.push(format!(
                    "  it_awareness_ts: {}",
                    inc.it_awareness_ts.clone().unwrap_or_else(|| "NULL".to_string())
                ));
                lines.push(format!("  ack_ts: {}", inc.ack_ts.clone().unwrap_or_else(|| "NULL".to_string())));
                lines.push(format!(
                    "  mitigate_ts: {}",
                    inc.mitigate_ts.clone().unwrap_or_else(|| "NULL".to_string())
                ));
                lines.push(format!(
                    "  resolve_ts: {}",
                    inc.resolve_ts.clone().unwrap_or_else(|| "NULL".to_string())
                ));

                lines.push("Deterministic metrics (seconds, nullable):".to_string());
                lines.push(format!("  mttd_seconds: {}", opt_i64(inc.metrics.mttd_seconds)));
                lines.push(format!(
                    "  it_awareness_lag_seconds: {}",
                    opt_i64(inc.metrics.it_awareness_lag_seconds)
                ));
                lines.push(format!("  mtta_seconds: {}", opt_i64(inc.metrics.mtta_seconds)));
                lines.push(format!(
                    "  time_to_mitigation_seconds: {}",
                    opt_i64(inc.metrics.time_to_mitigation_seconds)
                ));
                lines.push(format!("  mttr_seconds: {}", opt_i64(inc.metrics.mttr_seconds)));

                lines.push(format!("Warning count: {}", inc.warning_count));
                if !ws.is_empty() {
                    lines.push("Warnings (codes):".to_string());
                    let mut codes = ws.iter().map(|w| w.code.clone()).collect::<Vec<_>>();
                    codes.sort();
                    codes.dedup();
                    for c in codes {
                        lines.push(format!("  - {c}"));
                    }
                }
                if !evs.is_empty() {
                    lines.push("Timeline events (sanitized):".to_string());
                    for e in evs {
                        let ts = e.ts.clone().unwrap_or_else(|| "NULL".to_string());
                        let kind = e.kind.clone().unwrap_or_else(|| "NULL".to_string());
                        // Note: sanitized export does not include text; it is intentionally redacted at source.
                        lines.push(format!(
                            "  - ts={ts}; source={}; kind={kind}; text_redacted={}",
                            e.source, e.text_redacted
                        ));
                    }
                }

                let text = lines.join("\n");
                let meta = EvidenceChunkMeta {
                    kind: "sanitized_incident_bundle".to_string(),
                    incident_keys: Some(vec![key.clone()]),
                    time_range: Some(EvidenceTimeRange {
                        start_ts: inc.start_ts.clone(),
                        end_ts: inc.resolve_ts.clone(),
                    }),
                };
                out.push(ChunkDraft {
                    ordinal: i as u32,
                    token_count_est: text.len().min(u32::MAX as usize) as u32,
                    text,
                    meta,
                });
            }

            Ok(out)
        }
    }
}

fn opt_i64(v: Option<i64>) -> String {
    match v {
        Some(x) => x.to_string(),
        None => "NULL".to_string(),
    }
}
