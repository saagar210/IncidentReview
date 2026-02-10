use std::collections::BTreeMap;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
// Note: analytics uses only deterministic data from qir_core; no UI-side computation.

use crate::error::AppError;
use crate::metrics::compute_incident_metrics;
use crate::repo::list_incidents;
use crate::validate::validate_incident;

pub const DASHBOARD_PAYLOAD_VERSION: u32 = 1;
pub const DASHBOARD_PAYLOAD_V2_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeverityCount {
    pub severity: String,
    pub count: i64,
    pub incident_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncidentSummary {
    pub id: i64,
    pub external_id: Option<String>,
    pub title: String,
    pub severity: Option<String>,
    pub mtta_seconds: Option<i64>,
    pub mttr_seconds: Option<i64>,
    pub warning_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardPayloadV1 {
    pub version: u32,
    pub incident_count: i64,
    pub severity_counts: Vec<SeverityCount>,
    pub incidents: Vec<IncidentSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategoryBucket {
    pub key: String,
    pub label: String,
    pub count: i64,
    pub incident_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DurationBucket {
    pub key: String,
    pub label: String,
    pub count: i64,
    pub incident_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PainBucket {
    pub key: String,
    pub label: String,
    pub count: i64,
    pub pain_sum: i64,
    pub pain_known_count: i64,
    pub incident_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectionStoryV1 {
    pub detection_source_mix: Vec<CategoryBucket>,
    pub it_awareness_lag_buckets: Vec<DurationBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VendorServiceStoryV1 {
    pub top_vendors_by_count: Vec<CategoryBucket>,
    pub top_services_by_count: Vec<CategoryBucket>,
    pub top_vendors_by_pain: Vec<PainBucket>,
    pub top_services_by_pain: Vec<PainBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResponseStoryV1 {
    pub time_to_mitigation_buckets: Vec<DurationBucket>,
    pub time_to_resolve_buckets: Vec<DurationBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncidentSummaryV2 {
    pub id: i64,
    pub external_id: Option<String>,
    pub title: String,
    pub severity: Option<String>,
    pub detection_source: Option<String>,
    pub vendor: Option<String>,
    pub service: Option<String>,
    pub it_awareness_lag_seconds: Option<i64>,
    pub time_to_mitigation_seconds: Option<i64>,
    pub mttr_seconds: Option<i64>,
    pub warning_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardPayloadV2 {
    pub version: u32,
    pub incident_count: i64,
    pub severity_counts: Vec<SeverityCount>,
    pub incidents: Vec<IncidentSummaryV2>,
    pub detection_story: DetectionStoryV1,
    pub vendor_service_story: VendorServiceStoryV1,
    pub response_story: ResponseStoryV1,
}

pub fn build_dashboard_payload_v1(conn: &Connection) -> Result<DashboardPayloadV1, AppError> {
    let incidents = list_incidents(conn)?;

    let mut severity_map: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    let mut incident_summaries = Vec::new();

    for inc in &incidents {
        let severity_key = inc
            .severity
            .clone()
            .unwrap_or_else(|| "UNKNOWN".to_string());
        severity_map.entry(severity_key).or_default().push(inc.id);

        let (_, metric_warnings) = compute_incident_metrics(inc);
        let val_warnings = validate_incident(inc);
        let warning_count = (metric_warnings.len() + val_warnings.len()) as i64;

        let (metrics, _) = compute_incident_metrics(inc);

        incident_summaries.push(IncidentSummary {
            id: inc.id,
            external_id: inc.external_id.clone(),
            title: inc.title.clone(),
            severity: inc.severity.clone(),
            mtta_seconds: metrics.mtta_seconds,
            mttr_seconds: metrics.mttr_seconds,
            warning_count,
        });
    }

    // Deterministic ordering: external_id, then title, then id.
    incident_summaries.sort_by(|a, b| {
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

    let severity_counts = severity_map
        .into_iter()
        .map(|(severity, mut ids)| {
            ids.sort();
            SeverityCount {
                severity,
                count: ids.len() as i64,
                incident_ids: ids,
            }
        })
        .collect::<Vec<_>>();

    Ok(DashboardPayloadV1 {
        version: DASHBOARD_PAYLOAD_VERSION,
        incident_count: incidents.len() as i64,
        severity_counts,
        incidents: incident_summaries,
    })
}

fn bucket_label_for_duration(secs: Option<i64>) -> (&'static str, &'static str) {
    // Deterministic, human-friendly buckets. Always include UNKNOWN to reconcile to incident totals.
    match secs {
        None => ("unknown", "UNKNOWN (unparseable/missing)"),
        Some(s) if s <= 5 * 60 => ("le_5m", "0-5m"),
        Some(s) if s <= 15 * 60 => ("le_15m", "5-15m"),
        Some(s) if s <= 60 * 60 => ("le_1h", "15m-1h"),
        Some(s) if s <= 4 * 60 * 60 => ("le_4h", "1h-4h"),
        Some(s) if s <= 24 * 60 * 60 => ("le_24h", "4h-24h"),
        Some(_) => ("gt_24h", ">24h"),
    }
}

fn duration_bucket_order_key(key: &str) -> i32 {
    match key {
        "le_5m" => 1,
        "le_15m" => 2,
        "le_1h" => 3,
        "le_4h" => 4,
        "le_24h" => 5,
        "gt_24h" => 6,
        "unknown" => 7,
        _ => 999,
    }
}

fn category_key(raw: &Option<String>) -> String {
    raw.clone().unwrap_or_else(|| "UNKNOWN".to_string())
}

fn compute_pain_units(
    impact_pct: Option<i64>,
    service_health_pct: Option<i64>,
    duration_secs: Option<i64>,
) -> Option<i64> {
    let impact = impact_pct?;
    let health = service_health_pct?;
    let dur = duration_secs?;
    if !(0..=100).contains(&impact) || !(0..=100).contains(&health) || dur < 0 {
        return None;
    }
    let degradation = 100 - health;

    let p: i128 = (impact as i128) * (degradation as i128) * (dur as i128);
    if p <= 0 {
        return Some(0);
    }
    Some(std::cmp::min(p, i64::MAX as i128) as i64)
}

fn stable_top_n_keys_by_count(map: &BTreeMap<String, Vec<i64>>, n: usize) -> Vec<String> {
    let mut items = map
        .iter()
        .map(|(k, ids)| (k.clone(), ids.len() as i64))
        .collect::<Vec<_>>();
    items.sort_by(|a, b| (-(a.1), a.0.clone()).cmp(&(-(b.1), b.0.clone())));
    items.into_iter().take(n).map(|(k, _)| k).collect()
}

fn stable_top_n_keys_by_pain(map: &BTreeMap<String, PainBucket>, n: usize) -> Vec<String> {
    let mut items = map
        .iter()
        .map(|(k, b)| (k.clone(), b.pain_sum, b.count))
        .collect::<Vec<_>>();
    items.sort_by(|a, b| (-(a.1), -(a.2), a.0.clone()).cmp(&(-(b.1), -(b.2), b.0.clone())));
    items.into_iter().take(n).map(|(k, _, _)| k).collect()
}

pub fn build_dashboard_payload_v2(conn: &Connection) -> Result<DashboardPayloadV2, AppError> {
    let incidents = list_incidents(conn)?;
    let incident_count = incidents.len() as i64;

    let mut severity_map: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    let mut detection_source_map: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    let mut awareness_lag_map: BTreeMap<String, (String, Vec<i64>)> = BTreeMap::new();

    let mut vendor_map: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    let mut service_map: BTreeMap<String, Vec<i64>> = BTreeMap::new();

    let mut vendor_pain_map: BTreeMap<String, PainBucket> = BTreeMap::new();
    let mut service_pain_map: BTreeMap<String, PainBucket> = BTreeMap::new();

    let mut ttm_map: BTreeMap<String, (String, Vec<i64>)> = BTreeMap::new();
    let mut ttr_map: BTreeMap<String, (String, Vec<i64>)> = BTreeMap::new();

    let mut incident_summaries = Vec::new();

    for inc in &incidents {
        let severity_key = inc
            .severity
            .clone()
            .unwrap_or_else(|| "UNKNOWN".to_string());
        severity_map.entry(severity_key).or_default().push(inc.id);

        let det_key = category_key(&inc.detection_source);
        detection_source_map
            .entry(det_key)
            .or_default()
            .push(inc.id);

        let vendor_key = category_key(&inc.vendor);
        vendor_map
            .entry(vendor_key.clone())
            .or_default()
            .push(inc.id);
        let service_key = category_key(&inc.service);
        service_map
            .entry(service_key.clone())
            .or_default()
            .push(inc.id);

        let (metrics, metric_warnings) = compute_incident_metrics(inc);
        let val_warnings = validate_incident(inc);
        let warning_count = (metric_warnings.len() + val_warnings.len()) as i64;

        let (lag_key, lag_label) = bucket_label_for_duration(metrics.it_awareness_lag_seconds);
        awareness_lag_map
            .entry(lag_key.to_string())
            .or_insert_with(|| (lag_label.to_string(), Vec::new()))
            .1
            .push(inc.id);

        let (ttm_key, ttm_label) = bucket_label_for_duration(metrics.time_to_mitigation_seconds);
        ttm_map
            .entry(ttm_key.to_string())
            .or_insert_with(|| (ttm_label.to_string(), Vec::new()))
            .1
            .push(inc.id);

        let (ttr_key, ttr_label) = bucket_label_for_duration(metrics.mttr_seconds);
        ttr_map
            .entry(ttr_key.to_string())
            .or_insert_with(|| (ttr_label.to_string(), Vec::new()))
            .1
            .push(inc.id);

        let pain = compute_pain_units(inc.impact_pct, inc.service_health_pct, metrics.mttr_seconds);
        {
            let entry = vendor_pain_map
                .entry(vendor_key.clone())
                .or_insert(PainBucket {
                    key: vendor_key.clone(),
                    label: vendor_key.clone(),
                    count: 0,
                    pain_sum: 0,
                    pain_known_count: 0,
                    incident_ids: Vec::new(),
                });
            entry.count += 1;
            entry.incident_ids.push(inc.id);
            if let Some(p) = pain {
                entry.pain_sum = entry.pain_sum.saturating_add(p);
                entry.pain_known_count += 1;
            }
        }
        {
            let entry = service_pain_map
                .entry(service_key.clone())
                .or_insert(PainBucket {
                    key: service_key.clone(),
                    label: service_key.clone(),
                    count: 0,
                    pain_sum: 0,
                    pain_known_count: 0,
                    incident_ids: Vec::new(),
                });
            entry.count += 1;
            entry.incident_ids.push(inc.id);
            if let Some(p) = pain {
                entry.pain_sum = entry.pain_sum.saturating_add(p);
                entry.pain_known_count += 1;
            }
        }

        incident_summaries.push(IncidentSummaryV2 {
            id: inc.id,
            external_id: inc.external_id.clone(),
            title: inc.title.clone(),
            severity: inc.severity.clone(),
            detection_source: inc.detection_source.clone(),
            vendor: inc.vendor.clone(),
            service: inc.service.clone(),
            it_awareness_lag_seconds: metrics.it_awareness_lag_seconds,
            time_to_mitigation_seconds: metrics.time_to_mitigation_seconds,
            mttr_seconds: metrics.mttr_seconds,
            warning_count,
        });
    }

    // Deterministic ordering: external_id, then title, then id.
    incident_summaries.sort_by(|a, b| {
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

    let severity_counts = severity_map
        .into_iter()
        .map(|(severity, ids)| SeverityCount {
            severity,
            count: ids.len() as i64,
            incident_ids: ids,
        })
        .collect::<Vec<_>>();

    let mut detection_source_mix = detection_source_map
        .into_iter()
        .map(|(k, mut ids)| {
            ids.sort();
            CategoryBucket {
                key: format!("detection_source:{k}"),
                label: k,
                count: ids.len() as i64,
                incident_ids: ids,
            }
        })
        .collect::<Vec<_>>();
    detection_source_mix
        .sort_by(|a, b| (-(a.count), a.label.clone()).cmp(&(-(b.count), b.label.clone())));

    let mut it_awareness_lag_buckets = awareness_lag_map
        .into_iter()
        .map(|(k, (label, mut ids))| {
            ids.sort();
            DurationBucket {
                key: format!("it_awareness_lag:{k}"),
                label,
                count: ids.len() as i64,
                incident_ids: ids,
            }
        })
        .collect::<Vec<_>>();
    it_awareness_lag_buckets.sort_by(|a, b| {
        duration_bucket_order_key(a.key.split(':').nth(1).unwrap_or("")).cmp(
            &duration_bucket_order_key(b.key.split(':').nth(1).unwrap_or("")),
        )
    });

    let mut time_to_mitigation_buckets = ttm_map
        .into_iter()
        .map(|(k, (label, mut ids))| {
            ids.sort();
            DurationBucket {
                key: format!("time_to_mitigation:{k}"),
                label,
                count: ids.len() as i64,
                incident_ids: ids,
            }
        })
        .collect::<Vec<_>>();
    time_to_mitigation_buckets.sort_by(|a, b| {
        duration_bucket_order_key(a.key.split(':').nth(1).unwrap_or("")).cmp(
            &duration_bucket_order_key(b.key.split(':').nth(1).unwrap_or("")),
        )
    });

    let mut time_to_resolve_buckets = ttr_map
        .into_iter()
        .map(|(k, (label, mut ids))| {
            ids.sort();
            DurationBucket {
                key: format!("time_to_resolve:{k}"),
                label,
                count: ids.len() as i64,
                incident_ids: ids,
            }
        })
        .collect::<Vec<_>>();
    time_to_resolve_buckets.sort_by(|a, b| {
        duration_bucket_order_key(a.key.split(':').nth(1).unwrap_or("")).cmp(
            &duration_bucket_order_key(b.key.split(':').nth(1).unwrap_or("")),
        )
    });

    // Build top-N vendor/service by count with OTHER bucket for reconciliation.
    let top_n = 8usize;
    let top_vendor_keys = stable_top_n_keys_by_count(&vendor_map, top_n);
    let top_service_keys = stable_top_n_keys_by_count(&service_map, top_n);

    let top_vendors_by_count = {
        let mut out = Vec::new();
        let mut other_ids = Vec::new();
        for (k, ids) in &vendor_map {
            if top_vendor_keys.contains(k) {
                let mut ids2 = ids.clone();
                ids2.sort();
                out.push(CategoryBucket {
                    key: format!("vendor:{k}"),
                    label: k.clone(),
                    count: ids.len() as i64,
                    incident_ids: ids2,
                });
            } else {
                other_ids.extend(ids.iter().copied());
            }
        }
        out.sort_by(|a, b| (-(a.count), a.label.clone()).cmp(&(-(b.count), b.label.clone())));
        if !other_ids.is_empty() {
            other_ids.sort();
            out.push(CategoryBucket {
                key: "vendor:OTHER".to_string(),
                label: "OTHER".to_string(),
                count: other_ids.len() as i64,
                incident_ids: other_ids,
            });
        }
        out
    };

    let top_services_by_count = {
        let mut out = Vec::new();
        let mut other_ids = Vec::new();
        for (k, ids) in &service_map {
            if top_service_keys.contains(k) {
                let mut ids2 = ids.clone();
                ids2.sort();
                out.push(CategoryBucket {
                    key: format!("service:{k}"),
                    label: k.clone(),
                    count: ids.len() as i64,
                    incident_ids: ids2,
                });
            } else {
                other_ids.extend(ids.iter().copied());
            }
        }
        out.sort_by(|a, b| (-(a.count), a.label.clone()).cmp(&(-(b.count), b.label.clone())));
        if !other_ids.is_empty() {
            other_ids.sort();
            out.push(CategoryBucket {
                key: "service:OTHER".to_string(),
                label: "OTHER".to_string(),
                count: other_ids.len() as i64,
                incident_ids: other_ids,
            });
        }
        out
    };

    let top_vendor_pain_keys = stable_top_n_keys_by_pain(&vendor_pain_map, top_n);
    let top_service_pain_keys = stable_top_n_keys_by_pain(&service_pain_map, top_n);

    let top_vendors_by_pain = {
        let mut out = Vec::new();
        let mut other = PainBucket {
            key: "vendor:OTHER".to_string(),
            label: "OTHER".to_string(),
            count: 0,
            pain_sum: 0,
            pain_known_count: 0,
            incident_ids: Vec::new(),
        };
        for (k, b) in &vendor_pain_map {
            if top_vendor_pain_keys.contains(k) {
                let mut b2 = b.clone();
                b2.key = format!("vendor:{k}");
                b2.label = k.clone();
                b2.incident_ids.sort();
                out.push(b2);
            } else {
                other.count += b.count;
                other.pain_sum = other.pain_sum.saturating_add(b.pain_sum);
                other.pain_known_count += b.pain_known_count;
                other.incident_ids.extend(b.incident_ids.iter().copied());
            }
        }
        out.sort_by(|a, b| {
            (-(a.pain_sum), -(a.count), a.label.clone()).cmp(&(
                -(b.pain_sum),
                -(b.count),
                b.label.clone(),
            ))
        });
        if other.count > 0 {
            other.incident_ids.sort();
            out.push(other);
        }
        out
    };

    let top_services_by_pain = {
        let mut out = Vec::new();
        let mut other = PainBucket {
            key: "service:OTHER".to_string(),
            label: "OTHER".to_string(),
            count: 0,
            pain_sum: 0,
            pain_known_count: 0,
            incident_ids: Vec::new(),
        };
        for (k, b) in &service_pain_map {
            if top_service_pain_keys.contains(k) {
                let mut b2 = b.clone();
                b2.key = format!("service:{k}");
                b2.label = k.clone();
                b2.incident_ids.sort();
                out.push(b2);
            } else {
                other.count += b.count;
                other.pain_sum = other.pain_sum.saturating_add(b.pain_sum);
                other.pain_known_count += b.pain_known_count;
                other.incident_ids.extend(b.incident_ids.iter().copied());
            }
        }
        out.sort_by(|a, b| {
            (-(a.pain_sum), -(a.count), a.label.clone()).cmp(&(
                -(b.pain_sum),
                -(b.count),
                b.label.clone(),
            ))
        });
        if other.count > 0 {
            other.incident_ids.sort();
            out.push(other);
        }
        out
    };

    Ok(DashboardPayloadV2 {
        version: DASHBOARD_PAYLOAD_V2_VERSION,
        incident_count,
        severity_counts,
        incidents: incident_summaries,
        detection_story: DetectionStoryV1 {
            detection_source_mix,
            it_awareness_lag_buckets,
        },
        vendor_service_story: VendorServiceStoryV1 {
            top_vendors_by_count,
            top_services_by_count,
            top_vendors_by_pain,
            top_services_by_pain,
        },
        response_story: ResponseStoryV1 {
            time_to_mitigation_buckets,
            time_to_resolve_buckets,
        },
    })
}
