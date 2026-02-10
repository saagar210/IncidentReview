import { z } from "zod";

export const AppErrorSchema = z.object({
  code: z.string(),
  message: z.string(),
  details: z.string().nullable().optional(),
  retryable: z.boolean(),
});

export const ValidationWarningSchema = z.object({
  code: z.string(),
  message: z.string(),
  details: z.string().nullable().optional(),
});

export const JiraCsvMappingSchema = z.object({
  external_id: z.string().nullable().optional(),
  title: z.string(),
  description: z.string().nullable().optional(),
  severity: z.string().nullable().optional(),
  detection_source: z.string().nullable().optional(),
  vendor: z.string().nullable().optional(),
  service: z.string().nullable().optional(),
  impact_pct: z.string().nullable().optional(),
  service_health_pct: z.string().nullable().optional(),
  start_ts: z.string().nullable().optional(),
  first_observed_ts: z.string().nullable().optional(),
  it_awareness_ts: z.string().nullable().optional(),
  ack_ts: z.string().nullable().optional(),
  mitigate_ts: z.string().nullable().optional(),
  resolve_ts: z.string().nullable().optional(),
});

export const JiraMappingProfileSchema = z.object({
  id: z.number().int(),
  name: z.string(),
  mapping: JiraCsvMappingSchema,
});

export const JiraMappingProfileListSchema = z.array(JiraMappingProfileSchema);

export const JiraMappingProfileUpsertSchema = z.object({
  id: z.number().int().nullable().optional(),
  name: z.string(),
  mapping: JiraCsvMappingSchema,
});

export const JiraCsvPreviewSchema = z.object({
  headers: z.array(z.string()),
  rows: z.array(z.array(z.string())),
});

export const JiraImportConflictSchema = z.object({
  row: z.number().int(),
  reason: z.string(),
  external_id: z.string().nullable().optional(),
  fingerprint: z.string().nullable().optional(),
});

export const JiraImportSummarySchema = z.object({
  inserted: z.number().int().nonnegative(),
  updated: z.number().int().nonnegative(),
  skipped: z.number().int().nonnegative(),
  conflicts: z.array(JiraImportConflictSchema),
  warnings: z.array(ValidationWarningSchema),
});

export const InitDbResponseSchema = z.object({
  db_path: z.string(),
});

export const JiraIngestSummarySchema = z.object({
  inserted: z.number().int().nonnegative(),
  warnings: z.array(ValidationWarningSchema),
});

export const SeverityCountSchema = z.object({
  severity: z.string(),
  count: z.number().int().nonnegative(),
  incident_ids: z.array(z.number().int()),
});

export const IncidentSummarySchema = z.object({
  id: z.number().int(),
  external_id: z.string().nullable(),
  title: z.string(),
  severity: z.string().nullable(),
  mtta_seconds: z.number().int().nullable(),
  mttr_seconds: z.number().int().nullable(),
  warning_count: z.number().int().nonnegative(),
});

export const DashboardPayloadV1Schema = z.object({
  version: z.number().int(),
  incident_count: z.number().int().nonnegative(),
  severity_counts: z.array(SeverityCountSchema),
  incidents: z.array(IncidentSummarySchema),
});

export const CategoryBucketSchema = z.object({
  key: z.string(),
  label: z.string(),
  count: z.number().int().nonnegative(),
  incident_ids: z.array(z.number().int()),
});

export const DurationBucketSchema = z.object({
  key: z.string(),
  label: z.string(),
  count: z.number().int().nonnegative(),
  incident_ids: z.array(z.number().int()),
});

export const PainBucketSchema = z.object({
  key: z.string(),
  label: z.string(),
  count: z.number().int().nonnegative(),
  pain_sum: z.number().int().nonnegative(),
  pain_known_count: z.number().int().nonnegative(),
  incident_ids: z.array(z.number().int()),
});

export const DetectionStoryV1Schema = z.object({
  detection_source_mix: z.array(CategoryBucketSchema),
  it_awareness_lag_buckets: z.array(DurationBucketSchema),
});

export const VendorServiceStoryV1Schema = z.object({
  top_vendors_by_count: z.array(CategoryBucketSchema),
  top_services_by_count: z.array(CategoryBucketSchema),
  top_vendors_by_pain: z.array(PainBucketSchema),
  top_services_by_pain: z.array(PainBucketSchema),
});

export const ResponseStoryV1Schema = z.object({
  time_to_mitigation_buckets: z.array(DurationBucketSchema),
  time_to_resolve_buckets: z.array(DurationBucketSchema),
});

export const IncidentSummaryV2Schema = z.object({
  id: z.number().int(),
  external_id: z.string().nullable(),
  title: z.string(),
  severity: z.string().nullable(),
  detection_source: z.string().nullable(),
  vendor: z.string().nullable(),
  service: z.string().nullable(),
  it_awareness_lag_seconds: z.number().int().nullable(),
  time_to_mitigation_seconds: z.number().int().nullable(),
  mttr_seconds: z.number().int().nullable(),
  warning_count: z.number().int().nonnegative(),
});

export const DashboardPayloadV2Schema = z.object({
  version: z.number().int(),
  incident_count: z.number().int().nonnegative(),
  severity_counts: z.array(SeverityCountSchema),
  incidents: z.array(IncidentSummaryV2Schema),
  detection_story: DetectionStoryV1Schema,
  vendor_service_story: VendorServiceStoryV1Schema,
  response_story: ResponseStoryV1Schema,
});

export const AiHealthStatusSchema = z.object({
  ok: z.boolean(),
  message: z.string(),
});

export const DeleteResponseSchema = z.object({
  ok: z.boolean(),
});

export const IncidentListItemSchema = z.object({
  id: z.number().int(),
  external_id: z.string().nullable(),
  title: z.string(),
});

export const IncidentListSchema = z.array(IncidentListItemSchema);

export const BackupFileEntrySchema = z.object({
  rel_path: z.string(),
  sha256: z.string(),
  bytes: z.number().int().nonnegative(),
});

export const BackupCountsSchema = z.object({
  incidents: z.number().int().nonnegative(),
  timeline_events: z.number().int().nonnegative(),
  artifacts_rows: z.number().int().nonnegative(),
});

export const BackupDbInfoSchema = z.object({
  filename: z.string(),
  sha256: z.string(),
  bytes: z.number().int().nonnegative(),
});

export const BackupArtifactsInfoSchema = z.object({
  included: z.boolean(),
  files: z.array(BackupFileEntrySchema),
});

export const BackupManifestSchema = z.object({
  manifest_version: z.number().int().nonnegative(),
  app_version: z.string(),
  export_time: z.string(),
  schema_migrations: z.array(z.string()),
  counts: BackupCountsSchema,
  db: BackupDbInfoSchema,
  artifacts: BackupArtifactsInfoSchema,
});

export const BackupCreateResultSchema = z.object({
  backup_dir: z.string(),
  manifest: BackupManifestSchema,
});

export const RestoreResultSchema = z.object({
  ok: z.boolean(),
  restored_db_path: z.string(),
  restored_artifacts: z.boolean(),
});

export const SanitizedExportResultSchema = z.object({
  export_dir: z.string(),
  incident_count: z.number().int().nonnegative(),
});

export const SanitizedFileInfoSchema = z.object({
  filename: z.string(),
  bytes: z.number().int().nonnegative(),
  sha256: z.string(),
});

export const SanitizedExportManifestSchema = z.object({
  manifest_version: z.number().int(),
  app_version: z.string(),
  export_time: z.string(),
  incident_count: z.number().int().nonnegative(),
  files: z.array(SanitizedFileInfoSchema),
});

export const SanitizedImportSummarySchema = z.object({
  inserted_incidents: z.number().int().nonnegative(),
  inserted_timeline_events: z.number().int().nonnegative(),
  import_warnings: z.array(ValidationWarningSchema),
});

export const WorkspaceMetadataSchema = z.object({
  db_path: z.string(),
  is_empty: z.boolean(),
});

export const WorkspaceInfoSchema = z.object({
  current_db_path: z.string(),
  recent_db_paths: z.array(z.string()),
  load_error: AppErrorSchema.nullable().optional(),
});

export const SlackPreviewSchema = z.object({
  detected_format: z.string(),
  line_count: z.number().int().nonnegative(),
  message_count: z.number().int().nonnegative(),
  warnings: z.array(ValidationWarningSchema),
});

export const SlackIngestSummarySchema = z.object({
  incident_id: z.number().int(),
  incident_created: z.boolean(),
  detected_format: z.string(),
  inserted_events: z.number().int().nonnegative(),
  warnings: z.array(ValidationWarningSchema),
});

export const IncidentValidationReportItemSchema = z.object({
  id: z.number().int(),
  external_id: z.string().nullable(),
  title: z.string(),
  warnings: z.array(ValidationWarningSchema),
});

export const ValidationReportSchema = z.array(IncidentValidationReportItemSchema);

export const IncidentSchema = z.object({
  id: z.number().int(),
  external_id: z.string().nullable(),
  fingerprint: z.string(),
  title: z.string(),
  description: z.string().nullable(),
  severity: z.string().nullable(),
  detection_source: z.string().nullable(),
  vendor: z.string().nullable(),
  service: z.string().nullable(),
  impact_pct: z.number().int().nullable(),
  service_health_pct: z.number().int().nullable(),
  start_ts: z.string().nullable(),
  first_observed_ts: z.string().nullable(),
  it_awareness_ts: z.string().nullable(),
  ack_ts: z.string().nullable(),
  mitigate_ts: z.string().nullable(),
  resolve_ts: z.string().nullable(),
  start_ts_raw: z.string().nullable(),
  first_observed_ts_raw: z.string().nullable(),
  it_awareness_ts_raw: z.string().nullable(),
  ack_ts_raw: z.string().nullable(),
  mitigate_ts_raw: z.string().nullable(),
  resolve_ts_raw: z.string().nullable(),
});

export const IncidentMetricsSchema = z.object({
  mttd_seconds: z.number().int().nullable(),
  it_awareness_lag_seconds: z.number().int().nullable(),
  mtta_seconds: z.number().int().nullable(),
  time_to_mitigation_seconds: z.number().int().nullable(),
  mttr_seconds: z.number().int().nullable(),
});

export const ArtifactSchema = z.object({
  id: z.number().int(),
  incident_id: z.number().int().nullable(),
  kind: z.string(),
  sha256: z.string(),
  filename: z.string().nullable(),
  mime_type: z.string().nullable(),
  text: z.string().nullable(),
  created_at: z.string(),
});

export const TimelineEventSchema = z.object({
  id: z.number().int(),
  incident_id: z.number().int().nullable(),
  source: z.string(),
  ts: z.string().nullable(),
  author: z.string().nullable(),
  kind: z.string().nullable(),
  text: z.string(),
  raw_json: z.string().nullable(),
  created_at: z.string(),
});

export const IncidentDetailSchema = z.object({
  incident: IncidentSchema,
  metrics: IncidentMetricsSchema,
  warnings: z.array(ValidationWarningSchema),
  artifacts: z.array(ArtifactSchema),
  timeline_events: z.array(TimelineEventSchema),
});
