import { useEffect, useMemo, useState } from "react";

import { extractAppError, invokeValidated } from "./lib/tauri";
import { guidanceForSanitizedImportErrorCode } from "./lib/sanitized_import_guidance";
import { guidanceForWorkspaceErrorCode } from "./lib/workspace_guidance";
import { pickDbFile, pickDirectory } from "./lib/pickers";
import {
  DashboardPayloadV2Schema,
  InitDbResponseSchema,
  DeleteResponseSchema,
  JiraCsvPreviewSchema,
  JiraImportSummarySchema,
  JiraMappingProfileListSchema,
  JiraMappingProfileSchema,
  JiraMappingProfileUpsertSchema,
  AiHealthStatusSchema,
  IncidentListSchema,
  IncidentDetailSchema,
  BackupCreateResultSchema,
  BackupManifestSchema,
  RestoreResultSchema,
  SanitizedExportManifestSchema,
  SanitizedExportResultSchema,
  SanitizedImportSummarySchema,
  SlackPreviewSchema,
  SlackIngestSummarySchema,
  ValidationReportSchema,
  WorkspaceInfoSchema,
  WorkspaceMetadataSchema,
  WorkspaceMigrationStatusSchema,
} from "./lib/schemas";
import { AppNav } from "./ui/AppNav";
import { Modal } from "./ui/Modal";
import { ToastHost } from "./ui/ToastHost";
import { useToasts } from "./ui/useToasts";
import { WorkspaceSection } from "./features/workspace/WorkspaceSection";
import { JiraImportSection } from "./features/import_jira/JiraImportSection";
import { SlackImportSection } from "./features/import_slack/SlackImportSection";
import { SanitizedImportSection } from "./features/import_sanitized/SanitizedImportSection";
import { BackupRestoreSection } from "./features/backup_restore/BackupRestoreSection";
import { ValidationSection } from "./features/validation/ValidationSection";
import { ReportSection } from "./features/report/ReportSection";
import { DashboardsSection } from "./features/dashboards/DashboardsSection";
import { IncidentDetailDrawer } from "./features/dashboards/IncidentDetailDrawer";
import { AiSection } from "./features/ai/AiSection";
import { AboutSection } from "./features/about/AboutSection";

export default function App() {
  const { toasts, pushToast, dismissToast } = useToasts();

  const [dbPath, setDbPath] = useState<string | null>(null);
  const [seedInserted, setSeedInserted] = useState<number | null>(null);
  const [jiraProfiles, setJiraProfiles] = useState<
    Array<{
      id: number;
      name: string;
      mapping: {
        external_id?: string | null;
        title: string;
        description?: string | null;
        severity?: string | null;
        detection_source?: string | null;
        vendor?: string | null;
        service?: string | null;
        impact_pct?: string | null;
        service_health_pct?: string | null;
        start_ts?: string | null;
        first_observed_ts?: string | null;
        it_awareness_ts?: string | null;
        ack_ts?: string | null;
        mitigate_ts?: string | null;
        resolve_ts?: string | null;
      };
    }>
  >([]);
  const [selectedProfileId, setSelectedProfileId] = useState<number | null>(null);
  const [profileName, setProfileName] = useState<string>("");
  const [csvText, setCsvText] = useState<string>("");
  const [csvFileName, setCsvFileName] = useState<string>("");
  const [csvPreview, setCsvPreview] = useState<null | { headers: string[]; rows: string[][] }>(null);
  const [mapping, setMapping] = useState<{
    external_id: string | null;
    title: string;
    description: string | null;
    severity: string | null;
    detection_source: string | null;
    vendor: string | null;
    service: string | null;
    impact_pct: string | null;
    service_health_pct: string | null;
    start_ts: string | null;
    first_observed_ts: string | null;
    it_awareness_ts: string | null;
    ack_ts: string | null;
    mitigate_ts: string | null;
    resolve_ts: string | null;
  }>({
    external_id: null,
    title: "",
    description: null,
    severity: null,
    detection_source: null,
    vendor: null,
    service: null,
    impact_pct: null,
    service_health_pct: null,
    start_ts: null,
    first_observed_ts: null,
    it_awareness_ts: null,
    ack_ts: null,
    mitigate_ts: null,
    resolve_ts: null,
  });
  const [importSummary, setImportSummary] = useState<null | {
    inserted: number;
    updated: number;
    skipped: number;
    conflicts: Array<{ row: number; reason: string; external_id?: string | null; fingerprint?: string | null }>;
    warnings: Array<{ code: string; message: string; details?: string | null }>;
  }>(null);
  const [dashboard, setDashboard] = useState<null | {
    version: number;
    incident_count: number;
    severity_counts: Array<{ severity: string; count: number; incident_ids: number[] }>;
    incidents: Array<{
      id: number;
      external_id: string | null;
      title: string;
      severity: string | null;
      detection_source: string | null;
      vendor: string | null;
      service: string | null;
      it_awareness_lag_seconds: number | null;
      time_to_mitigation_seconds: number | null;
      mttr_seconds: number | null;
      warning_count: number;
    }>;
    detection_story: {
      detection_source_mix: Array<{ key: string; label: string; count: number; incident_ids: number[] }>;
      it_awareness_lag_buckets: Array<{ key: string; label: string; count: number; incident_ids: number[] }>;
    };
    vendor_service_story: {
      top_vendors_by_count: Array<{ key: string; label: string; count: number; incident_ids: number[] }>;
      top_services_by_count: Array<{ key: string; label: string; count: number; incident_ids: number[] }>;
      top_vendors_by_pain: Array<{
        key: string;
        label: string;
        count: number;
        pain_sum: number;
        pain_known_count: number;
        incident_ids: number[];
      }>;
      top_services_by_pain: Array<{
        key: string;
        label: string;
        count: number;
        pain_sum: number;
        pain_known_count: number;
        incident_ids: number[];
      }>;
    };
    response_story: {
      time_to_mitigation_buckets: Array<{ key: string; label: string; count: number; incident_ids: number[] }>;
      time_to_resolve_buckets: Array<{ key: string; label: string; count: number; incident_ids: number[] }>;
    };
  }>(null);
  const [selectedSeverity, setSelectedSeverity] = useState<string | null>(null);
  const [reportMd, setReportMd] = useState<string>("");
  const [incidentFilterIds, setIncidentFilterIds] = useState<number[] | null>(null);
  const [incidentFilterLabel, setIncidentFilterLabel] = useState<string>("");

  const [workspaceInfo, setWorkspaceInfo] = useState<null | {
    current_db_path: string;
    recent_db_paths: string[];
    load_error?: { code: string; message: string; details?: string | null; retryable: boolean } | null;
  }>(null);
  const [workspaceMeta, setWorkspaceMeta] = useState<null | { db_path: string; is_empty: boolean }>(null);
  const [workspaceNewFilename, setWorkspaceNewFilename] = useState<string>("incidentreview.sqlite");
  const [workspaceRecentPick, setWorkspaceRecentPick] = useState<string>("");
  const [migrationGuard, setMigrationGuard] = useState<null | {
    action: "init_db" | "open_workspace";
    dbPath: string;
    latestMigration: string;
    pendingMigrations: string[];
  }>(null);

  const [backupResult, setBackupResult] = useState<null | {
    backup_dir: string;
    manifest: {
      manifest_version: number;
      app_version: string;
      export_time: string;
      schema_migrations: string[];
      counts: { incidents: number; timeline_events: number; artifacts_rows: number };
      db: { filename: string; sha256: string; bytes: number };
      artifacts: { included: boolean; files: Array<{ rel_path: string; sha256: string; bytes: number }> };
    };
  }>(null);

  const [restoreBackupDir, setRestoreBackupDir] = useState<string>("");
  const [restoreManifest, setRestoreManifest] = useState<null | {
    manifest_version: number;
    app_version: string;
    export_time: string;
    schema_migrations: string[];
    counts: { incidents: number; timeline_events: number; artifacts_rows: number };
    db: { filename: string; sha256: string; bytes: number };
    artifacts: { included: boolean; files: Array<{ rel_path: string; sha256: string; bytes: number }> };
  }>(null);
  const [restoreAllowOverwrite, setRestoreAllowOverwrite] = useState<boolean>(false);
  const [restoreResult, setRestoreResult] = useState<null | { ok: boolean; restored_db_path: string; restored_artifacts: boolean }>(
    null
  );

  const [sanitizedExport, setSanitizedExport] = useState<null | { export_dir: string; incident_count: number }>(null);

  const [sanitizedImportDir, setSanitizedImportDir] = useState<string>("");
  const [sanitizedImportManifest, setSanitizedImportManifest] = useState<null | {
    manifest_version: number;
    app_version: string;
    export_time: string;
    incident_count: number;
    files: Array<{ filename: string; bytes: number; sha256: string }>;
  }>(null);
  const [sanitizedImportSummary, setSanitizedImportSummary] = useState<null | {
    inserted_incidents: number;
    inserted_timeline_events: number;
    import_warnings: Array<{ code: string; message: string; details?: string | null }>;
  }>(null);

  const [incidentDetailOpen, setIncidentDetailOpen] = useState<boolean>(false);
  const [incidentDetailLoading, setIncidentDetailLoading] = useState<boolean>(false);
  const [incidentDetail, setIncidentDetail] = useState<null | {
    incident: {
      id: number;
      external_id: string | null;
      fingerprint: string;
      title: string;
      description: string | null;
      severity: string | null;
      detection_source: string | null;
      vendor: string | null;
      service: string | null;
      impact_pct: number | null;
      service_health_pct: number | null;
      start_ts: string | null;
      first_observed_ts: string | null;
      it_awareness_ts: string | null;
      ack_ts: string | null;
      mitigate_ts: string | null;
      resolve_ts: string | null;
      start_ts_raw: string | null;
      first_observed_ts_raw: string | null;
      it_awareness_ts_raw: string | null;
      ack_ts_raw: string | null;
      mitigate_ts_raw: string | null;
      resolve_ts_raw: string | null;
    };
    metrics: {
      mttd_seconds: number | null;
      it_awareness_lag_seconds: number | null;
      mtta_seconds: number | null;
      time_to_mitigation_seconds: number | null;
      mttr_seconds: number | null;
    };
    warnings: Array<{ code: string; message: string; details?: string | null }>;
    artifacts: Array<{
      id: number;
      incident_id: number | null;
      kind: string;
      sha256: string;
      filename: string | null;
      mime_type: string | null;
      text: string | null;
      created_at: string;
    }>;
    timeline_events: Array<{
      id: number;
      incident_id: number | null;
      source: string;
      ts: string | null;
      author: string | null;
      kind: string | null;
      text: string;
      raw_json: string | null;
      created_at: string;
    }>;
  }>(null);

  const [incidentOptions, setIncidentOptions] = useState<
    Array<{ id: number; external_id: string | null; title: string }>
  >([]);

  const [slackTargetMode, setSlackTargetMode] = useState<"existing" | "new">("existing");
  const [slackExistingIncidentId, setSlackExistingIncidentId] = useState<number | null>(null);
  const [slackNewIncidentTitle, setSlackNewIncidentTitle] = useState<string>("");
  const [slackFileName, setSlackFileName] = useState<string>("");
  const [slackText, setSlackText] = useState<string>("");
  const [slackPreview, setSlackPreview] = useState<null | {
    detected_format: string;
    line_count: number;
    message_count: number;
    warnings: Array<{ code: string; message: string; details?: string | null }>;
  }>(null);
  const [slackSummary, setSlackSummary] = useState<null | {
    incident_id: number;
    incident_created: boolean;
    detected_format: string;
    inserted_events: number;
    warnings: Array<{ code: string; message: string; details?: string | null }>;
  }>(null);

  const [validationReport, setValidationReport] = useState<
    null | Array<{ id: number; external_id: string | null; title: string; warnings: Array<{ code: string; message: string; details?: string | null }> }>
  >(null);

  async function preflightWorkspaceMigrations(dbPath: string, action: "init_db" | "open_workspace") {
    try {
      const status = await invokeValidated(
        "workspace_migration_status",
        { dbPath },
        WorkspaceMigrationStatusSchema
      );
      if (status.pendingMigrations.length > 0) {
        setMigrationGuard({
          action,
          dbPath: status.dbPath,
          latestMigration: status.latestMigration,
          pendingMigrations: status.pendingMigrations,
        });
        return false;
      }
      return true;
    } catch (e) {
      const appErr = extractAppError(e);
      // First-run and "default workspace does not exist yet" is OK; init_db can create it.
      if (appErr?.code === "WORKSPACE_DB_NOT_FOUND") return true;
      pushToast({ kind: "error", title: "Migration preflight failed", message: String(e) });
      return false;
    }
  }

  async function ensureDbInitialized(opts?: { toastOnSuccess?: boolean; skipPreflight?: boolean }) {
    const toastOnSuccess = opts?.toastOnSuccess ?? false;
    const skipPreflight = opts?.skipPreflight ?? false;

    if (!skipPreflight) {
      try {
        const info = await invokeValidated("workspace_get_current", undefined, WorkspaceInfoSchema);
        const okToMigrate = await preflightWorkspaceMigrations(info.current_db_path, "init_db");
        if (!okToMigrate) return;
      } catch (e) {
        pushToast({ kind: "error", title: "Workspace load failed", message: String(e) });
        return;
      }
    }

    try {
      const res = await invokeValidated("init_db", undefined, InitDbResponseSchema);
      setDbPath(res.db_path);
      if (toastOnSuccess) {
        pushToast({ kind: "success", title: "DB initialized", message: res.db_path });
      }
    } catch (e) {
      const appErr = extractAppError(e);
      if (appErr) {
        const guidance = guidanceForWorkspaceErrorCode(appErr.code);
        pushToast({
          kind: "error",
          title: "Workspace init failed",
          message: guidance ? `${appErr.code}: ${appErr.message}\n\n${guidance}` : `${appErr.code}: ${appErr.message}`,
        });
      } else {
        pushToast({ kind: "error", title: "Workspace init failed", message: String(e) });
      }
    }
  }

  useEffect(() => {
    void (async () => {
      try {
        const info = await invokeValidated("workspace_get_current", undefined, WorkspaceInfoSchema);
        setWorkspaceInfo(info);
        setDbPath(info.current_db_path);
        if (info.recent_db_paths.length > 0) {
          setWorkspaceRecentPick(info.recent_db_paths[0] ?? "");
        }
        if (info.load_error) {
          pushToast({
            kind: "warning",
            title: "Workspace config warning",
            message: `${info.load_error.code}: ${info.load_error.message}`,
          });
        }

        // Startup migration guard: if the selected workspace has pending migrations, prompt to back up before we migrate.
        const okToMigrate = await preflightWorkspaceMigrations(info.current_db_path, "init_db");
        if (!okToMigrate) return;
      } catch (e) {
        pushToast({ kind: "error", title: "Workspace load failed", message: String(e) });
        return;
      }

      // Ensure the current workspace DB is usable. For the default app-data workspace this
      // creates the DB on first run.
      await ensureDbInitialized({ toastOnSuccess: false, skipPreflight: true });
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function clearWorkspaceScopedState() {
    setSeedInserted(null);
    setDashboard(null);
    setSelectedSeverity(null);
    setIncidentFilterIds(null);
    setIncidentFilterLabel("");
    setReportMd("");
    setValidationReport(null);
    setIncidentOptions([]);
    setIncidentDetailOpen(false);
    setIncidentDetail(null);
    setImportSummary(null);
    setBackupResult(null);
    setRestoreBackupDir("");
    setRestoreManifest(null);
    setRestoreAllowOverwrite(false);
    setRestoreResult(null);
    setSanitizedExport(null);
    setSanitizedImportDir("");
    setSanitizedImportManifest(null);
    setSanitizedImportSummary(null);
  }

  async function onInitDb() {
    await ensureDbInitialized({ toastOnSuccess: true, skipPreflight: false });
  }

  async function onSeedDemo() {
    try {
      const res = await invokeValidated("seed_demo_dataset", undefined, JiraImportSummarySchema);
      setSeedInserted(res.inserted);
      if (res.warnings.length > 0 || res.conflicts.length > 0) {
        pushToast({
          kind: res.conflicts.length > 0 ? "error" : "warning",
          title: "Seeded with issues",
          message: `${res.inserted} inserted, ${res.warnings.length} warnings, ${res.conflicts.length} conflicts`,
        });
      } else {
        pushToast({ kind: "success", title: "Demo seeded", message: `${res.inserted} incidents` });
      }
    } catch (e) {
      pushToast({ kind: "error", title: "Seed failed", message: String(e) });
    }
  }

  async function onRefreshProfiles() {
    try {
      const res = await invokeValidated("jira_profiles_list", undefined, JiraMappingProfileListSchema);
      setJiraProfiles(res);
    } catch (e) {
      pushToast({ kind: "error", title: "Load profiles failed", message: String(e) });
    }
  }

  function applyCommonJiraDefaults() {
    if (!csvPreview) return;
    const headers = csvPreview.headers;
    const pick = (candidates: string[]) => {
      for (const c of candidates) {
        if (headers.includes(c)) return c;
      }
      return null;
    };
    setMapping((m) => ({
      ...m,
      external_id: pick(["Key", "Issue key", "Issue Key"]),
      title: pick(["Summary", "Title"]) ?? m.title,
      description: pick(["Description"]),
      severity: pick(["Severity", "Priority"]),
      detection_source: pick(["DetectionSource", "Detection Source"]),
      vendor: pick(["Vendor", "Provider"]),
      service: pick(["Service", "Component"]),
      impact_pct: pick(["ImpactPct", "Impact %"]),
      service_health_pct: pick(["ServiceHealthPct", "Service Health %"]),
      start_ts: pick(["StartTs", "Start", "Start TS"]),
      first_observed_ts: pick(["FirstObservedTs", "First Observed", "First Observed TS"]),
      it_awareness_ts: pick(["ItAwarenessTs", "IT Awareness", "IT Awareness TS"]),
      ack_ts: pick(["AckTs", "Ack", "Ack TS"]),
      mitigate_ts: pick(["MitigateTs", "Mitigate", "Mitigate TS"]),
      resolve_ts: pick(["ResolveTs", "Resolve", "Resolve TS"]),
    }));
  }

  async function onSaveProfile() {
    try {
      if (profileName.trim().length === 0) {
        pushToast({ kind: "error", title: "Profile name required", message: "Enter a profile name." });
        return;
      }
      if (mapping.title.trim().length === 0) {
        pushToast({ kind: "error", title: "Title mapping required", message: "Map the required Title field." });
        return;
      }
      const payload = JiraMappingProfileUpsertSchema.parse({
        id: selectedProfileId,
        name: profileName,
        mapping,
      });
      const saved = await invokeValidated("jira_profiles_upsert", { profile: payload }, JiraMappingProfileSchema);
      setSelectedProfileId(saved.id);
      pushToast({ kind: "success", title: "Profile saved", message: `${saved.name} (id=${saved.id})` });
      await onRefreshProfiles();
    } catch (e) {
      pushToast({ kind: "error", title: "Save profile failed", message: String(e) });
    }
  }

  async function onDeleteProfile() {
    if (!selectedProfileId) return;
    try {
      await invokeValidated("jira_profiles_delete", { id: selectedProfileId }, DeleteResponseSchema);
      pushToast({ kind: "success", title: "Profile deleted", message: `id=${selectedProfileId}` });
      setSelectedProfileId(null);
      setProfileName("");
      await onRefreshProfiles();
    } catch (e) {
      pushToast({ kind: "error", title: "Delete profile failed", message: String(e) });
    }
  }

  async function onPickCsvFile(file: File | null) {
    setImportSummary(null);
    if (!file) return;
    const text = await file.text();
    setCsvFileName(file.name);
    setCsvText(text);
    try {
      const prev = await invokeValidated("jira_csv_preview", { csvText: text, maxRows: 5 }, JiraCsvPreviewSchema);
      setCsvPreview(prev);
      pushToast({ kind: "success", title: "CSV loaded", message: `${prev.headers.length} columns` });
    } catch (e) {
      pushToast({ kind: "error", title: "CSV preview failed", message: String(e) });
    }
  }

  async function onImportCsv() {
    try {
      if (!csvText) {
        pushToast({ kind: "error", title: "No CSV selected", message: "Choose a CSV file first." });
        return;
      }
      if (!selectedProfileId) {
        pushToast({
          kind: "error",
          title: "No profile selected",
          message: "Save/select a mapping profile before importing.",
        });
        return;
      }
      const res = await invokeValidated(
        "jira_import_using_profile",
        { profileId: selectedProfileId, csvText },
        JiraImportSummarySchema
      );
      setImportSummary(res);
      const kind = res.conflicts.length > 0 ? "error" : res.warnings.length > 0 ? "warning" : "success";
      pushToast({
        kind,
        title: "Import complete",
        message: `inserted=${res.inserted}, updated=${res.updated}, skipped=${res.skipped}, conflicts=${res.conflicts.length}, warnings=${res.warnings.length}`,
      });
    } catch (e) {
      pushToast({ kind: "error", title: "Import failed", message: String(e) });
    }
  }

  async function onLoadDashboard() {
    try {
      const res = await invokeValidated("get_dashboard_v2", undefined, DashboardPayloadV2Schema);
      setDashboard(res);
      setSelectedSeverity(null);
      setIncidentFilterIds(null);
      setIncidentFilterLabel("");
      pushToast({ kind: "success", title: "Dashboard loaded", message: `${res.incident_count} incidents` });
    } catch (e) {
      pushToast({ kind: "error", title: "Dashboard failed", message: String(e) });
    }
  }

  async function onGenerateReport() {
    try {
      const md = await invokeValidated<string>("generate_report_md", undefined, null);
      setReportMd(md);
      pushToast({ kind: "success", title: "Report generated", message: `${md.length} chars` });
    } catch (e) {
      pushToast({ kind: "error", title: "Report failed", message: String(e) });
    }
  }

  async function onAiHealthCheck() {
    try {
      const res = await invokeValidated("ai_health_check", undefined, AiHealthStatusSchema);
      pushToast({ kind: "success", title: "AI OK", message: res.message });
    } catch (e) {
      pushToast({ kind: "error", title: "AI unavailable", message: String(e) });
    }
  }
  const navItems = useMemo(
    () => [
      { label: "Workspace", href: "#workspace" },
      { label: "Imports: Jira", href: "#jira" },
      { label: "Imports: Slack", href: "#slack" },
      { label: "Imports: Sanitized", href: "#sanitized" },
      { label: "Validation/Anomalies", href: "#validation" },
      { label: "Dashboards", href: "#dashboards", kind: "accent" as const },
      { label: "Backup/Restore", href: "#data" },
      { label: "Report", href: "#report" },
      { label: "AI (Phase 5)", href: "#ai" },
      { label: "About", href: "#about" },
    ],
    []
  );

  async function refreshAllViewsAfterWorkspaceChange() {
    await onRefreshIncidentsList();
    await onLoadDashboard();
    await onGenerateReport();
    await onRefreshValidationReport();
  }

  async function onWorkspaceCreate() {
    try {
      const dest = await pickDirectory();
      if (!dest) return;
      clearWorkspaceScopedState();
      const meta = await invokeValidated(
        "workspace_create",
        { destinationDir: dest, filename: workspaceNewFilename || "incidentreview.sqlite" },
        WorkspaceMetadataSchema
      );
      setWorkspaceMeta(meta);
      setDbPath(meta.db_path);
      const info = await invokeValidated("workspace_get_current", undefined, WorkspaceInfoSchema);
      setWorkspaceInfo(info);
      pushToast({ kind: "success", title: "Workspace created", message: meta.db_path });
      await refreshAllViewsAfterWorkspaceChange();
    } catch (e) {
      const appErr = extractAppError(e);
      if (appErr) {
        const guidance = guidanceForWorkspaceErrorCode(appErr.code);
        pushToast({
          kind: "error",
          title: "Workspace create failed",
          message: guidance ? `${appErr.code}: ${appErr.message}\n\n${guidance}` : `${appErr.code}: ${appErr.message}`,
        });
      } else {
        pushToast({ kind: "error", title: "Workspace create failed", message: String(e) });
      }
    }
  }

  async function openWorkspaceAtPath(file: string) {
    clearWorkspaceScopedState();
    const meta = await invokeValidated("workspace_open", { dbPath: file }, WorkspaceMetadataSchema);
    setWorkspaceMeta(meta);
    setDbPath(meta.db_path);
    const info = await invokeValidated("workspace_get_current", undefined, WorkspaceInfoSchema);
    setWorkspaceInfo(info);
    pushToast({ kind: "success", title: "Workspace opened", message: meta.db_path });
    await refreshAllViewsAfterWorkspaceChange();
  }

  async function onWorkspaceOpen() {
    try {
      const file = await pickDbFile();
      if (!file) return;
      const okToMigrate = await preflightWorkspaceMigrations(file, "open_workspace");
      if (!okToMigrate) return;
      await openWorkspaceAtPath(file);
    } catch (e) {
      const appErr = extractAppError(e);
      if (appErr) {
        const guidance = guidanceForWorkspaceErrorCode(appErr.code);
        pushToast({
          kind: "error",
          title: "Workspace open failed",
          message: guidance ? `${appErr.code}: ${appErr.message}\n\n${guidance}` : `${appErr.code}: ${appErr.message}`,
        });
      } else {
        pushToast({ kind: "error", title: "Workspace open failed", message: String(e) });
      }
    }
  }

  async function onWorkspaceSwitchToRecent() {
    try {
      if (!workspaceRecentPick) {
        pushToast({ kind: "error", title: "No workspace selected", message: "Pick a recent workspace first." });
        return;
      }
      const okToMigrate = await preflightWorkspaceMigrations(workspaceRecentPick, "open_workspace");
      if (!okToMigrate) return;
      await openWorkspaceAtPath(workspaceRecentPick);
    } catch (e) {
      const appErr = extractAppError(e);
      if (appErr) {
        const guidance = guidanceForWorkspaceErrorCode(appErr.code);
        pushToast({
          kind: "error",
          title: "Workspace switch failed",
          message: guidance ? `${appErr.code}: ${appErr.message}\n\n${guidance}` : `${appErr.code}: ${appErr.message}`,
        });
      } else {
        pushToast({ kind: "error", title: "Workspace switch failed", message: String(e) });
      }
    }
  }

  async function onBackupCreate() {
    try {
      const dest = await pickDirectory();
      if (!dest) return;
      const res = await invokeValidated("backup_create", { destinationDir: dest }, BackupCreateResultSchema);
      setBackupResult(res);
      pushToast({
        kind: "success",
        title: "Backup created",
        message: `${res.backup_dir} (incidents=${res.manifest.counts.incidents})`,
      });
    } catch (e) {
      pushToast({ kind: "error", title: "Backup failed", message: String(e) });
    }
  }

  async function onPickBackupForRestore() {
    try {
      const dir = await pickDirectory();
      if (!dir) return;
      setRestoreBackupDir(dir);
      setRestoreAllowOverwrite(false);
      setRestoreResult(null);
      const manifest = await invokeValidated("backup_inspect", { backupDir: dir }, BackupManifestSchema);
      setRestoreManifest(manifest);
      pushToast({
        kind: "success",
        title: "Backup selected",
        message: `${manifest.export_time} (incidents=${manifest.counts.incidents})`,
      });
    } catch (e) {
      setRestoreBackupDir("");
      setRestoreManifest(null);
      pushToast({ kind: "error", title: "Load backup failed", message: String(e) });
    }
  }

  async function onRestoreFromBackup() {
    try {
      if (!restoreBackupDir || !restoreManifest) {
        pushToast({ kind: "error", title: "No backup selected", message: "Pick a backup folder first." });
        return;
      }
      if (!restoreAllowOverwrite) {
        pushToast({
          kind: "error",
          title: "Confirmation required",
          message: "Check the overwrite confirmation box before restoring.",
        });
        return;
      }
      const res = await invokeValidated(
        "restore_from_backup",
        { backupDir: restoreBackupDir, allowOverwrite: true },
        RestoreResultSchema
      );
      setRestoreResult(res);
      pushToast({
        kind: "success",
        title: "Restore complete",
        message: `db=${res.restored_db_path} artifacts=${res.restored_artifacts ? "yes" : "no"}`,
      });
    } catch (e) {
      pushToast({ kind: "error", title: "Restore failed", message: String(e) });
    }
  }

  async function onExportSanitizedDataset() {
    try {
      const dest = await pickDirectory();
      if (!dest) return;
      const res = await invokeValidated("export_sanitized_dataset", { destinationDir: dest }, SanitizedExportResultSchema);
      setSanitizedExport(res);
      pushToast({
        kind: "success",
        title: "Sanitized export created",
        message: `${res.export_dir} (incidents=${res.incident_count})`,
      });
    } catch (e) {
      pushToast({ kind: "error", title: "Sanitized export failed", message: String(e) });
    }
  }

  async function onPickSanitizedDatasetForImport() {
    try {
      const dir = await pickDirectory();
      if (!dir) return;
      setSanitizedImportDir(dir);
      setSanitizedImportSummary(null);
      const manifest = await invokeValidated("inspect_sanitized_dataset", { datasetDir: dir }, SanitizedExportManifestSchema);
      setSanitizedImportManifest(manifest);
      pushToast({
        kind: "success",
        title: "Sanitized dataset selected",
        message: `${manifest.export_time} (incidents=${manifest.incident_count})`,
      });
    } catch (e) {
      setSanitizedImportDir("");
      setSanitizedImportManifest(null);
      setSanitizedImportSummary(null);
      pushToast({ kind: "error", title: "Inspect sanitized dataset failed", message: String(e) });
    }
  }

  async function onImportSanitizedDataset() {
    try {
      if (!sanitizedImportDir || !sanitizedImportManifest) {
        pushToast({ kind: "error", title: "No dataset selected", message: "Pick a sanitized dataset folder first." });
        return;
      }

      const res = await invokeValidated(
        "import_sanitized_dataset",
        { datasetDir: sanitizedImportDir },
        SanitizedImportSummarySchema
      );
      setSanitizedImportSummary(res);

      const warnCount = res.import_warnings.length;
      pushToast({
        kind: warnCount > 0 ? "warning" : "success",
        title: "Sanitized import complete",
        message: `incidents=${res.inserted_incidents} events=${res.inserted_timeline_events} warnings=${warnCount}`,
      });

      // Refresh views: UI renders only server-provided datasets.
      await onRefreshIncidentsList();
      await onLoadDashboard();
      await onGenerateReport();
      await onRefreshValidationReport();
    } catch (e) {
      const appErr = extractAppError(e);
      if (appErr) {
        const guidance = guidanceForSanitizedImportErrorCode(appErr.code);
        const details = appErr.details ? `\n\nDetails:\n${typeof appErr.details === "string" ? appErr.details : JSON.stringify(appErr.details)}` : "";
        const msg = `${appErr.code}: ${appErr.message}${details}`;
        pushToast({
          kind: "error",
          title: "Sanitized import failed",
          message: guidance ? `${msg}\n\n${guidance}` : msg,
        });
      } else {
        pushToast({ kind: "error", title: "Sanitized import failed", message: String(e) });
      }
    }
  }

  async function onRefreshIncidentsList() {
    try {
      const res = await invokeValidated("incidents_list", undefined, IncidentListSchema);
      setIncidentOptions(res);
      if (res.length > 0 && slackExistingIncidentId == null) {
        setSlackExistingIncidentId(res[0].id);
      }
      pushToast({ kind: "success", title: "Incidents loaded", message: `${res.length} incidents` });
    } catch (e) {
      pushToast({ kind: "error", title: "Load incidents failed", message: String(e) });
    }
  }

  async function onSlackPickFile(file: File | null) {
    setSlackSummary(null);
    setSlackPreview(null);
    if (!file) return;
    const text = await file.text();
    setSlackFileName(file.name);
    setSlackText(text);
    try {
      const prev = await invokeValidated("slack_preview", { transcriptText: text }, SlackPreviewSchema);
      setSlackPreview(prev);
      pushToast({ kind: "success", title: "Slack loaded", message: `${prev.detected_format} (${prev.message_count} msgs)` });
    } catch (e) {
      pushToast({ kind: "error", title: "Slack preview failed", message: String(e) });
    }
  }

  async function onSlackPreview() {
    try {
      if (!slackText.trim()) {
        pushToast({ kind: "error", title: "No Slack transcript", message: "Choose a file or paste text first." });
        return;
      }
      const prev = await invokeValidated("slack_preview", { transcriptText: slackText }, SlackPreviewSchema);
      setSlackPreview(prev);
      pushToast({ kind: "success", title: "Preview OK", message: `${prev.detected_format} (${prev.message_count} msgs)` });
    } catch (e) {
      pushToast({ kind: "error", title: "Preview failed", message: String(e) });
    }
  }

  async function onSlackIngest() {
    try {
      if (!slackText.trim()) {
        pushToast({ kind: "error", title: "No Slack transcript", message: "Choose a file or paste text first." });
        return;
      }

      if (slackTargetMode === "existing" && slackExistingIncidentId == null) {
        pushToast({ kind: "error", title: "Select an incident", message: "Choose an incident to attach to." });
        return;
      }
      if (slackTargetMode === "new" && slackNewIncidentTitle.trim().length === 0) {
        pushToast({ kind: "error", title: "Title required", message: "Enter a title for the new Slack-only incident." });
        return;
      }

      const args =
        slackTargetMode === "existing"
          ? { incidentId: slackExistingIncidentId, newIncidentTitle: null, transcriptText: slackText }
          : { incidentId: null, newIncidentTitle: slackNewIncidentTitle, transcriptText: slackText };

      const res = await invokeValidated("slack_ingest", args, SlackIngestSummarySchema);
      setSlackSummary(res);
      pushToast({
        kind: res.warnings.length > 0 ? "warning" : "success",
        title: "Slack ingest complete",
        message: `incident_id=${res.incident_id}, events=${res.inserted_events}, warnings=${res.warnings.length}`,
      });
      await onRefreshIncidentsList();
    } catch (e) {
      pushToast({ kind: "error", title: "Slack ingest failed", message: String(e) });
    }
  }

  async function onRefreshValidationReport() {
    try {
      const res = await invokeValidated("validation_report", undefined, ValidationReportSchema);
      setValidationReport(res);
      const withIssues = res.filter((i) => i.warnings.length > 0).length;
      pushToast({
        kind: withIssues > 0 ? "warning" : "success",
        title: "Validation loaded",
        message: `${withIssues}/${res.length} incidents with warnings`,
      });
    } catch (e) {
      pushToast({ kind: "error", title: "Validation failed", message: String(e) });
    }
  }

  function onFilterIncidentFromValidation(incidentId: number, label: string) {
    setIncidentFilterIds([incidentId]);
    setIncidentFilterLabel(label);
    if (!dashboard) {
      pushToast({
        kind: "warning",
        title: "Filter set",
        message: "Load the dashboard to view the incidents table.",
      });
    }
  }

  async function onOpenIncidentDetail(id: number) {
    try {
      setIncidentDetailOpen(true);
      setIncidentDetailLoading(true);
      const res = await invokeValidated("incident_detail", { incidentId: id }, IncidentDetailSchema);
      setIncidentDetail(res);
    } catch (e) {
      pushToast({ kind: "error", title: "Incident detail failed", message: String(e) });
      setIncidentDetailOpen(false);
      setIncidentDetail(null);
    } finally {
      setIncidentDetailLoading(false);
    }
  }

  return (
    <main className="app">
      <ToastHost toasts={toasts} onDismiss={dismissToast} />

      {migrationGuard ? (
        <Modal
          title="Pending Migrations Detected"
          footer={
            <>
              <button
                className="btn"
                type="button"
                onClick={() => {
                  window.location.hash = "#data";
                  setMigrationGuard(null);
                }}
              >
                Backup Now
              </button>
              <button className="btn" type="button" onClick={() => setMigrationGuard(null)}>
                Cancel
              </button>
              <button
                className="btn btn--accent"
                type="button"
                onClick={() => {
                  const g = migrationGuard;
                  setMigrationGuard(null);
                  if (g.action === "init_db") {
                    void ensureDbInitialized({ toastOnSuccess: true, skipPreflight: true });
                  } else {
                    void openWorkspaceAtPath(g.dbPath);
                  }
                }}
              >
                Continue (Migrate)
              </button>
            </>
          }
        >
          <p className="hint">
            This workspace DB has <span className="mono">{migrationGuard.pendingMigrations.length}</span> pending migrations. To
            avoid accidental data loss, back up the workspace before continuing. Migrations are applied deterministically in{" "}
            <code>crates/qir_core</code>.
          </p>
          <p className="hint">
            Workspace: <span className="mono">{migrationGuard.dbPath}</span>
          </p>
          <p className="hint">
            Latest migration: <span className="mono">{migrationGuard.latestMigration}</span>
          </p>
          <div className="subhead">Pending migrations</div>
          <ul className="list">
            {migrationGuard.pendingMigrations.map((m) => (
              <li key={m}>
                <span className="mono">{m}</span>
              </li>
            ))}
          </ul>
        </Modal>
      ) : null}

      <IncidentDetailDrawer
        open={incidentDetailOpen}
        loading={incidentDetailLoading}
        detail={incidentDetail}
        onClose={() => {
          setIncidentDetailOpen(false);
          setIncidentDetail(null);
        }}
      />

      <header className="app__header">
        <div className="app__title">
          <h1>IncidentReview</h1>
          <p className="app__sub">Local-first QIR tooling. Deterministic metrics live in Rust.</p>
        </div>
        <div className="app__meta">
          <div className="pill">
            <span className="pill__label">DB</span>
            <span className="pill__value">{dbPath ?? "not initialized"}</span>
          </div>
          <div className="pill">
            <span className="pill__label">Seed</span>
            <span className="pill__value">{seedInserted == null ? "not run" : `${seedInserted} inserted`}</span>
          </div>
        </div>
      </header>

      <AppNav items={navItems} />

      <section className="card">
        <h2>Actions</h2>
        <div className="actions">
          <button className="btn" onClick={onInitDb} type="button">
            Init DB
          </button>
          <button className="btn" onClick={onSeedDemo} type="button">
            Seed Demo Dataset
          </button>
          <button className="btn btn--accent" onClick={onLoadDashboard} type="button">
            Load Dashboard
          </button>
          <button className="btn" onClick={onGenerateReport} type="button">
            Generate Report (MD)
          </button>
          <button className="btn" onClick={onAiHealthCheck} type="button">
            Check AI (Ollama)
          </button>
        </div>
        <p className="hint">
          This app does not compute metrics in the UI. Dashboards and report data are computed in{" "}
          <code>crates/qir_core</code>.
        </p>
      </section>

      <WorkspaceSection
        currentDbPathLabel={workspaceInfo?.current_db_path ?? dbPath ?? "unknown"}
        workspaceInfo={workspaceInfo}
        workspaceMeta={workspaceMeta}
        workspaceNewFilename={workspaceNewFilename}
        onWorkspaceNewFilenameChange={setWorkspaceNewFilename}
        workspaceRecentPick={workspaceRecentPick}
        onWorkspaceRecentPickChange={setWorkspaceRecentPick}
        onOpenWorkspace={onWorkspaceOpen}
        onCreateWorkspace={onWorkspaceCreate}
        onSwitchToRecent={onWorkspaceSwitchToRecent}
      />

      <BackupRestoreSection
        backupResult={backupResult}
        restoreBackupDir={restoreBackupDir}
        restoreManifest={restoreManifest}
        restoreAllowOverwrite={restoreAllowOverwrite}
        setRestoreAllowOverwrite={setRestoreAllowOverwrite}
        restoreResult={restoreResult}
        onBackupCreate={onBackupCreate}
        onPickBackupForRestore={onPickBackupForRestore}
        onRestoreFromBackup={onRestoreFromBackup}
      />

      <JiraImportSection
        jiraProfiles={jiraProfiles}
        selectedProfileId={selectedProfileId}
        setSelectedProfileId={setSelectedProfileId}
        profileName={profileName}
        setProfileName={setProfileName}
        csvFileName={csvFileName}
        csvPreview={csvPreview}
        mapping={mapping}
        setMapping={setMapping as never}
        importSummary={importSummary}
        onRefreshProfiles={onRefreshProfiles}
        onPickCsvFile={onPickCsvFile}
        applyCommonJiraDefaults={applyCommonJiraDefaults}
        onImportCsv={onImportCsv}
        onSaveProfile={onSaveProfile}
        onDeleteProfile={onDeleteProfile}
      />

      <SlackImportSection
        incidentOptions={incidentOptions}
        slackTargetMode={slackTargetMode}
        setSlackTargetMode={setSlackTargetMode}
        slackExistingIncidentId={slackExistingIncidentId}
        setSlackExistingIncidentId={setSlackExistingIncidentId}
        slackNewIncidentTitle={slackNewIncidentTitle}
        setSlackNewIncidentTitle={setSlackNewIncidentTitle}
        slackFileName={slackFileName}
        slackText={slackText}
        setSlackText={setSlackText}
        setSlackPreview={setSlackPreview}
        setSlackSummary={setSlackSummary}
        slackPreview={slackPreview}
        slackSummary={slackSummary}
        onRefreshIncidentsList={onRefreshIncidentsList}
        onSlackPickFile={onSlackPickFile}
        onSlackPreview={onSlackPreview}
        onSlackIngest={onSlackIngest}
      />

      <SanitizedImportSection
        sanitizedExport={sanitizedExport}
        sanitizedImportDir={sanitizedImportDir}
        sanitizedImportManifest={sanitizedImportManifest}
        sanitizedImportSummary={sanitizedImportSummary}
        onExportSanitizedDataset={onExportSanitizedDataset}
        onPickSanitizedDatasetForImport={onPickSanitizedDatasetForImport}
        onImportSanitizedDataset={onImportSanitizedDataset}
      />

      <ValidationSection
        validationReport={validationReport}
        dashboardLoaded={!!dashboard}
        hasIncidentFilter={!!(incidentFilterIds && incidentFilterIds.length > 0)}
        onRefreshValidation={onRefreshValidationReport}
        onRefreshIncidents={onRefreshIncidentsList}
        onClearIncidentFilter={() => {
          setIncidentFilterIds(null);
          setIncidentFilterLabel("");
        }}
        onFilterIncidentFromValidation={onFilterIncidentFromValidation}
      />

      <DashboardsSection
        dashboard={dashboard}
        selectedSeverity={selectedSeverity}
        setSelectedSeverity={setSelectedSeverity}
        incidentFilterIds={incidentFilterIds}
        incidentFilterLabel={incidentFilterLabel}
        setIncidentFilterIds={setIncidentFilterIds}
        setIncidentFilterLabel={setIncidentFilterLabel}
        onOpenIncidentDetail={onOpenIncidentDetail}
      />

      <ReportSection reportMd={reportMd} />

      <AiSection
        onToast={(t) => {
          pushToast({ kind: t.kind, title: t.title, message: t.message });
        }}
      />

      <AboutSection />
    </main>
  );
}
