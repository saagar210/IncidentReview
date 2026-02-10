-- 0003_add_story_fields.sql
-- Add optional fields used for storytelling dashboards and deterministic reporting.
--
-- Contract:
-- - These fields are optional (nullable).
-- - Unknown values remain NULL; analytics buckets may surface UNKNOWN explicitly.

ALTER TABLE incidents ADD COLUMN detection_source TEXT NULL;
ALTER TABLE incidents ADD COLUMN vendor TEXT NULL;
ALTER TABLE incidents ADD COLUMN service TEXT NULL;

