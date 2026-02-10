use std::fs;
use std::sync::Mutex;
use std::path::PathBuf;

use qir_ai::ollama::{OllamaClient, OllamaModelInfo};
use qir_ai::evidence::{
    BuildChunksResult as AiBuildChunksResult, EvidenceAddSourceInput as AiEvidenceAddSourceInput,
    EvidenceChunkSummary as AiEvidenceChunkSummary, EvidenceOrigin as AiEvidenceOrigin,
    EvidenceQueryStore as AiEvidenceQueryStore, EvidenceSource as AiEvidenceSource,
    EvidenceSourceType as AiEvidenceSourceType, EvidenceStore as AiEvidenceStore,
    AiIndexBuildInput as AiIndexBuildInput, AiIndexStatus as AiIndexStatus, IndexStore as AiIndexStore,
};
use qir_ai::embeddings::ollama_embed::OllamaEmbedder;
use qir_ai::llm::ollama_llm::OllamaLlm;
use qir_ai::retrieve::{query_with_embedder as ai_query_with_embedder, EvidenceQueryResponse as AiEvidenceQueryResponse};
use qir_ai::draft::{draft_section_with_llm as ai_draft_with_llm, AiDraftResponse as AiDraftResponse, AiDraftSectionRequest as AiDraftSectionRequest, SectionId as AiSectionId};
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
use qir_core::sanitize::{
    export_sanitized_dataset as core_export_sanitized_dataset, import_sanitized_dataset as core_import_sanitized_dataset,
    inspect_sanitized_dataset as core_inspect_sanitized_dataset, SanitizedExportManifest, SanitizedExportResult,
    SanitizedImportSummary,
};
use qir_core::validate::{validate_all_incidents, IncidentValidationReportItem};
use qir_core::workspace::WorkspaceMetadata;
use tauri::Manager;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

const WORKSPACE_CONFIG_FILE: &str = "workspace.json";
const WORKSPACE_DEFAULT_DB_FILENAME: &str = "incidentreview.sqlite";
const WORKSPACE_RECENT_LIMIT: usize = 8;
const AI_STORE_DIRNAME: &str = "ai";

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
pub struct AiModelInfo {
    pub name: String,
    pub size: Option<u64>,
    pub digest: Option<String>,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvidenceAddSourceRequest {
    #[serde(rename = "type")]
    pub source_type: AiEvidenceSourceType,
    pub origin: AiEvidenceOrigin,
    pub label: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiIndexBuildRequest {
    pub model: String,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiEvidenceQueryRequest {
    pub query: String,
    pub top_k: u32,
    pub source_filter: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiDraftSectionRequestWire {
    pub section_id: AiSectionId,
    pub quarter_label: String,
    pub prompt: String,
    pub citation_chunk_ids: Vec<String>,
    pub model: String,
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

#[derive(Debug, serde::Serialize)]
pub struct WorkspaceInfo {
    pub current_db_path: String,
    pub recent_db_paths: Vec<String>,
    pub load_error: Option<AppError>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct WorkspaceConfig {
    last_db_path: Option<String>,
    recent_db_paths: Vec<String>,
}

#[derive(Default)]
struct WorkspaceState {
    current_db_path: Mutex<Option<PathBuf>>,
    recent_db_paths: Mutex<Vec<PathBuf>>,
    load_error: Mutex<Option<AppError>>,
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

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let dir = app.path().app_config_dir().map_err(|e| {
        AppError::new("WORKSPACE_PERSIST_FAILED", "Failed to resolve app config directory")
            .with_details(e.to_string())
    })?;
    fs::create_dir_all(&dir).map_err(|e| {
        AppError::new("WORKSPACE_PERSIST_FAILED", "Failed to create app config directory")
            .with_details(format!("path={}; err={}", dir.display(), e))
    })?;
    Ok(dir.join(WORKSPACE_CONFIG_FILE))
}

fn read_workspace_config(app: &tauri::AppHandle) -> Result<WorkspaceConfig, AppError> {
    let path = config_path(app)?;
    if !path.exists() {
        return Ok(WorkspaceConfig {
            last_db_path: None,
            recent_db_paths: Vec::new(),
        });
    }
    let bytes = fs::read(&path).map_err(|e| {
        AppError::new("WORKSPACE_PERSIST_FAILED", "Failed to read workspace config")
            .with_details(format!("path={}; err={}", path.display(), e))
    })?;
    serde_json::from_slice(&bytes).map_err(|e| {
        AppError::new("WORKSPACE_PERSIST_FAILED", "Failed to decode workspace config")
            .with_details(format!("path={}; err={}", path.display(), e))
    })
}

fn write_workspace_config(app: &tauri::AppHandle, cfg: &WorkspaceConfig) -> Result<(), AppError> {
    let path = config_path(app)?;
    let tmp = path.with_extension("tmp");
    let json = serde_json::to_string_pretty(cfg).map_err(|e| {
        AppError::new("WORKSPACE_PERSIST_FAILED", "Failed to encode workspace config")
            .with_details(e.to_string())
    })?;
    fs::write(&tmp, json.as_bytes()).map_err(|e| {
        AppError::new("WORKSPACE_PERSIST_FAILED", "Failed to write workspace config")
            .with_details(format!("path={}; err={}", tmp.display(), e))
    })?;
    fs::rename(&tmp, &path).map_err(|e| {
        AppError::new("WORKSPACE_PERSIST_FAILED", "Failed to finalize workspace config write")
            .with_details(format!("tmp={}; dest={}; err={}", tmp.display(), path.display(), e))
    })?;
    Ok(())
}

fn resolve_current_db_path(app: &tauri::AppHandle, state: &WorkspaceState) -> Result<PathBuf, AppError> {
    if let Some(p) = state.current_db_path.lock().unwrap().clone() {
        return Ok(p);
    }
    default_db_path(app)
}

fn open_current_workspace_conn(
    app: &tauri::AppHandle,
    state: &WorkspaceState,
) -> Result<rusqlite::Connection, AppError> {
    let db_path = resolve_current_db_path(app, state)?;
    qir_core::workspace::open_workspace_connection(&db_path)
}

fn default_artifacts_dir(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let dir = app.path().app_data_dir().map_err(|e| {
        AppError::new("DB_PATH_FAILED", "Failed to resolve app data directory")
            .with_details(e.to_string())
    })?;
    Ok(dir.join("artifacts"))
}

fn ai_store_root(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let dir = app.path().app_data_dir().map_err(|e| {
        AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to resolve app data directory")
            .with_details(e.to_string())
    })?;
    let root = dir.join(AI_STORE_DIRNAME);
    fs::create_dir_all(&root).map_err(|e| {
        AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to create AI store directory")
            .with_details(format!("path={}; err={}", root.display(), e))
    })?;
    Ok(root)
}

fn ai_embedder() -> Result<OllamaEmbedder, AppError> {
    let client = OllamaClient::new("http://127.0.0.1:11434")?;
    Ok(OllamaEmbedder::new(client))
}

fn ai_llm() -> Result<OllamaLlm, AppError> {
    let client = OllamaClient::new("http://127.0.0.1:11434")?;
    Ok(OllamaLlm::new(client))
}

fn now_rfc3339_utc() -> Result<String, AppError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|e| AppError::new("DB_BACKUP_TIME_FAILED", "Failed to format time").with_details(e.to_string()))
}

#[tauri::command]
fn init_db(app: tauri::AppHandle) -> Result<InitDbResponse, AppError> {
    let state = app.state::<WorkspaceState>();
    let db_path = resolve_current_db_path(&app, &state)?;
    let default_path = default_db_path(&app)?;
    let _conn = if db_path.exists() {
        qir_core::workspace::open_workspace_connection(&db_path)?
    } else if db_path == default_path {
        // First-run convenience: create the default app-data workspace on demand.
        qir_core::workspace::create_workspace_connection(&db_path)?
    } else {
        return Err(AppError::new(
            "WORKSPACE_DB_NOT_FOUND",
            "Workspace database file not found",
        )
        .with_details(db_path.display().to_string()));
    };
    Ok(InitDbResponse {
        db_path: db_path.to_string_lossy().to_string(),
    })
}

fn push_recent(state: &WorkspaceState, db_path: &PathBuf) {
    let mut v = state.recent_db_paths.lock().unwrap();
    v.retain(|p| p != db_path);
    v.insert(0, db_path.clone());
    if v.len() > WORKSPACE_RECENT_LIMIT {
        v.truncate(WORKSPACE_RECENT_LIMIT);
    }
}

#[tauri::command]
fn workspace_get_current(app: tauri::AppHandle) -> Result<WorkspaceInfo, AppError> {
    let state = app.state::<WorkspaceState>();
    let current = resolve_current_db_path(&app, &state)?;
    let recent = state
        .recent_db_paths
        .lock()
        .unwrap()
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let load_error = state.load_error.lock().unwrap().clone();
    Ok(WorkspaceInfo {
        current_db_path: current.to_string_lossy().to_string(),
        recent_db_paths: recent,
        load_error,
    })
}

#[tauri::command]
fn workspace_open(app: tauri::AppHandle, db_path: String) -> Result<WorkspaceMetadata, AppError> {
    let state = app.state::<WorkspaceState>();
    let db_path = PathBuf::from(db_path);

    let meta = qir_core::workspace::open_workspace(&db_path)?;

    *state.current_db_path.lock().unwrap() = Some(db_path.clone());
    push_recent(&state, &db_path);

    let cfg = WorkspaceConfig {
        last_db_path: Some(db_path.to_string_lossy().to_string()),
        recent_db_paths: state
            .recent_db_paths
            .lock()
            .unwrap()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
    };
    write_workspace_config(&app, &cfg)?;

    Ok(meta)
}

#[tauri::command]
fn workspace_create(app: tauri::AppHandle, destination_dir: String, filename: Option<String>) -> Result<WorkspaceMetadata, AppError> {
    let state = app.state::<WorkspaceState>();
    let dir = PathBuf::from(destination_dir);
    if !dir.is_dir() {
        return Err(AppError::new(
            "WORKSPACE_INVALID_PATH",
            "Workspace destination must be an existing directory",
        )
        .with_details(dir.display().to_string()));
    }

    let name = filename.unwrap_or_else(|| WORKSPACE_DEFAULT_DB_FILENAME.to_string());
    let db_path = dir.join(name);

    let meta = qir_core::workspace::create_workspace(&db_path)?;

    *state.current_db_path.lock().unwrap() = Some(db_path.clone());
    push_recent(&state, &db_path);

    let cfg = WorkspaceConfig {
        last_db_path: Some(db_path.to_string_lossy().to_string()),
        recent_db_paths: state
            .recent_db_paths
            .lock()
            .unwrap()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
    };
    write_workspace_config(&app, &cfg)?;

    Ok(meta)
}

#[tauri::command]
fn seed_demo_jira(app: tauri::AppHandle) -> Result<JiraImportSummary, AppError> {
    let state = app.state::<WorkspaceState>();
    let mut conn = open_current_workspace_conn(&app, &state)?;

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
    let state = app.state::<WorkspaceState>();
    let mut conn = open_current_workspace_conn(&app, &state)?;
    core_seed_demo_dataset(&mut conn)
}

#[tauri::command]
fn get_dashboard_v1(app: tauri::AppHandle) -> Result<DashboardPayloadV1, AppError> {
    let state = app.state::<WorkspaceState>();
    let conn = open_current_workspace_conn(&app, &state)?;
    qir_core::analytics::build_dashboard_payload_v1(&conn)
}

#[tauri::command]
fn get_dashboard_v2(app: tauri::AppHandle) -> Result<DashboardPayloadV2, AppError> {
    let state = app.state::<WorkspaceState>();
    let conn = open_current_workspace_conn(&app, &state)?;
    qir_core::analytics::build_dashboard_payload_v2(&conn)
}

#[tauri::command]
fn generate_report_md(app: tauri::AppHandle) -> Result<String, AppError> {
    let state = app.state::<WorkspaceState>();
    let conn = open_current_workspace_conn(&app, &state)?;
    generate_qir_markdown(&conn)
}

#[tauri::command]
fn jira_csv_preview(csv_text: String, max_rows: usize) -> Result<JiraCsvPreview, AppError> {
    preview_jira_csv(&csv_text, max_rows)
}

#[tauri::command]
fn jira_profiles_list(app: tauri::AppHandle) -> Result<Vec<JiraMappingProfile>, AppError> {
    let state = app.state::<WorkspaceState>();
    let conn = open_current_workspace_conn(&app, &state)?;
    list_profiles(&conn)
}

#[tauri::command]
fn jira_profiles_upsert(
    app: tauri::AppHandle,
    profile: JiraMappingProfileUpsert,
) -> Result<JiraMappingProfile, AppError> {
    let state = app.state::<WorkspaceState>();
    let mut conn = open_current_workspace_conn(&app, &state)?;
    upsert_profile(&mut conn, profile)
}

#[tauri::command]
fn jira_profiles_delete(app: tauri::AppHandle, id: i64) -> Result<DeleteResponse, AppError> {
    let state = app.state::<WorkspaceState>();
    let mut conn = open_current_workspace_conn(&app, &state)?;
    delete_profile(&mut conn, id)?;
    Ok(DeleteResponse { ok: true })
}

#[tauri::command]
fn jira_import_using_profile(
    app: tauri::AppHandle,
    profile_id: i64,
    csv_text: String,
) -> Result<JiraImportSummary, AppError> {
    let state = app.state::<WorkspaceState>();
    let mut conn = open_current_workspace_conn(&app, &state)?;
    let profile = qir_core::profiles::jira::get_profile(&conn, profile_id)?;
    import_jira_csv(&mut conn, &csv_text, &profile.mapping)
}

#[tauri::command]
fn incidents_list(app: tauri::AppHandle) -> Result<Vec<IncidentListItem>, AppError> {
    let state = app.state::<WorkspaceState>();
    let conn = open_current_workspace_conn(&app, &state)?;
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
    let state = app.state::<WorkspaceState>();
    let conn = open_current_workspace_conn(&app, &state)?;
    qir_core::repo::get_incident_detail(&conn, incident_id)
}

#[tauri::command]
fn validation_report(app: tauri::AppHandle) -> Result<Vec<IncidentValidationReportItem>, AppError> {
    let state = app.state::<WorkspaceState>();
    let conn = open_current_workspace_conn(&app, &state)?;
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
    let state = app.state::<WorkspaceState>();
    let mut conn = open_current_workspace_conn(&app, &state)?;
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
fn ai_models_list() -> Result<Vec<AiModelInfo>, AppError> {
    let client = OllamaClient::new("http://127.0.0.1:11434")?;
    let mut models = client.list_models()?;
    models.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(models
        .into_iter()
        .map(|m: OllamaModelInfo| AiModelInfo {
            name: m.name,
            size: m.size,
            digest: m.digest,
            modified_at: m.modified_at,
        })
        .collect())
}

#[tauri::command]
fn ai_evidence_add_source(
    app: tauri::AppHandle,
    req: EvidenceAddSourceRequest,
) -> Result<AiEvidenceSource, AppError> {
    let root = ai_store_root(&app)?;
    let store = AiEvidenceStore::open(root);
    let created_at = now_rfc3339_utc()?;
    store.add_source(AiEvidenceAddSourceInput {
        source_type: req.source_type,
        origin: req.origin,
        label: req.label,
        created_at,
        text: req.text,
    })
}

#[tauri::command]
fn ai_evidence_list_sources(app: tauri::AppHandle) -> Result<Vec<AiEvidenceSource>, AppError> {
    let root = ai_store_root(&app)?;
    let store = AiEvidenceStore::open(root);
    store.list_sources()
}

#[tauri::command]
fn ai_evidence_build_chunks(
    app: tauri::AppHandle,
    source_id: Option<String>,
) -> Result<AiBuildChunksResult, AppError> {
    let root = ai_store_root(&app)?;
    let store = AiEvidenceStore::open(root);
    let updated_at = now_rfc3339_utc()?;
    store.build_chunks(source_id, &updated_at)
}

#[tauri::command]
fn ai_evidence_list_chunks(
    app: tauri::AppHandle,
    source_id: Option<String>,
) -> Result<Vec<AiEvidenceChunkSummary>, AppError> {
    let root = ai_store_root(&app)?;
    let store = AiEvidenceStore::open(root);
    store.list_chunks(AiEvidenceQueryStore {
        include_text: false,
        source_id,
    })
}

#[tauri::command]
fn ai_index_status(app: tauri::AppHandle) -> Result<AiIndexStatus, AppError> {
    let root = ai_store_root(&app)?;
    let index = AiIndexStore::open(root);
    index.status()
}

#[tauri::command]
fn ai_index_build(app: tauri::AppHandle, req: AiIndexBuildRequest) -> Result<AiIndexStatus, AppError> {
    let root = ai_store_root(&app)?;
    let evidence = AiEvidenceStore::open(root.clone());
    let index = AiIndexStore::open(root);
    let embedder = ai_embedder()?;
    let updated_at = now_rfc3339_utc()?;
    index.build_with_embedder(
        &evidence,
        &embedder,
        AiIndexBuildInput {
            model: req.model,
            source_id: req.source_id,
            updated_at,
        },
    )
}

#[tauri::command]
fn ai_evidence_query(
    app: tauri::AppHandle,
    req: AiEvidenceQueryRequest,
) -> Result<AiEvidenceQueryResponse, AppError> {
    let root = ai_store_root(&app)?;
    let evidence = AiEvidenceStore::open(root.clone());
    let index = AiIndexStore::open(root);
    let embedder = ai_embedder()?;
    ai_query_with_embedder(
        &evidence,
        &index,
        &embedder,
        &req.query,
        req.top_k,
        req.source_filter.as_deref(),
    )
}

#[tauri::command]
fn ai_draft_section(
    app: tauri::AppHandle,
    req: AiDraftSectionRequestWire,
) -> Result<AiDraftResponse, AppError> {
    let root = ai_store_root(&app)?;
    let evidence = AiEvidenceStore::open(root);
    let llm = ai_llm()?;

    ai_draft_with_llm(
        &evidence,
        &llm,
        &req.model,
        AiDraftSectionRequest {
            section_id: req.section_id,
            quarter_label: req.quarter_label,
            prompt: req.prompt,
            citation_chunk_ids: req.citation_chunk_ids,
        },
    )
}

#[tauri::command]
fn backup_create(app: tauri::AppHandle, destination_dir: String) -> Result<BackupCreateResult, AppError> {
    let state = app.state::<WorkspaceState>();
    let db_path = resolve_current_db_path(&app, &state)?;
    let conn = qir_core::workspace::open_workspace_connection(&db_path)?;
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
    let state = app.state::<WorkspaceState>();
    let db_path = resolve_current_db_path(&app, &state)?;
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
    let state = app.state::<WorkspaceState>();
    let conn = open_current_workspace_conn(&app, &state)?;
    let export_time = now_rfc3339_utc()?;
    let dest_root = PathBuf::from(destination_dir);
    core_export_sanitized_dataset(&conn, dest_root.as_path(), &export_time, env!("CARGO_PKG_VERSION"))
}

#[tauri::command]
fn inspect_sanitized_dataset(dataset_dir: String) -> Result<SanitizedExportManifest, AppError> {
    core_inspect_sanitized_dataset(PathBuf::from(dataset_dir).as_path())
}

#[tauri::command]
fn import_sanitized_dataset(
    app: tauri::AppHandle,
    dataset_dir: String,
) -> Result<SanitizedImportSummary, AppError> {
    let state = app.state::<WorkspaceState>();
    let mut conn = open_current_workspace_conn(&app, &state)?;
    core_import_sanitized_dataset(&mut conn, PathBuf::from(dataset_dir).as_path())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(WorkspaceState::default())
        .setup(|app| {
            let handle = app.handle();
            let state = app.state::<WorkspaceState>();
            let cfg = match read_workspace_config(&handle) {
                Ok(c) => c,
                Err(e) => {
                    *state.load_error.lock().unwrap() = Some(e);
                    WorkspaceConfig {
                        last_db_path: None,
                        recent_db_paths: Vec::new(),
                    }
                }
            };

            if let Some(p) = cfg.last_db_path.as_deref() {
                *state.current_db_path.lock().unwrap() = Some(PathBuf::from(p));
            }
            let recent = cfg
                .recent_db_paths
                .iter()
                .map(PathBuf::from)
                .collect::<Vec<_>>();
            *state.recent_db_paths.lock().unwrap() = recent;
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            init_db,
            workspace_get_current,
            workspace_open,
            workspace_create,
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
            ai_models_list,
            ai_health_check,
            ai_evidence_add_source,
            ai_evidence_list_sources,
            ai_evidence_build_chunks,
            ai_evidence_list_chunks,
            ai_index_status,
            ai_index_build,
            ai_evidence_query,
            ai_draft_section,
            backup_create,
            backup_inspect,
            restore_from_backup,
            export_sanitized_dataset,
            inspect_sanitized_dataset,
            import_sanitized_dataset
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
