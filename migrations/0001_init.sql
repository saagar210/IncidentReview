-- 0001_init.sql
-- Initial schema for IncidentReview.
--
-- Notes:
-- - All timestamps are nullable and stored as ISO-8601 text (UTC recommended).
-- - Unknown values are stored as NULL; validators surface anomalies instead of silently defaulting.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS _migrations (
  name TEXT PRIMARY KEY NOT NULL,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS incidents (
  id INTEGER PRIMARY KEY NOT NULL,

  external_id TEXT NULL,
  fingerprint TEXT NOT NULL,

  title TEXT NOT NULL,
  description TEXT NULL,

  severity TEXT NULL,

  impact_pct INTEGER NULL,
  service_health_pct INTEGER NULL,

  start_ts TEXT NULL,
  first_observed_ts TEXT NULL,
  it_awareness_ts TEXT NULL,
  ack_ts TEXT NULL,
  mitigate_ts TEXT NULL,
  resolve_ts TEXT NULL,

  ingested_at TEXT NOT NULL
);

-- Prefer unique external IDs when provided (e.g., Jira key).
CREATE UNIQUE INDEX IF NOT EXISTS incidents_external_id_unique
  ON incidents(external_id)
  WHERE external_id IS NOT NULL;

-- When external_id is missing, enforce stable dedupe via fingerprint.
CREATE UNIQUE INDEX IF NOT EXISTS incidents_fingerprint_unique
  ON incidents(fingerprint);

CREATE TABLE IF NOT EXISTS artifacts (
  id INTEGER PRIMARY KEY NOT NULL,
  incident_id INTEGER NULL,

  kind TEXT NOT NULL,
  sha256 TEXT NOT NULL,

  filename TEXT NULL,
  mime_type TEXT NULL,
  text TEXT NULL,

  created_at TEXT NOT NULL,

  FOREIGN KEY (incident_id) REFERENCES incidents(id) ON DELETE SET NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS artifacts_sha256_unique
  ON artifacts(sha256);

CREATE TABLE IF NOT EXISTS timeline_events (
  id INTEGER PRIMARY KEY NOT NULL,
  incident_id INTEGER NULL,

  source TEXT NOT NULL,
  ts TEXT NULL,
  author TEXT NULL,
  kind TEXT NULL,
  text TEXT NOT NULL,

  raw_json TEXT NULL,

  created_at TEXT NOT NULL,

  FOREIGN KEY (incident_id) REFERENCES incidents(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS jira_mapping_profiles (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  mapping_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);

