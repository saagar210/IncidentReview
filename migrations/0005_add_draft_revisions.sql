-- Draft revision hierarchy for multi-turn drafting
-- Enables revision workflows: create draft → revise → revise → create alternative

-- Add revision hierarchy columns to ai_drafts table
ALTER TABLE ai_drafts ADD COLUMN parent_draft_id INTEGER NULL
  REFERENCES ai_drafts(id) ON DELETE SET NULL;

ALTER TABLE ai_drafts ADD COLUMN revision_number INTEGER NOT NULL DEFAULT 1;

ALTER TABLE ai_drafts ADD COLUMN revision_notes TEXT NULL;

ALTER TABLE ai_drafts ADD COLUMN branch_label TEXT NULL;

-- Indexes for efficient revision traversal
CREATE INDEX IF NOT EXISTS idx_ai_drafts_parent ON ai_drafts(parent_draft_id);
CREATE INDEX IF NOT EXISTS idx_ai_drafts_revision ON ai_drafts(parent_draft_id, revision_number);

-- Audit trail: track prompts used for each draft revision
CREATE TABLE IF NOT EXISTS ai_draft_prompts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  draft_id INTEGER NOT NULL REFERENCES ai_drafts(id) ON DELETE CASCADE,
  prompt_text TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ai_draft_prompts_draft ON ai_draft_prompts(draft_id);
