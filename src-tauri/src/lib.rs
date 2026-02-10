use std::fs;
use std::path::PathBuf;

use qir_ai::ollama::OllamaClient;
use qir_core::analytics::{DashboardPayloadV1, DashboardPayloadV2};
use qir_core::backup::{BackupCreateResult, BackupManifest, RestoreResult};
use qir_core::demo::seed_demo_dataset as core_seed_demo_dataset;
use qir_core::error::AppError;
use qir_core::ingest::jira_csv::{
    import_jira_csv, preview_jira_csv, JiraCsvMapping, JiraCsvPreview, JiraImportSummary,
};
use qir_core::ingest::slack_transcript::{
    ingest_slack_transcript_text, preview_slack_transcript_text, SlackIngestSummary, SlackPreview,
};
use qir_core::profiles::jira::{
    delete_profile, list_profiles, upsert_profile, JiraMappingProfile, JiraMappingProfileUpsert,
};
use qir_core::report::generate_qir_markdown;
use qir_core::sanitize::{export_sanitized_dataset as core_export_sanitized_dataset, SanitizedExportResult};
use qir_core::validate::{validate_all_incidents, IncidentValidationReportItem};
use tauri::Manager;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Debug, serde::Serialize)]
pub struct InitDbResponse {
    pub db_path: String,
}

#[derive(Debug, serde::Serialize)]
pub struct AiHealthStatus {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, serde::Serialize)]
pub struct DeleteResponse {
    pub ok: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct IncidentListItem {
    pub id: i64,
    pub external_id: Option<String>,
    pub title: String,
}

fn default_db_path(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let dir = app.path().app_data_dir().map_err(|e| {
        AppError::new("DB_PATH_FAILED", "Failed to resolve app data directory")
            .with_details(e.to_string())
    })?;

    fs::create_dir_all(&dir).map_err(|e| {
        AppError::new("DB_PATH_FAILED", "Failed to create app data directory")
            .with_details(e.to_string())
    })?;

    Ok(dir.join("incidentreview.sqlite"))
}

fn open_and_migrate(db_path: &PathBuf) -> Result<rusqlite::Connection, AppError> {
    let mut conn = qir_core::db::open(db_path)?;
    qir_core::db::migrate(&mut conn)?;
    Ok(conn)
}

fn default_artifacts_dir(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let dir = app.path().app_data_dir().map_err(|e| {
        AppError::new("DB_PATH_FAILED", "Failed to resolve app data directory")
            .with_details(e.to_string())
    })?;
    Ok(dir.join("artifacts"))
}

fn now_rfc3339_utc() -> Result<String, AppError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|e| AppError::new("DB_BACKUP_TIME_FAILED", "Failed to format time").with_details(e.to_string()))
}

#[tauri::command]
fn init_db(app: tauri::AppHandle) -> Result<InitDbResponse, AppError> {
    let db_path = default_db_path(&app)?;
    let _conn = open_and_migrate(&db_path)?;
    Ok(InitDbResponse {
        db_path: db_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn seed_demo_jira(app: tauri::AppHandle) -> Result<JiraImportSummary, AppError> {
    let db_path = default_db_path(&app)?;
    let mut conn = open_and_migrate(&db_path)?;

    let csv_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/demo/jira_sample.csv"
    ));

    // Demo mapping for the sanitized fixture.
    let mapping = JiraCsvMapping {
        external_id: Some("Key".to_string()),
        title: "Summary".to_string(),
        description: Some("Description".to_string()),
        severity: Some("Severity".to_string()),
        detection_source: None,
        vendor: None,
        service: None,
        impact_pct: Some("ImpactPct".to_string()),
        service_health_pct: Some("ServiceHealthPct".to_string()),
        start_ts: Some("StartTs".to_string()),
        first_observed_ts: None,
        it_awareness_ts: None,
        ack_ts: Some("AckTs".to_string()),
        mitigate_ts: None,
        resolve_ts: Some("ResolveTs".to_string()),
    };

    import_jira_csv(&mut conn, csv_text, &mapping)
}

#[tauri::command]
fn seed_demo_dataset(app: tauri::AppHandle) -> Result<JiraImportSummary, AppError> {
    let db_path = default_db_path(&app)?;
    let mut conn = open_and_migrate(&db_path)?;
    core_seed_demo_dataset(&mut conn)
}

#[tauri::command]
fn get_dashboard_v1(app: tauri::AppHandle) -> Result<DashboardPayloadV1, AppError> {
    let db_path = default_db_path(&app)?;
    let conn = open_and_migrate(&db_path)?;
    qir_core::analytics::build_dashboard_payload_v1(&conn)
}

#[tauri::command]
fn get_dashboard_v2(app: tauri::AppHandle) -> Result<DashboardPayloadV2, AppError> {
    let db_path = default_db_path(&app)?;
    let conn = open_and_migrate(&db_path)?;
    qir_core::analytics::build_dashboard_payload_v2(&conn)
}

#[tauri::command]
fn generate_report_md(app: tauri::AppHandle) -> Result<String, AppError> {
    let db_path = default_db_path(&app)?;
    let conn = open_and_migrate(&db_path)?;
    generate_qir_markdown(&conn)
}

#[tauri::command]
fn jira_csv_preview(csv_text: String, max_rows: usize) -> Result<JiraCsvPreview, AppError> {
    preview_jira_csv(&csv_text, max_rows)
}

#[tauri::command]
fn jira_profiles_list(app: tauri::AppHandle) -> Result<Vec<JiraMappingProfile>, AppError> {
    let db_path = default_db_path(&app)?;
    let conn = open_and_migrate(&db_path)?;
    list_profiles(&conn)
}

#[tauri::command]
fn jira_profiles_upsert(
    app: tauri::AppHandle,
    profile: JiraMappingProfileUpsert,
) -> Result<JiraMappingProfile, AppError> {
    let db_path = default_db_path(&app)?;
    let mut conn = open_and_migrate(&db_path)?;
    upsert_profile(&mut conn, profile)
}

#[tauri::command]
fn jira_profiles_delete(app: tauri::AppHandle, id: i64) -> Result<DeleteResponse, AppError> {
    let db_path = default_db_path(&app)?;
    let mut conn = open_and_migrate(&db_path)?;
    delete_profile(&mut conn, id)?;
    Ok(DeleteResponse { ok: true })
}

#[tauri::command]
fn jira_import_using_profile(
    app: tauri::AppHandle,
    profile_id: i64,
    csv_text: String,
) -> Result<JiraImportSummary, AppError> {
    let db_path = default_db_path(&app)?;
    let mut conn = open_and_migrate(&db_path)?;
    let profile = qir_core::profiles::jira::get_profile(&conn, profile_id)?;
    import_jira_csv(&mut conn, &csv_text, &profile.mapping)
}

#[tauri::command]
fn incidents_list(app: tauri::AppHandle) -> Result<Vec<IncidentListItem>, AppError> {
    let db_path = default_db_path(&app)?;
    let conn = open_and_migrate(&db_path)?;
    let incidents = qir_core::repo::list_incidents(&conn)?;

    Ok(incidents
        .into_iter()
        .map(|i| IncidentListItem {
            id: i.id,
            external_id: i.external_id,
            title: i.title,
        })
        .collect())
}

#[tauri::command]
fn incident_detail(app: tauri::AppHandle, incident_id: i64) -> Result<qir_core::repo::IncidentDetail, AppError> {
    let db_path = default_db_path(&app)?;
    let conn = open_and_migrate(&db_path)?;
    qir_core::repo::get_incident_detail(&conn, incident_id)
}

#[tauri::command]
fn validation_report(app: tauri::AppHandle) -> Result<Vec<IncidentValidationReportItem>, AppError> {
    let db_path = default_db_path(&app)?;
    let conn = open_and_migrate(&db_path)?;
    validate_all_incidents(&conn)
}

#[tauri::command]
fn slack_preview(transcript_text: String) -> Result<SlackPreview, AppError> {
    Ok(preview_slack_transcript_text(&transcript_text))
}

#[tauri::command]
fn slack_ingest(
    app: tauri::AppHandle,
    incident_id: Option<i64>,
    new_incident_title: Option<String>,
    transcript_text: String,
) -> Result<SlackIngestSummary, AppError> {
    let db_path = default_db_path(&app)?;
    let mut conn = open_and_migrate(&db_path)?;
    ingest_slack_transcript_text(
        &mut conn,
        incident_id,
        new_incident_title.as_deref(),
        &transcript_text,
    )
}

#[tauri::command]
fn ai_health_check() -> Result<AiHealthStatus, AppError> {
    let client = OllamaClient::new("http://127.0.0.1:11434")?;
    client.health_check()?;
    Ok(AiHealthStatus {
        ok: true,
        message: "Ollama reachable on 127.0.0.1".to_string(),
    })
}

#[tauri::command]
fn backup_create(app: tauri::AppHandle, destination_dir: String) -> Result<BackupCreateResult, AppError> {
    let db_path = default_db_path(&app)?;
    let conn = open_and_migrate(&db_path)?;
    let artifacts_dir = default_artifacts_dir(&app)?;
    let export_time = now_rfc3339_utc()?;
    let dest_root = PathBuf::from(destination_dir);

    let artifacts_opt = if artifacts_dir.is_dir() {
        Some(artifacts_dir.as_path())
    } else {
        None
    };

    qir_core::backup::create_backup(
        &conn,
        &db_path,
        artifacts_opt,
        dest_root.as_path(),
        &export_time,
        env!("CARGO_PKG_VERSION"),
    )
}

#[tauri::command]
fn backup_inspect(backup_dir: String) -> Result<BackupManifest, AppError> {
    qir_core::backup::read_manifest(PathBuf::from(backup_dir).as_path())
}

#[tauri::command]
fn restore_from_backup(
    app: tauri::AppHandle,
    backup_dir: String,
    allow_overwrite: bool,
) -> Result<RestoreResult, AppError> {
    let db_path = default_db_path(&app)?;
    let artifacts_dir = default_artifacts_dir(&app)?;
    let artifacts_opt = Some(artifacts_dir.as_path());

    qir_core::backup::restore_from_backup(
        PathBuf::from(backup_dir).as_path(),
        &db_path,
        artifacts_opt,
        allow_overwrite,
    )
}

#[tauri::command]
fn export_sanitized_dataset(
    app: tauri::AppHandle,
    destination_dir: String,
) -> Result<SanitizedExportResult, AppError> {
    let db_path = default_db_path(&app)?;
    let conn = open_and_migrate(&db_path)?;
    let export_time = now_rfc3339_utc()?;
    let dest_root = PathBuf::from(destination_dir);
    core_export_sanitized_dataset(&conn, dest_root.as_path(), &export_time, env!("CARGO_PKG_VERSION"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            init_db,
            seed_demo_jira,
            seed_demo_dataset,
            get_dashboard_v1,
            get_dashboard_v2,
            generate_report_md,
            jira_csv_preview,
            jira_profiles_list,
            jira_profiles_upsert,
            jira_profiles_delete,
            jira_import_using_profile,
            incidents_list,
            incident_detail,
            validation_report,
            slack_preview,
            slack_ingest,
            ai_health_check
            ,
            backup_create,
            backup_inspect,
            restore_from_backup,
            export_sanitized_dataset
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
