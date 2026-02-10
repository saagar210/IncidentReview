use rusqlite::Connection;

use crate::analytics::build_dashboard_payload_v2;
use crate::error::AppError;
use crate::metrics::compute_incident_metrics;
use crate::repo::list_incidents;
use crate::validate::validate_incident;

fn format_duration_seconds(secs: Option<i64>) -> String {
    match secs {
        None => "UNKNOWN".to_string(),
        Some(s) => {
            let minutes = s / 60;
            let rem = s % 60;
            if minutes >= 60 {
                let hours = minutes / 60;
                let m = minutes % 60;
                format!("{hours}h {m}m")
            } else if minutes > 0 {
                format!("{minutes}m {rem}s")
            } else {
                format!("{rem}s")
            }
        }
    }
}

fn percentile(values: &mut Vec<i64>, pct_num: i64, pct_den: i64) -> Option<i64> {
    if values.is_empty() {
        return None;
    }
    values.sort();
    let n = values.len() as i64;
    if n == 1 {
        return Some(values[0]);
    }
    // Deterministic "nearest-rank on 0..n-1": idx = floor((n-1) * pct).
    let idx = ((n - 1) * pct_num) / pct_den;
    values.get(idx as usize).copied()
}

fn metric_summary_row(name: &str, vals: &[Option<i64>], total: i64) -> String {
    let mut known = Vec::new();
    for v in vals {
        if let Some(x) = v {
            known.push(*x);
        }
    }
    let known_count = known.len() as i64;
    let p50 = percentile(&mut known.clone(), 50, 100);
    let p90 = percentile(&mut known.clone(), 90, 100);
    format!(
        "| {name} | {known_count}/{total} | {} | {} |\n",
        format_duration_seconds(p50),
        format_duration_seconds(p90)
    )
}

/// Generate a deterministic QIR Markdown report from the current DB contents.
///
/// Ordering rules are stable so outputs are snapshot-testable.
pub fn generate_qir_markdown(conn: &Connection) -> Result<String, AppError> {
    let dash = build_dashboard_payload_v2(conn)?;
    let incidents = list_incidents(conn)?;
    let total = incidents.len() as i64;

    // Deterministic ordering: external_id, then title, then id.
    let mut incident_rows = incidents.clone();
    incident_rows.sort_by(|a, b| {
        (
            a.external_id.clone().unwrap_or_default(),
            a.title.clone(),
            a.id,
        )
            .cmp(&(
                b.external_id.clone().unwrap_or_default(),
                b.title.clone(),
                b.id,
            ))
    });

    let mut mttd = Vec::new();
    let mut awareness = Vec::new();
    let mut mtta = Vec::new();
    let mut ttm = Vec::new();
    let mut mttr = Vec::new();

    let mut warnings_by_incident: Vec<(i64, Vec<String>)> = Vec::new();

    for inc in &incident_rows {
        let (m, metric_warnings) = compute_incident_metrics(inc);
        let val_warnings = validate_incident(inc);

        mttd.push(m.mttd_seconds);
        awareness.push(m.it_awareness_lag_seconds);
        mtta.push(m.mtta_seconds);
        ttm.push(m.time_to_mitigation_seconds);
        mttr.push(m.mttr_seconds);

        let mut codes: Vec<String> = metric_warnings
            .into_iter()
            .chain(val_warnings.into_iter())
            .map(|w| w.code)
            .collect();
        codes.sort();
        warnings_by_incident.push((inc.id, codes));
    }

    let mut out = String::new();
    out.push_str("# Quarterly Incident Review (QIR)\n\n");
    out.push_str(&format!("Incident count: **{}**\n\n", dash.incident_count));

    out.push_str("## Executive summary\n\n");
    out.push_str(
        "- This report is **deterministic** and generated locally from the incident database.\n",
    );
    out.push_str(&format!(
        "- Total incidents in scope: **{}**\n",
        dash.incident_count
    ));
    let mut mttr_known: Vec<i64> = mttr.iter().filter_map(|x| *x).collect();
    let mttr_p50 = percentile(&mut mttr_known, 50, 100);
    out.push_str(&format!(
        "- Median time to resolve (P50 MTTR): **{}**\n",
        format_duration_seconds(mttr_p50)
    ));
    out.push('\n');

    out.push_str("## Metrics summary (distributions)\n\n");
    out.push_str("| Metric | Known | P50 | P90 |\n");
    out.push_str("|---|---:|---:|---:|\n");
    out.push_str(&metric_summary_row(
        "MTTD (start → first observed)",
        &mttd,
        total,
    ));
    out.push_str(&metric_summary_row(
        "IT awareness lag (observed → IT aware)",
        &awareness,
        total,
    ));
    out.push_str(&metric_summary_row("MTTA (IT aware → ack)", &mtta, total));
    out.push_str(&metric_summary_row(
        "Time to mitigation (ack → mitigate)",
        &ttm,
        total,
    ));
    out.push_str(&metric_summary_row(
        "MTTR (start/observed → resolve)",
        &mttr,
        total,
    ));
    out.push('\n');

    out.push_str("## Severity distribution\n\n");
    for s in &dash.severity_counts {
        out.push_str(&format!("- {}: {}\n", s.severity, s.count));
    }
    out.push('\n');

    out.push_str("## Detection story\n\n");
    out.push_str("### Detection source mix\n\n");
    for b in &dash.detection_story.detection_source_mix {
        out.push_str(&format!("- {}: {}\n", b.label, b.count));
    }
    out.push('\n');
    out.push_str("### IT awareness lag distribution\n\n");
    for b in &dash.detection_story.it_awareness_lag_buckets {
        out.push_str(&format!("- {}: {}\n", b.label, b.count));
    }
    out.push('\n');

    out.push_str("## Vendor/service reliability\n\n");
    out.push_str("### Top vendors by incident count\n\n");
    for b in &dash.vendor_service_story.top_vendors_by_count {
        out.push_str(&format!("- {}: {}\n", b.label, b.count));
    }
    out.push('\n');
    out.push_str("### Top services by incident count\n\n");
    for b in &dash.vendor_service_story.top_services_by_count {
        out.push_str(&format!("- {}: {}\n", b.label, b.count));
    }
    out.push('\n');
    out.push_str("### Top vendors by weighted pain (impact × degradation × duration)\n\n");
    for b in &dash.vendor_service_story.top_vendors_by_pain {
        out.push_str(&format!(
            "- {}: pain_sum={}, incidents={}, pain_known={}\n",
            b.label, b.pain_sum, b.count, b.pain_known_count
        ));
    }
    out.push('\n');
    out.push_str("### Top services by weighted pain (impact × degradation × duration)\n\n");
    for b in &dash.vendor_service_story.top_services_by_pain {
        out.push_str(&format!(
            "- {}: pain_sum={}, incidents={}, pain_known={}\n",
            b.label, b.pain_sum, b.count, b.pain_known_count
        ));
    }
    out.push('\n');

    out.push_str("## Response story\n\n");
    out.push_str("### Time to mitigation distribution\n\n");
    for b in &dash.response_story.time_to_mitigation_buckets {
        out.push_str(&format!("- {}: {}\n", b.label, b.count));
    }
    out.push('\n');
    out.push_str("### Time to resolve distribution\n\n");
    for b in &dash.response_story.time_to_resolve_buckets {
        out.push_str(&format!("- {}: {}\n", b.label, b.count));
    }
    out.push('\n');

    out.push_str("## Incidents (stable ordering)\n\n");
    out.push_str(
        "_Sort keys:_ `external_id` (missing treated as empty), then `title`, then `id`.\n\n",
    );
    out.push_str(
        "| External ID | Title | Severity | Detection | Vendor | Service | MTTR | Warnings |\n",
    );
    out.push_str("|---|---|---|---|---|---|---:|---:|\n");
    for inc in &dash.incidents {
        let external = inc
            .external_id
            .clone()
            .unwrap_or_else(|| "NO_EXTERNAL_ID".to_string());
        let sev = inc
            .severity
            .clone()
            .unwrap_or_else(|| "UNKNOWN".to_string());
        let det = inc
            .detection_source
            .clone()
            .unwrap_or_else(|| "UNKNOWN".to_string());
        let vendor = inc.vendor.clone().unwrap_or_else(|| "UNKNOWN".to_string());
        let service = inc.service.clone().unwrap_or_else(|| "UNKNOWN".to_string());
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            external,
            inc.title,
            sev,
            det,
            vendor,
            service,
            format_duration_seconds(inc.mttr_seconds),
            inc.warning_count
        ));
    }
    out.push('\n');

    out.push_str("## Validation and anomalies appendix\n\n");
    let mut any = false;
    for (i, inc) in incident_rows.iter().enumerate() {
        let codes = warnings_by_incident
            .get(i)
            .map(|(_, c)| c.clone())
            .unwrap_or_default();
        if codes.is_empty() {
            continue;
        }
        any = true;
        let id_label = inc.external_id.as_deref().unwrap_or("NO_EXTERNAL_ID");
        out.push_str(&format!("### {id_label}: {}\n\n", inc.title));
        out.push_str(&format!("- Warning codes: {}\n\n", codes.join(", ")));
    }
    if !any {
        out.push_str("- None.\n");
    }

    Ok(out)
}
