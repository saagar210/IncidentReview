import { useEffect, useMemo, useState } from "react";
import ReactECharts from "echarts-for-react";
import { open } from "@tauri-apps/plugin-dialog";

import { extractAppError, invokeValidated } from "./lib/tauri";
import { guidanceForSanitizedImportErrorCode } from "./lib/sanitized_import_guidance";
import { guidanceForWorkspaceErrorCode } from "./lib/workspace_guidance";
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
} from "./lib/schemas";
import { ToastHost } from "./ui/ToastHost";
import { useToasts } from "./ui/useToasts";

function formatSeconds(secs: number | null | undefined): string {
  if (secs == null) return "UNKNOWN";
  const minutes = Math.floor(secs / 60);
  const rem = secs % 60;
  if (minutes >= 60) {
    const hours = Math.floor(minutes / 60);
    const m = minutes % 60;
    return `${hours}h ${m}m`;
  }
  if (minutes > 0) return `${minutes}m ${rem}s`;
  return `${rem}s`;
}

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
      } catch (e) {
        pushToast({ kind: "error", title: "Workspace load failed", message: String(e) });
      }

      // Ensure the current workspace DB is usable. For the default app-data workspace this
      // creates the DB on first run.
      try {
        const res = await invokeValidated("init_db", undefined, InitDbResponseSchema);
        setDbPath(res.db_path);
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

  const filteredIncidents = useMemo(() => {
    if (!dashboard) return [];
    if (incidentFilterIds && incidentFilterIds.length > 0) {
      const set = new Set(incidentFilterIds);
      return dashboard.incidents.filter((i) => set.has(i.id));
    }
    if (!selectedSeverity) return dashboard.incidents;
    return dashboard.incidents.filter((i) => (i.severity ?? "UNKNOWN") === selectedSeverity);
  }, [dashboard, incidentFilterIds, selectedSeverity]);

  const severityChartOption = useMemo(() => {
    if (!dashboard) return null;
    return {
      tooltip: { trigger: "item" },
      series: [
        {
          name: "Severity",
          type: "pie",
          radius: ["45%", "70%"],
          label: { show: true, formatter: "{b}: {c}" },
          data: dashboard.severity_counts.map((s) => ({
            name: s.severity,
            value: s.count,
            incident_ids: s.incident_ids,
            key: `severity:${s.severity}`,
          })),
        },
      ],
    };
  }, [dashboard]);

  function applyIncidentFilter(ids: number[], label: string) {
    const sorted = [...ids].sort((a, b) => a - b);
    setIncidentFilterIds(sorted);
    setIncidentFilterLabel(label);
  }

  const detectionSourceChartOption = useMemo(() => {
    if (!dashboard) return null;
    return {
      tooltip: { trigger: "item" },
      series: [
        {
          name: "Detection Source",
          type: "pie",
          radius: ["35%", "70%"],
          label: { show: true, formatter: "{b}: {c}" },
          data: dashboard.detection_story.detection_source_mix.map((b) => ({
            name: b.label,
            value: b.count,
            incident_ids: b.incident_ids,
            key: b.key,
          })),
        },
      ],
    };
  }, [dashboard]);

  const itAwarenessLagOption = useMemo(() => {
    if (!dashboard) return null;
    const buckets = dashboard.detection_story.it_awareness_lag_buckets;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "IT awareness lag",
          type: "bar",
          data: buckets.map((b) => ({
            value: b.count,
            incident_ids: b.incident_ids,
            key: b.key,
          })),
        },
      ],
    };
  }, [dashboard]);

  const vendorCountOption = useMemo(() => {
    if (!dashboard) return null;
    const buckets = dashboard.vendor_service_story.top_vendors_by_count;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0, rotate: 25 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "Incidents",
          type: "bar",
          data: buckets.map((b) => ({ value: b.count, incident_ids: b.incident_ids, key: b.key })),
        },
      ],
    };
  }, [dashboard]);

  const vendorPainOption = useMemo(() => {
    if (!dashboard) return null;
    const buckets = dashboard.vendor_service_story.top_vendors_by_pain;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0, rotate: 25 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "Pain (impact * degradation * duration)",
          type: "bar",
          data: buckets.map((b) => ({
            value: b.pain_sum,
            incident_ids: b.incident_ids,
            key: b.key,
            pain_known_count: b.pain_known_count,
          })),
        },
      ],
    };
  }, [dashboard]);

  const serviceCountOption = useMemo(() => {
    if (!dashboard) return null;
    const buckets = dashboard.vendor_service_story.top_services_by_count;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0, rotate: 25 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "Incidents",
          type: "bar",
          data: buckets.map((b) => ({ value: b.count, incident_ids: b.incident_ids, key: b.key })),
        },
      ],
    };
  }, [dashboard]);

  const servicePainOption = useMemo(() => {
    if (!dashboard) return null;
    const buckets = dashboard.vendor_service_story.top_services_by_pain;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0, rotate: 25 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "Pain (impact * degradation * duration)",
          type: "bar",
          data: buckets.map((b) => ({
            value: b.pain_sum,
            incident_ids: b.incident_ids,
            key: b.key,
            pain_known_count: b.pain_known_count,
          })),
        },
      ],
    };
  }, [dashboard]);

  const timeToMitigationOption = useMemo(() => {
    if (!dashboard) return null;
    const buckets = dashboard.response_story.time_to_mitigation_buckets;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "Time to mitigate",
          type: "bar",
          data: buckets.map((b) => ({ value: b.count, incident_ids: b.incident_ids, key: b.key })),
        },
      ],
    };
  }, [dashboard]);

  const timeToResolveOption = useMemo(() => {
    if (!dashboard) return null;
    const buckets = dashboard.response_story.time_to_resolve_buckets;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "Time to resolve",
          type: "bar",
          data: buckets.map((b) => ({ value: b.count, incident_ids: b.incident_ids, key: b.key })),
        },
      ],
    };
  }, [dashboard]);

  async function onInitDb() {
    try {
      const res = await invokeValidated("init_db", undefined, InitDbResponseSchema);
      setDbPath(res.db_path);
      pushToast({ kind: "success", title: "DB initialized", message: res.db_path });
    } catch (e) {
      pushToast({ kind: "error", title: "Init failed", message: String(e) });
    }
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

  async function pickDirectory(): Promise<string | null> {
    const res = await open({ directory: true, multiple: false });
    if (!res) return null;
    if (Array.isArray(res)) return res[0] ?? null;
    return res;
  }

  async function pickDbFile(): Promise<string | null> {
    const res = await open({
      directory: false,
      multiple: false,
      filters: [{ name: "SQLite DB", extensions: ["sqlite", "db"] }],
    });
    if (!res) return null;
    if (Array.isArray(res)) return res[0] ?? null;
    return res;
  }

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

  async function onWorkspaceOpen() {
    try {
      const file = await pickDbFile();
      if (!file) return;
      clearWorkspaceScopedState();
      const meta = await invokeValidated("workspace_open", { dbPath: file }, WorkspaceMetadataSchema);
      setWorkspaceMeta(meta);
      setDbPath(meta.db_path);
      const info = await invokeValidated("workspace_get_current", undefined, WorkspaceInfoSchema);
      setWorkspaceInfo(info);
      pushToast({ kind: "success", title: "Workspace opened", message: meta.db_path });
      await refreshAllViewsAfterWorkspaceChange();
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
      clearWorkspaceScopedState();
      const meta = await invokeValidated("workspace_open", { dbPath: workspaceRecentPick }, WorkspaceMetadataSchema);
      setWorkspaceMeta(meta);
      setDbPath(meta.db_path);
      const info = await invokeValidated("workspace_get_current", undefined, WorkspaceInfoSchema);
      setWorkspaceInfo(info);
      pushToast({ kind: "success", title: "Workspace switched", message: meta.db_path });
      await refreshAllViewsAfterWorkspaceChange();
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

      {incidentDetailOpen && (
        <div
          className="drawerOverlay"
          role="dialog"
          aria-modal="true"
          onClick={() => {
            setIncidentDetailOpen(false);
            setIncidentDetail(null);
          }}
        >
          <aside
            className="drawer"
            onClick={(e) => {
              e.stopPropagation();
            }}
          >
            <div className="drawerHeader">
              <div>
                <div className="muted">Incident detail</div>
                <div className="drawerTitle">
                  {incidentDetail?.incident.external_id ?? "NO_EXTERNAL_ID"}:{" "}
                  {incidentDetail?.incident.title ?? (incidentDetailLoading ? "Loading..." : "")}
                </div>
              </div>
              <button
                className="btn"
                type="button"
                onClick={() => {
                  setIncidentDetailOpen(false);
                  setIncidentDetail(null);
                }}
              >
                Close
              </button>
            </div>

            {incidentDetailLoading && <p className="muted">Loading incident detail...</p>}

            {incidentDetail && (
              <div className="drawerBody">
                <section className="drawerSection">
                  <h3>Computed metrics (deterministic)</h3>
                  <ul className="list">
                    <li>
                      <span className="mono">MTTD</span>: {formatSeconds(incidentDetail.metrics.mttd_seconds)}
                    </li>
                    <li>
                      <span className="mono">Awareness lag</span>:{" "}
                      {formatSeconds(incidentDetail.metrics.it_awareness_lag_seconds)}
                    </li>
                    <li>
                      <span className="mono">MTTA</span>: {formatSeconds(incidentDetail.metrics.mtta_seconds)}
                    </li>
                    <li>
                      <span className="mono">Time to mitigate</span>:{" "}
                      {formatSeconds(incidentDetail.metrics.time_to_mitigation_seconds)}
                    </li>
                    <li>
                      <span className="mono">MTTR</span>: {formatSeconds(incidentDetail.metrics.mttr_seconds)}
                    </li>
                  </ul>
                </section>

                <section className="drawerSection">
                  <h3>Validation/anomalies</h3>
                  {incidentDetail.warnings.length === 0 ? (
                    <p className="muted">No warnings.</p>
                  ) : (
                    <ul className="list">
                      {incidentDetail.warnings.map((w, idx) => (
                        <li key={idx}>
                          <span className="mono">{w.code}</span>: {w.message}{" "}
                          {w.details ? <span className="mono">({w.details})</span> : null}
                        </li>
                      ))}
                    </ul>
                  )}
                </section>

                <section className="drawerSection">
                  <h3>Timeline events</h3>
                  {incidentDetail.timeline_events.length === 0 ? (
                    <p className="muted">No timeline events attached.</p>
                  ) : (
                    <ul className="list">
                      {incidentDetail.timeline_events.map((e) => (
                        <li key={e.id}>
                          {e.text === "[REDACTED]" || (e.raw_json && e.raw_json.includes('"text_redacted":true')) ? (
                            <span className="muted">(redacted)</span>
                          ) : null}{" "}
                          <span className="mono">{e.ts ?? "UNKNOWN_TS"}</span>{" "}
                          <span className="muted">({e.source})</span>: {e.text}
                        </li>
                      ))}
                    </ul>
                  )}
                </section>

                <section className="drawerSection">
                  <h3>Artifacts</h3>
                  {incidentDetail.artifacts.length === 0 ? (
                    <p className="muted">No artifacts attached.</p>
                  ) : (
                    <ul className="list">
                      {incidentDetail.artifacts.map((a) => (
                        <li key={a.id}>
                          <span className="mono">{a.kind}</span>{" "}
                          {a.filename ? <span className="mono">({a.filename})</span> : null}{" "}
                          <span className="muted">{a.sha256.slice(0, 12)}...</span>
                        </li>
                      ))}
                    </ul>
                  )}
                </section>
              </div>
            )}
          </aside>
        </div>
      )}

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

      <nav className="card">
        <h2>Jump To</h2>
        <div className="actions">
          <a className="btn" href="#workspace">
            Workspace
          </a>
          <a className="btn" href="#jira">
            Jira Import
          </a>
          <a className="btn" href="#slack">
            Slack Import
          </a>
          <a className="btn" href="#validation">
            Validation
          </a>
          <a className="btn btn--accent" href="#dashboards">
            Dashboards
          </a>
          <a className="btn" href="#data">
            Backup/Restore
          </a>
          <a className="btn" href="#report">
            Report
          </a>
        </div>
        <p className="hint">Dashboards and reports are computed in Rust (analytics payloads only) and rendered by the UI.</p>
      </nav>

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

      <section className="card" id="workspace">
        <h2>Workspace (Create / Open / Switch)</h2>
        <p className="hint">
          A workspace is a local SQLite DB file. Switching workspaces reloads all data (incidents, dashboards, validation, report) from the selected DB.
        </p>
        <p className="hint">
          Current: <span className="mono">{workspaceInfo?.current_db_path ?? dbPath ?? "unknown"}</span>
        </p>
        {workspaceMeta ? (
          <p className="hint">
            Status: <span className="mono">{workspaceMeta.is_empty ? "empty" : "non-empty"}</span>
          </p>
        ) : null}

        <div className="actions">
          <button className="btn" type="button" onClick={onWorkspaceOpen}>
            Open Workspace DB...
          </button>
          <input
            className="textInput"
            value={workspaceNewFilename}
            onChange={(e) => setWorkspaceNewFilename(e.currentTarget.value)}
            placeholder="New DB filename (e.g. incidentreview.sqlite)"
          />
          <button className="btn btn--accent" type="button" onClick={onWorkspaceCreate}>
            Create New Workspace...
          </button>
        </div>

        <div className="actions">
          <select className="select" value={workspaceRecentPick} onChange={(e) => setWorkspaceRecentPick(e.currentTarget.value)}>
            <option value="">(no recent workspaces)</option>
            {(workspaceInfo?.recent_db_paths ?? []).map((p) => (
              <option key={p} value={p}>
                {p}
              </option>
            ))}
          </select>
          <button className="btn" type="button" onClick={onWorkspaceSwitchToRecent} disabled={!workspaceRecentPick}>
            Switch To Selected
          </button>
        </div>
      </section>

      <section className="card" id="data">
        <h2>Backup / Restore (Local-Only)</h2>
        <div className="actions">
          <button className="btn" type="button" onClick={onBackupCreate}>
            Create Backup Folder...
          </button>
          <button className="btn" type="button" onClick={onPickBackupForRestore}>
            Pick Backup For Restore...
          </button>
          <button className="btn btn--accent" type="button" onClick={onRestoreFromBackup} disabled={!restoreManifest}>
            Restore (Overwrite)
          </button>
          <button className="btn" type="button" onClick={onExportSanitizedDataset}>
            Export Sanitized Dataset...
          </button>
          <button className="btn" type="button" onClick={onPickSanitizedDatasetForImport}>
            Pick Sanitized Dataset...
          </button>
          <button
            className="btn btn--accent"
            type="button"
            onClick={onImportSanitizedDataset}
            disabled={!sanitizedImportManifest}
          >
            Import Sanitized Dataset
          </button>
        </div>
        <p className="hint">
          Backups are exported as folders containing <span className="mono">incidentreview.sqlite</span> and{" "}
          <span className="mono">manifest.json</span> (no zip by default). Restore requires explicit overwrite
          confirmation and validates DB hashes from the manifest.
        </p>
        <p className="hint">
          Sanitized import is deterministic and refuses to run on a non-empty DB. If you already have local data, restore
          a fresh DB first.
        </p>

        {sanitizedExport && (
          <section className="card">
            <h2>Last Sanitized Export</h2>
            <p className="hint">
              Folder: <span className="mono">{sanitizedExport.export_dir}</span>
            </p>
            <p className="hint">
              Incidents: <span className="mono">{sanitizedExport.incident_count}</span>
            </p>
            <p className="hint">
              Free-text fields (Slack text, notes) are redacted; categories are pseudonymized deterministically for sharing.
            </p>
          </section>
        )}

        {sanitizedImportManifest && (
          <section className="card">
            <h2>Sanitized Import Preview</h2>
            <p className="hint">
              Selected: <span className="mono">{sanitizedImportDir}</span>
            </p>
            <ul className="list">
              <li>
                Export time: <span className="mono">{sanitizedImportManifest.export_time}</span>
              </li>
              <li>
                App version: <span className="mono">{sanitizedImportManifest.app_version}</span>
              </li>
              <li>
                Incidents: <span className="mono">{sanitizedImportManifest.incident_count}</span>
              </li>
              <li>
                Files:{" "}
                <span className="mono">
                  {sanitizedImportManifest.files.map((f) => f.filename).sort().join(", ")}
                </span>
              </li>
            </ul>
            <p className="hint">
              On import, incident titles become <span className="mono">Incident INC_###</span> and timeline text becomes{" "}
              <span className="mono">[REDACTED]</span>.
            </p>
          </section>
        )}

        {sanitizedImportSummary && (
          <section className="card">
            <h2>Sanitized Import Result</h2>
            <ul className="list">
              <li>
                Inserted incidents: <span className="mono">{sanitizedImportSummary.inserted_incidents}</span>
              </li>
              <li>
                Inserted events: <span className="mono">{sanitizedImportSummary.inserted_timeline_events}</span>
              </li>
              <li>
                Import warnings: <span className="mono">{sanitizedImportSummary.import_warnings.length}</span>
              </li>
            </ul>
            {sanitizedImportSummary.import_warnings.length > 0 ? (
              <>
                <h3>Warnings</h3>
                <ul className="list">
                  {sanitizedImportSummary.import_warnings.map((w, idx) => (
                    <li key={idx}>
                      <span className="mono">{w.code}</span>: {w.message}{" "}
                      {w.details ? <span className="mono">({w.details})</span> : null}
                    </li>
                  ))}
                </ul>
              </>
            ) : (
              <p className="muted">No import warnings.</p>
            )}
          </section>
        )}

        {backupResult && (
          <section className="card">
            <h2>Last Backup</h2>
            <p className="hint">
              Folder: <span className="mono">{backupResult.backup_dir}</span>
            </p>
            <div className="kpiRow">
              <div className="kpi">
                <div className="kpi__label">Incidents</div>
                <div className="kpi__value">{backupResult.manifest.counts.incidents}</div>
              </div>
              <div className="kpi">
                <div className="kpi__label">Timeline events</div>
                <div className="kpi__value">{backupResult.manifest.counts.timeline_events}</div>
              </div>
              <div className="kpi">
                <div className="kpi__label">Artifacts rows</div>
                <div className="kpi__value">{backupResult.manifest.counts.artifacts_rows}</div>
              </div>
            </div>
          </section>
        )}

        {restoreManifest && (
          <section className="card">
            <h2>Restore Preview</h2>
            <p className="hint">
              Selected: <span className="mono">{restoreBackupDir}</span>
            </p>
            <ul className="list">
              <li>
                Export time: <span className="mono">{restoreManifest.export_time}</span>
              </li>
              <li>
                App version: <span className="mono">{restoreManifest.app_version}</span>
              </li>
              <li>
                Incidents: <span className="mono">{restoreManifest.counts.incidents}</span>
              </li>
              <li>
                Timeline events: <span className="mono">{restoreManifest.counts.timeline_events}</span>
              </li>
              <li>
                Artifacts included: <span className="mono">{restoreManifest.artifacts.included ? "yes" : "no"}</span>
              </li>
            </ul>

            <label className="checkbox">
              <input
                type="checkbox"
                checked={restoreAllowOverwrite}
                onChange={(e) => setRestoreAllowOverwrite(e.currentTarget.checked)}
              />
              I understand this will overwrite my local database.
            </label>

            {restoreResult && (
              <p className="hint">
                Restore result: <span className="mono">{restoreResult.ok ? "ok" : "failed"}</span> (db:{" "}
                <span className="mono">{restoreResult.restored_db_path}</span>)
              </p>
            )}
          </section>
        )}
      </section>

      <section className="card" id="jira">
        <h2>Jira CSV Import (Mapping Profiles)</h2>
        <div className="actions">
          <button className="btn" type="button" onClick={onRefreshProfiles}>
            Refresh profiles
          </button>
          <label className="btn">
            Choose CSV
            <input
              className="fileInput"
              type="file"
              accept=".csv,text/csv"
              onChange={(e) => void onPickCsvFile(e.currentTarget.files?.[0] ?? null)}
            />
          </label>
          <button className="btn" type="button" onClick={applyCommonJiraDefaults} disabled={!csvPreview}>
            Apply common Jira defaults
          </button>
          <button className="btn btn--accent" type="button" onClick={onImportCsv}>
            Import CSV
          </button>
        </div>
        <p className="hint">
          Selected file: <span className="mono">{csvFileName || "none"}</span>
        </p>

        <div className="twoCol">
          <section className="card">
            <h2>Profiles</h2>
            <div className="actions">
              <select
                className="select"
                value={selectedProfileId ?? ""}
                onChange={(e) => {
                  const v = e.currentTarget.value;
                  const id = v ? Number(v) : null;
                  setSelectedProfileId(id);
                  const prof = jiraProfiles.find((p) => p.id === id);
                  if (prof) {
                    setProfileName(prof.name);
                    setMapping({
                      external_id: prof.mapping.external_id ?? null,
                      title: prof.mapping.title ?? "",
                      description: prof.mapping.description ?? null,
                      severity: prof.mapping.severity ?? null,
                      detection_source: prof.mapping.detection_source ?? null,
                      vendor: prof.mapping.vendor ?? null,
                      service: prof.mapping.service ?? null,
                      impact_pct: prof.mapping.impact_pct ?? null,
                      service_health_pct: prof.mapping.service_health_pct ?? null,
                      start_ts: prof.mapping.start_ts ?? null,
                      first_observed_ts: prof.mapping.first_observed_ts ?? null,
                      it_awareness_ts: prof.mapping.it_awareness_ts ?? null,
                      ack_ts: prof.mapping.ack_ts ?? null,
                      mitigate_ts: prof.mapping.mitigate_ts ?? null,
                      resolve_ts: prof.mapping.resolve_ts ?? null,
                    });
                  }
                }}
              >
                <option value="">(no profile selected)</option>
                {jiraProfiles.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} (id={p.id})
                  </option>
                ))}
              </select>
              <input
                className="textInput"
                placeholder="Profile name"
                value={profileName}
                onChange={(e) => setProfileName(e.currentTarget.value)}
              />
              <button className="btn" type="button" onClick={onSaveProfile}>
                Save profile
              </button>
              <button className="btn" type="button" onClick={onDeleteProfile} disabled={!selectedProfileId}>
                Delete
              </button>
            </div>
            <p className="hint">Profiles are stored locally in SQLite via qir_core.</p>
          </section>

          <section className="card">
            <h2>CSV Preview</h2>
            {!csvPreview ? (
              <p className="muted">Choose a CSV to preview headers and sample rows.</p>
            ) : (
              <>
                <p className="hint">
                  Columns: <span className="mono">{csvPreview.headers.join(", ")}</span>
                </p>
                <div className="tableWrap">
                  <table className="table">
                    <thead>
                      <tr>
                        {csvPreview.headers.map((h) => (
                          <th key={h}>{h}</th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {csvPreview.rows.map((r, idx) => (
                        <tr key={idx}>
                          {r.map((c, i) => (
                            <td key={i} className="mono">
                              {c}
                            </td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </>
            )}
          </section>
        </div>

        <section className="card">
          <h2>Mapping</h2>
          {!csvPreview ? (
            <p className="muted">Load a CSV first.</p>
          ) : (
            <div className="mapping">
              {(
                [
                  { key: "external_id", label: "External ID (optional)", required: false },
                  { key: "title", label: "Title (required)", required: true },
                  { key: "description", label: "Description (optional)", required: false },
                  { key: "severity", label: "Severity (optional)", required: false },
                  { key: "detection_source", label: "Detection Source (optional)", required: false },
                  { key: "vendor", label: "Vendor (optional)", required: false },
                  { key: "service", label: "Service (optional)", required: false },
                  { key: "impact_pct", label: "Impact % (optional)", required: false },
                  { key: "service_health_pct", label: "Service Health % (optional)", required: false },
                  { key: "start_ts", label: "Start TS (optional)", required: false },
                  { key: "first_observed_ts", label: "First Observed TS (optional)", required: false },
                  { key: "it_awareness_ts", label: "IT Awareness TS (optional)", required: false },
                  { key: "ack_ts", label: "Ack TS (optional)", required: false },
                  { key: "mitigate_ts", label: "Mitigate TS (optional)", required: false },
                  { key: "resolve_ts", label: "Resolve TS (optional)", required: false },
                ] as const
              ).map((f) => (
                <div key={f.key} className="mappingRow">
                  <div className="mappingRow__label">{f.label}</div>
                  <select
                    className="select"
                    value={(mapping as never)[f.key] ?? (f.required ? "" : "")}
                    onChange={(e) => {
                      const v = e.currentTarget.value;
                      setMapping((m) => ({
                        ...m,
                        [f.key]: v === "" ? (f.required ? "" : null) : v,
                      }) as never);
                    }}
                  >
                    <option value="">{f.required ? "(select a column)" : "(none)"}</option>
                    {csvPreview.headers.map((h) => (
                      <option key={h} value={h}>
                        {h}
                      </option>
                    ))}
                  </select>
                </div>
              ))}
            </div>
          )}
          <p className="hint">
            Timestamp normalization is deterministic: canonical incident timestamps are stored as RFC3339 UTC; non-RFC3339
            inputs are preserved as raw strings and surfaced as warnings (no fuzzy parsing, no guessing).
          </p>
        </section>

        {importSummary && (
          <section className="card">
            <h2>Import Result</h2>
            <div className="kpiRow">
              <div className="kpi">
                <div className="kpi__label">Inserted</div>
                <div className="kpi__value">{importSummary.inserted}</div>
              </div>
              <div className="kpi">
                <div className="kpi__label">Updated</div>
                <div className="kpi__value">{importSummary.updated}</div>
              </div>
              <div className="kpi">
                <div className="kpi__label">Skipped</div>
                <div className="kpi__value">{importSummary.skipped}</div>
              </div>
              <div className="kpi">
                <div className="kpi__label">Conflicts</div>
                <div className="kpi__value">{importSummary.conflicts.length}</div>
              </div>
              <div className="kpi">
                <div className="kpi__label">Warnings</div>
                <div className="kpi__value">{importSummary.warnings.length}</div>
              </div>
            </div>

            {importSummary.conflicts.length > 0 && (
              <>
                <h3 className="subhead">Conflicts</h3>
                <ul className="list">
                  {importSummary.conflicts.map((c, idx) => (
                    <li key={idx} className="mono">
                      row={c.row}: {c.reason}
                    </li>
                  ))}
                </ul>
              </>
            )}

            {importSummary.warnings.length > 0 && (
              <>
                <h3 className="subhead">Warnings</h3>
                <ul className="list">
                  {importSummary.warnings.map((w, idx) => (
                    <li key={idx}>
                      <span className="mono">{w.code}</span>: {w.message}{" "}
                      {w.details ? <span className="mono">({w.details})</span> : null}
                    </li>
                  ))}
                </ul>
              </>
            )}
          </section>
        )}
      </section>

      <section className="card" id="slack">
        <h2>Slack Import (Transcript)</h2>
        <div className="actions">
          <button className="btn" type="button" onClick={onRefreshIncidentsList}>
            Refresh incidents
          </button>
          <label className="btn">
            Choose Slack file
            <input
              className="fileInput"
              type="file"
              accept=".txt,.json,application/json,text/plain"
              onChange={(e) => void onSlackPickFile(e.currentTarget.files?.[0] ?? null)}
            />
          </label>
          <button className="btn" type="button" onClick={onSlackPreview}>
            Preview
          </button>
          <button className="btn btn--accent" type="button" onClick={onSlackIngest}>
            Ingest
          </button>
        </div>

        <p className="hint">
          Selected file: <span className="mono">{slackFileName || "none"}</span>
        </p>

        <div className="twoCol">
          <section className="card">
            <h2>Attach To</h2>
            <div className="actions">
              <label className="radio">
                <input
                  type="radio"
                  name="slackTarget"
                  checked={slackTargetMode === "existing"}
                  onChange={() => setSlackTargetMode("existing")}
                />
                Existing incident
              </label>
              <label className="radio">
                <input
                  type="radio"
                  name="slackTarget"
                  checked={slackTargetMode === "new"}
                  onChange={() => setSlackTargetMode("new")}
                />
                New Slack-only incident shell
              </label>
            </div>

            {slackTargetMode === "existing" ? (
              <>
                <p className="hint">Choose an incident to attach events to.</p>
                <select
                  className="select"
                  value={slackExistingIncidentId ?? ""}
                  onChange={(e) => setSlackExistingIncidentId(e.currentTarget.value ? Number(e.currentTarget.value) : null)}
                >
                  <option value="">(select an incident)</option>
                  {incidentOptions.map((i) => (
                    <option key={i.id} value={i.id}>
                      {(i.external_id ?? `id=${i.id}`) + " — " + i.title}
                    </option>
                  ))}
                </select>
              </>
            ) : (
              <>
                <p className="hint">Title is required (no silent defaults).</p>
                <input
                  className="textInput"
                  placeholder="New incident title"
                  value={slackNewIncidentTitle}
                  onChange={(e) => setSlackNewIncidentTitle(e.currentTarget.value)}
                />
              </>
            )}
          </section>

          <section className="card">
            <h2>Transcript</h2>
            <textarea
              className="md"
              value={slackText}
              placeholder="Paste transcript text here, or choose a file above."
              onChange={(e) => {
                setSlackText(e.currentTarget.value);
                setSlackPreview(null);
                setSlackSummary(null);
              }}
            />
            {slackPreview ? (
              <>
                <h3 className="subhead">Preview</h3>
                <p className="hint">
                  detected_format=<span className="mono">{slackPreview.detected_format}</span>, lines=
                  <span className="mono">{slackPreview.line_count}</span>, messages=
                  <span className="mono">{slackPreview.message_count}</span>
                </p>
                {slackPreview.warnings.length > 0 && (
                  <ul className="list">
                    {slackPreview.warnings.map((w, idx) => (
                      <li key={idx}>
                        <span className="mono">{w.code}</span>: {w.message}{" "}
                        {w.details ? <span className="mono">({w.details})</span> : null}
                      </li>
                    ))}
                  </ul>
                )}
              </>
            ) : (
              <p className="muted">Preview shows detected format and warnings (no timestamp guessing).</p>
            )}
          </section>
        </div>

        {slackSummary && (
          <section className="card">
            <h2>Slack Ingest Result</h2>
            <div className="kpiRow">
              <div className="kpi">
                <div className="kpi__label">Incident ID</div>
                <div className="kpi__value mono">{slackSummary.incident_id}</div>
              </div>
              <div className="kpi">
                <div className="kpi__label">Created</div>
                <div className="kpi__value">{slackSummary.incident_created ? "yes" : "no"}</div>
              </div>
              <div className="kpi">
                <div className="kpi__label">Format</div>
                <div className="kpi__value mono">{slackSummary.detected_format}</div>
              </div>
              <div className="kpi">
                <div className="kpi__label">Events</div>
                <div className="kpi__value">{slackSummary.inserted_events}</div>
              </div>
              <div className="kpi">
                <div className="kpi__label">Warnings</div>
                <div className="kpi__value">{slackSummary.warnings.length}</div>
              </div>
            </div>

            {slackSummary.warnings.length > 0 && (
              <>
                <h3 className="subhead">Warnings</h3>
                <ul className="list">
                  {slackSummary.warnings.map((w, idx) => (
                    <li key={idx}>
                      <span className="mono">{w.code}</span>: {w.message}{" "}
                      {w.details ? <span className="mono">({w.details})</span> : null}
                    </li>
                  ))}
                </ul>
              </>
            )}
          </section>
        )}
      </section>

      <section className="card" id="validation">
        <h2>Validation / Anomalies</h2>
        <div className="actions">
          <button className="btn" type="button" onClick={onRefreshValidationReport}>
            Refresh validation
          </button>
          <button className="btn" type="button" onClick={onRefreshIncidentsList}>
            Refresh incidents
          </button>
          <button
            className="btn"
            type="button"
            onClick={() => {
              setIncidentFilterIds(null);
              setIncidentFilterLabel("");
            }}
            disabled={!incidentFilterIds || incidentFilterIds.length === 0}
          >
            Clear incident filter
          </button>
        </div>

        {!validationReport ? (
          <p className="muted">Load validation to see incidents with warnings/errors.</p>
        ) : (
          <>
            <p className="hint">
              Showing incidents with warnings. Validators run in <code>crates/qir_core</code>; the UI only renders the
              payload.
            </p>
            <ul className="list">
              {validationReport
                .filter((i) => i.warnings.length > 0)
                .map((i) => (
                  <li key={i.id}>
                    <div className="actions">
                      <span className="mono">{i.external_id ?? `id=${i.id}`}</span>
                      <span>{i.title}</span>
                      <span className="mono">warnings={i.warnings.length}</span>
                      <button
                        className="linkBtn"
                        type="button"
                        onClick={() => {
                          setIncidentFilterIds([i.id]);
                          setIncidentFilterLabel(`validation:${i.external_id ?? `id=${i.id}`}`);
                          if (!dashboard) {
                            pushToast({
                              kind: "warning",
                              title: "Filter set",
                              message: "Load the dashboard to view the incidents table.",
                            });
                          }
                        }}
                      >
                        Filter incidents table
                      </button>
                    </div>
                    <ul className="list">
                      {i.warnings.map((w, idx) => (
                        <li key={idx}>
                          <span className="mono">{w.code}</span>: {w.message}{" "}
                          {w.details ? <span className="mono">({w.details})</span> : null}
                        </li>
                      ))}
                    </ul>
                  </li>
                ))}
            </ul>
            {validationReport.filter((i) => i.warnings.length > 0).length === 0 && (
              <p className="muted">No validation warnings found.</p>
            )}
          </>
        )}
      </section>

      <section className="grid" id="dashboards">
        <section className="card">
          <h2>Quarter At A Glance</h2>
          {!dashboard ? (
            <p className="muted">Load the dashboard to view severity distribution and incidents.</p>
          ) : (
            <>
              <div className="kpiRow">
                <div className="kpi">
                  <div className="kpi__label">Incident Count</div>
                  <div className="kpi__value">{dashboard.incident_count}</div>
                </div>
                <div className="kpi">
                  <div className="kpi__label">Selected Severity</div>
                  <div className="kpi__value">{selectedSeverity ?? "ALL"}</div>
                </div>
                <div className="kpi">
                  <div className="kpi__label">Incident Filter</div>
                  <div className="kpi__value">
                    {!incidentFilterIds || incidentFilterIds.length === 0
                      ? "NONE"
                      : `${incidentFilterIds.length} selected`}
                  </div>
                </div>
              </div>

              {severityChartOption && (
                <div className="chart">
                  <ReactECharts
                    option={severityChartOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { name?: string; data?: { incident_ids?: number[]; key?: string } }) => {
                        setSelectedSeverity(params?.name ?? null);
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "severity";
                        if (ids && ids.length > 0) {
                          applyIncidentFilter(ids, key);
                        }
                      },
                    }}
                  />
                  <div className="chart__footer">
                    <button className="linkBtn" type="button" onClick={() => setSelectedSeverity(null)}>
                      Clear filter
                    </button>
                    <button
                      className="linkBtn"
                      type="button"
                      onClick={() => {
                        setIncidentFilterIds(null);
                        setIncidentFilterLabel("");
                      }}
                      disabled={!incidentFilterIds || incidentFilterIds.length === 0}
                    >
                      Clear incident filter
                    </button>
                  </div>
                </div>
              )}
            </>
          )}
        </section>

        <section className="card">
          <h2>Incidents (Drill-down)</h2>
          {!dashboard ? (
            <p className="muted">Load the dashboard first.</p>
          ) : (
            <>
              {incidentFilterIds && incidentFilterIds.length > 0 && (
                <p className="hint">
                  Filtering to <span className="mono">{incidentFilterLabel || `${incidentFilterIds.length} incidents`}</span>.{" "}
                  <button
                    className="linkBtn"
                    type="button"
                    onClick={() => {
                      setIncidentFilterIds(null);
                      setIncidentFilterLabel("");
                    }}
                  >
                    Clear
                  </button>
                </p>
              )}
              <div className="tableWrap">
                <table className="table">
                  <thead>
                    <tr>
                      <th>External ID</th>
                      <th>Title</th>
                      <th>Severity</th>
                      <th>Awareness lag</th>
                      <th>Time to mitigate</th>
                      <th>MTTR</th>
                      <th>Warnings</th>
                    </tr>
                  </thead>
                  <tbody>
                    {filteredIncidents.map((i) => (
                      <tr key={i.id}>
                        <td className="mono">{i.external_id ?? "NO_EXTERNAL_ID"}</td>
                        <td>
                          <button className="linkBtn" type="button" onClick={() => onOpenIncidentDetail(i.id)}>
                            {i.title}
                          </button>
                        </td>
                        <td>{i.severity ?? "UNKNOWN"}</td>
                        <td className="mono">{formatSeconds(i.it_awareness_lag_seconds)}</td>
                        <td className="mono">{formatSeconds(i.time_to_mitigation_seconds)}</td>
                        <td className="mono">{formatSeconds(i.mttr_seconds)}</td>
                        <td className="mono">{i.warning_count}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <p className="hint">
                Reconciliation check: the severity counts sum to the incident total (enforced in{" "}
                <code>crates/qir_core</code> tests).
              </p>
            </>
          )}
        </section>
      </section>

      <section className="grid">
        <section className="card">
          <h2>Detection Story</h2>
          {!dashboard ? (
            <p className="muted">Load the dashboard first.</p>
          ) : (
            <>
              <p className="hint">All charts reconcile to total incidents via explicit UNKNOWN buckets.</p>
              <div className="chart">
                <h3 className="subhead">Detection Source Mix</h3>
                {detectionSourceChartOption && (
                  <ReactECharts
                    option={detectionSourceChartOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "detection_source";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
              <div className="chart">
                <h3 className="subhead">IT Awareness Lag Distribution</h3>
                {itAwarenessLagOption && (
                  <ReactECharts
                    option={itAwarenessLagOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "it_awareness_lag";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
            </>
          )}
        </section>

        <section className="card">
          <h2>Response Story</h2>
          {!dashboard ? (
            <p className="muted">Load the dashboard first.</p>
          ) : (
            <>
              <div className="chart">
                <h3 className="subhead">Time To Mitigate</h3>
                {timeToMitigationOption && (
                  <ReactECharts
                    option={timeToMitigationOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "time_to_mitigation";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
              <div className="chart">
                <h3 className="subhead">Time To Resolve</h3>
                {timeToResolveOption && (
                  <ReactECharts
                    option={timeToResolveOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "time_to_resolve";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
            </>
          )}
        </section>
      </section>

      <section className="grid">
        <section className="card">
          <h2>Vendor/Service Reliability</h2>
          {!dashboard ? (
            <p className="muted">Load the dashboard first.</p>
          ) : (
            <>
              <div className="chart">
                <h3 className="subhead">Top Vendors By Incident Count</h3>
                {vendorCountOption && (
                  <ReactECharts
                    option={vendorCountOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "vendor_count";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
              <div className="chart">
                <h3 className="subhead">Top Vendors By Weighted Pain</h3>
                {vendorPainOption && (
                  <ReactECharts
                    option={vendorPainOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "vendor_pain";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
                <p className="hint">
                  Pain is computed deterministically as <span className="mono">impact * degradation * duration_seconds</span>{" "}
                  when inputs are present; otherwise it contributes 0 to pain_sum but incidents still drill down.
                </p>
              </div>
            </>
          )}
        </section>

        <section className="card">
          <h2>Service Reliability</h2>
          {!dashboard ? (
            <p className="muted">Load the dashboard first.</p>
          ) : (
            <>
              <div className="chart">
                <h3 className="subhead">Top Services By Incident Count</h3>
                {serviceCountOption && (
                  <ReactECharts
                    option={serviceCountOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "service_count";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
              <div className="chart">
                <h3 className="subhead">Top Services By Weighted Pain</h3>
                {servicePainOption && (
                  <ReactECharts
                    option={servicePainOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "service_pain";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
            </>
          )}
        </section>
      </section>

      <section className="card" id="report">
        <h2>QIR Report (Markdown)</h2>
        <textarea className="md" value={reportMd} readOnly placeholder="Generate the report to view Markdown output." />
      </section>
    </main>
  );
}
