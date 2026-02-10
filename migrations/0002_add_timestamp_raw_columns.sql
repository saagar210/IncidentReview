-- 0002_add_timestamp_raw_columns.sql
-- Preserve raw, non-RFC3339 timestamp inputs deterministically while keeping canonical columns RFC3339-only.
--
-- Contract:
-- - Canonical fields (*_ts) remain nullable RFC3339 UTC strings.
-- - Raw inputs are stored in *_ts_raw when the user provided a non-RFC3339 value or an unparseable value.

ALTER TABLE incidents ADD COLUMN start_ts_raw TEXT NULL;
ALTER TABLE incidents ADD COLUMN first_observed_ts_raw TEXT NULL;
ALTER TABLE incidents ADD COLUMN it_awareness_ts_raw TEXT NULL;
ALTER TABLE incidents ADD COLUMN ack_ts_raw TEXT NULL;
ALTER TABLE incidents ADD COLUMN mitigate_ts_raw TEXT NULL;
ALTER TABLE incidents ADD COLUMN resolve_ts_raw TEXT NULL;

