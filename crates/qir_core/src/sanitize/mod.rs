use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::metrics;
use crate::repo;
use crate::validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedExportManifest {
    pub manifest_version: u32,
    pub app_version: String,
    pub export_time: String,
    pub incident_count: i64,
    pub files: Vec<SanitizedFileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedFileInfo {
    pub filename: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedExportResult {
    pub export_dir: String,
    pub incident_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedIncident {
    pub incident_key: String,
    pub severity: Option<String>,
    pub detection_source: Option<String>,
    pub vendor: Option<String>,
    pub service: Option<String>,
    pub impact_pct: Option<i64>,
    pub service_health_pct: Option<i64>,
    pub start_ts: Option<String>,
    pub first_observed_ts: Option<String>,
    pub it_awareness_ts: Option<String>,
    pub ack_ts: Option<String>,
    pub mitigate_ts: Option<String>,
    pub resolve_ts: Option<String>,
    pub metrics: SanitizedMetrics,
    pub warning_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedMetrics {
    pub mttd_seconds: Option<i64>,
    pub it_awareness_lag_seconds: Option<i64>,
    pub mtta_seconds: Option<i64>,
    pub time_to_mitigation_seconds: Option<i64>,
    pub mttr_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedTimelineEvent {
    pub incident_key: String,
    pub source: String,
    pub ts: Option<String>,
    pub kind: Option<String>,
    pub text_redacted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedWarning {
    pub incident_key: String,
    pub code: String,
}

fn filename_safe_timestamp(export_time: &str) -> String {
    export_time
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => c,
            _ => '_',
        })
        .collect()
}

fn sha256_file_hex(path: &Path) -> Result<(String, u64), AppError> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut f = fs::File::open(path).map_err(|e| {
        AppError::new("EXPORT_SANITIZED_FILE_OPEN_FAILED", "Failed to open file for hashing")
            .with_details(format!("path={}: {}", path.display(), e))
    })?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    let mut total: u64 = 0;
    loop {
        let n = f.read(&mut buf).map_err(|e| {
            AppError::new("EXPORT_SANITIZED_FILE_READ_FAILED", "Failed to read file for hashing")
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

fn pseudo_map_alpha(prefix: &str, values: Vec<String>) -> Vec<(String, String)> {
    // values is already sorted/deduped.
    values
        .into_iter()
        .enumerate()
        .map(|(idx, v)| {
            let label = format!("{prefix}_{}", (b'A' + (idx as u8)) as char);
            (v, label)
        })
        .collect()
}

fn pseudo_map_numeric(prefix: &str, values: Vec<String>) -> Vec<(String, String)> {
    values
        .into_iter()
        .enumerate()
        .map(|(idx, v)| (v, format!("{prefix}_{:03}", idx + 1)))
        .collect()
}

pub fn export_sanitized_dataset(
    conn: &Connection,
    destination_dir: &Path,
    export_time: &str,
    app_version: &str,
) -> Result<SanitizedExportResult, AppError> {
    if !destination_dir.is_dir() {
        return Err(AppError::new(
            "EXPORT_SANITIZED_DEST_NOT_DIR",
            "Sanitized export destination must be an existing directory",
        )
        .with_details(destination_dir.display().to_string()));
    }

    let folder_name = format!(
        "IncidentReviewSanitized_{}",
        filename_safe_timestamp(export_time)
    );
    let export_dir = destination_dir.join(folder_name);
    if export_dir.exists() {
        return Err(AppError::new(
            "EXPORT_SANITIZED_DEST_EXISTS",
            "Sanitized export destination folder already exists",
        )
        .with_details(export_dir.display().to_string()));
    }
    fs::create_dir_all(&export_dir).map_err(|e| {
        AppError::new(
            "EXPORT_SANITIZED_MKDIR_FAILED",
            "Failed to create sanitized export directory",
        )
        .with_details(format!("path={}: {}", export_dir.display(), e))
    })?;

    // Pull incidents and build deterministic incident_key mapping.
    let incidents = repo::list_incidents(conn)?;
    let mut ids: Vec<i64> = incidents.iter().map(|i| i.id).collect();
    ids.sort();
    let incident_key_by_id: Vec<(i64, String)> = ids
        .into_iter()
        .enumerate()
        .map(|(idx, id)| (id, format!("INC_{:03}", idx + 1)))
        .collect();

    let key_for = |id: i64| -> String {
        incident_key_by_id
            .iter()
            .find(|(x, _)| *x == id)
            .map(|(_, k)| k.clone())
            .unwrap_or_else(|| "INC_UNKNOWN".to_string())
    };

    // Deterministic pseudonymization maps for categories.
    let mut vendors: Vec<String> = incidents
        .iter()
        .filter_map(|i| i.vendor.clone())
        .filter(|s| !s.trim().is_empty())
        .collect();
    vendors.sort();
    vendors.dedup();
    let vendor_map = pseudo_map_alpha("VENDOR", vendors);

    let mut services: Vec<String> = incidents
        .iter()
        .filter_map(|i| i.service.clone())
        .filter(|s| !s.trim().is_empty())
        .collect();
    services.sort();
    services.dedup();
    let service_map = pseudo_map_numeric("SERVICE", services);

    let mut detections: Vec<String> = incidents
        .iter()
        .filter_map(|i| i.detection_source.clone())
        .filter(|s| !s.trim().is_empty())
        .collect();
    detections.sort();
    detections.dedup();
    let detection_map = pseudo_map_numeric("DETECT", detections);

    let map_lookup = |m: &[(String, String)], v: &Option<String>| -> Option<String> {
        let vv = v.as_ref()?;
        for (raw, pseudo) in m {
            if raw == vv {
                return Some(pseudo.clone());
            }
        }
        None
    };

    // Validator payload (we export codes only to avoid leaking raw details).
    let validation = validate::validate_all_incidents(conn)?;
    let mut warning_count_by_id: Vec<(i64, i64)> = validation
        .iter()
        .map(|i| (i.id, i.warnings.len() as i64))
        .collect();
    warning_count_by_id.sort_by_key(|(id, _)| *id);

    let warnings_flat: Vec<SanitizedWarning> = validation
        .iter()
        .flat_map(|item| {
            let incident_key = key_for(item.id);
            item.warnings.iter().map(move |w| SanitizedWarning {
                incident_key: incident_key.clone(),
                code: w.code.clone(),
            })
        })
        .collect();

    let mut sanitized_incidents = Vec::new();
    for i in &incidents {
        let (m, _metric_warnings) = metrics::compute_incident_metrics(i);
        let warning_count = warning_count_by_id
            .iter()
            .find(|(id, _)| *id == i.id)
            .map(|(_, c)| *c)
            .unwrap_or(0);

        sanitized_incidents.push(SanitizedIncident {
            incident_key: key_for(i.id),
            severity: i.severity.clone(),
            detection_source: map_lookup(&detection_map, &i.detection_source),
            vendor: map_lookup(&vendor_map, &i.vendor),
            service: map_lookup(&service_map, &i.service),
            impact_pct: i.impact_pct,
            service_health_pct: i.service_health_pct,
            start_ts: i.start_ts.clone(),
            first_observed_ts: i.first_observed_ts.clone(),
            it_awareness_ts: i.it_awareness_ts.clone(),
            ack_ts: i.ack_ts.clone(),
            mitigate_ts: i.mitigate_ts.clone(),
            resolve_ts: i.resolve_ts.clone(),
            metrics: SanitizedMetrics {
                mttd_seconds: m.mttd_seconds,
                it_awareness_lag_seconds: m.it_awareness_lag_seconds,
                mtta_seconds: m.mtta_seconds,
                time_to_mitigation_seconds: m.time_to_mitigation_seconds,
                mttr_seconds: m.mttr_seconds,
            },
            warning_count,
        });
    }
    sanitized_incidents.sort_by(|a, b| a.incident_key.cmp(&b.incident_key));

    // Timeline events, with text redacted.
    let events = repo::list_timeline_events(conn)?;
    let mut sanitized_events: Vec<SanitizedTimelineEvent> = events
        .into_iter()
        .map(|e| SanitizedTimelineEvent {
            incident_key: e.incident_id.map(key_for).unwrap_or_else(|| "INC_UNKNOWN".to_string()),
            source: e.source,
            ts: e.ts,
            kind: e.kind,
            text_redacted: true,
        })
        .collect();
    sanitized_events.sort_by(|a, b| {
        a.incident_key
            .cmp(&b.incident_key)
            .then_with(|| a.ts.cmp(&b.ts))
            .then_with(|| a.source.cmp(&b.source))
    });

    // Write files deterministically.
    fn write_json<T: Serialize>(dir: &Path, name: &str, value: &T) -> Result<PathBuf, AppError> {
        let path = dir.join(name);
        let json = serde_json::to_string_pretty(value).map_err(|e| {
            AppError::new("EXPORT_SANITIZED_ENCODE_FAILED", "Failed to encode sanitized JSON")
                .with_details(e.to_string())
        })?;
        fs::write(&path, json.as_bytes()).map_err(|e| {
            AppError::new("EXPORT_SANITIZED_WRITE_FAILED", "Failed to write sanitized export file")
                .with_details(format!("path={}: {}", path.display(), e))
        })?;
        Ok(path)
    }

    let incidents_path = write_json(&export_dir, "incidents.json", &sanitized_incidents)?;
    let events_path = write_json(&export_dir, "timeline_events.json", &sanitized_events)?;
    let warnings_path = write_json(&export_dir, "warnings.json", &warnings_flat)?;

    let mut files = Vec::new();
    for (name, path) in [
        ("incidents.json", incidents_path),
        ("timeline_events.json", events_path),
        ("warnings.json", warnings_path),
    ] {
        let (sha, bytes) = sha256_file_hex(&path)?;
        files.push(SanitizedFileInfo {
            filename: name.to_string(),
            bytes,
            sha256: sha,
        });
    }
    files.sort_by(|a, b| a.filename.cmp(&b.filename));

    let manifest = SanitizedExportManifest {
        manifest_version: 1,
        app_version: app_version.to_string(),
        export_time: export_time.to_string(),
        incident_count: sanitized_incidents.len() as i64,
        files,
    };
    let manifest_path = export_dir.join("sanitized_manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| {
        AppError::new(
            "EXPORT_SANITIZED_ENCODE_FAILED",
            "Failed to encode sanitized export manifest",
        )
        .with_details(e.to_string())
    })?;
    fs::write(&manifest_path, manifest_json.as_bytes()).map_err(|e| {
        AppError::new(
            "EXPORT_SANITIZED_WRITE_FAILED",
            "Failed to write sanitized export manifest",
        )
        .with_details(format!("path={}: {}", manifest_path.display(), e))
    })?;

    Ok(SanitizedExportResult {
        export_dir: export_dir.to_string_lossy().to_string(),
        incident_count: sanitized_incidents.len() as i64,
    })
}
