use crate::error::AppError;
use crate::ingest::jira_csv::{import_jira_csv, JiraCsvMapping, JiraImportSummary};
use rusqlite::Connection;

fn demo_csv() -> String {
    // Sanitized, deterministic dataset large enough to make dashboards/reports meaningful.
    // Keep timestamps RFC3339 so deterministic metrics can be computed.
    let mut out = String::new();
    out.push_str("Key,Summary,Severity,DetectionSource,Vendor,Service,ImpactPct,ServiceHealthPct,StartTs,FirstObservedTs,ItAwarenessTs,AckTs,MitigateTs,ResolveTs\n");

    let severities = ["SEV0", "SEV1", "SEV2", "SEV3"];
    let detections = ["monitoring", "customer", "vendor", "internal_test"];
    let vendors = ["AcmeCloud", "ContosoNet", "ExampleVendor", "WidgetCo"];
    let services = ["payments", "auth", "api", "search", "billing"];

    // Base time: 2026-01-01T00:00:00Z
    for i in 1..=40 {
        let sev = severities[(i - 1) % severities.len()];
        let det = detections[(i - 1) % detections.len()];
        let vendor = vendors[(i - 1) % vendors.len()];
        let service = services[(i - 1) % services.len()];

        let impact_pct = match sev {
            "SEV0" => 80,
            "SEV1" => 50,
            "SEV2" => 25,
            _ => 10,
        };
        let service_health = 100 - (impact_pct / 2);

        // Spread incidents across a deterministic window.
        // Times are simple and allowlisted (RFC3339).
        let day = 1 + (i - 1) / 2; // two incidents per day
        let hour = ((i - 1) % 2) * 6; // 0 or 6

        let start = format!("2026-01-{:02}T{:02}:00:00Z", day, hour);
        let first_observed = format!("2026-01-{:02}T{:02}:05:00Z", day, hour);
        let it_awareness = format!("2026-01-{:02}T{:02}:10:00Z", day, hour);
        let ack = format!("2026-01-{:02}T{:02}:15:00Z", day, hour);
        let mitigate = format!("2026-01-{:02}T{:02}:45:00Z", day, hour);
        let resolve = format!("2026-01-{:02}T{:02}:55:00Z", day, hour);

        out.push_str(&format!(
            "IR-{i:03},\"Demo incident {i}\",{sev},{det},{vendor},{service},{impact_pct},{service_health},{start},{first_observed},{it_awareness},{ack},{mitigate},{resolve}\n"
        ));
    }
    out
}

pub fn seed_demo_dataset(conn: &mut Connection) -> Result<JiraImportSummary, AppError> {
    let mapping = JiraCsvMapping {
        external_id: Some("Key".to_string()),
        title: "Summary".to_string(),
        description: None,
        severity: Some("Severity".to_string()),
        detection_source: Some("DetectionSource".to_string()),
        vendor: Some("Vendor".to_string()),
        service: Some("Service".to_string()),
        impact_pct: Some("ImpactPct".to_string()),
        service_health_pct: Some("ServiceHealthPct".to_string()),
        start_ts: Some("StartTs".to_string()),
        first_observed_ts: Some("FirstObservedTs".to_string()),
        it_awareness_ts: Some("ItAwarenessTs".to_string()),
        ack_ts: Some("AckTs".to_string()),
        mitigate_ts: Some("MitigateTs".to_string()),
        resolve_ts: Some("ResolveTs".to_string()),
    };

    import_jira_csv(conn, &demo_csv(), &mapping)
}

