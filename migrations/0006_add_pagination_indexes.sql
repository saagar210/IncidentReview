-- Pagination performance indexes for efficient list queries with sorting
-- Supports: pagination on incidents, timeline_events, and evidence chunks

-- Index for paginating incidents sorted by creation/start time
CREATE INDEX IF NOT EXISTS idx_incidents_created_at_desc
  ON incidents(start_ts DESC, id DESC);

-- Index for paginating incidents sorted by title
CREATE INDEX IF NOT EXISTS idx_incidents_title_asc
  ON incidents(title ASC, id ASC);

-- Index for paginating timeline events by incident
CREATE INDEX IF NOT EXISTS idx_timeline_events_incident_ts
  ON timeline_events(incident_id, ts, id);

-- Index for efficient incident lookups by external ID
CREATE INDEX IF NOT EXISTS idx_incidents_external_id
  ON incidents(external_id);
