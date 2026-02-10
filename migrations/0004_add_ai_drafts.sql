-- AI draft artifacts are stored locally for audit/provenance only.
-- They never overwrite deterministic report/metrics truth.

CREATE TABLE IF NOT EXISTS ai_drafts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  quarter_label TEXT NOT NULL,
  section_type TEXT NOT NULL,
  draft_text TEXT NOT NULL,
  citation_chunk_ids_json TEXT NOT NULL,
  model_name TEXT NOT NULL,
  model_params_hash TEXT NOT NULL,
  prompt_template_version TEXT NOT NULL,
  created_at TEXT NOT NULL,
  artifact_hash TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_ai_drafts_quarter_label ON ai_drafts(quarter_label);
CREATE INDEX IF NOT EXISTS idx_ai_drafts_section_type ON ai_drafts(section_type);
