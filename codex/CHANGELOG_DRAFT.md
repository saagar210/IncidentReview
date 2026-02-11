# Changelog Draft

## Theme: Frontend bundle hardening
- Added deterministic manual chunking to `vite.config.ts` so ECharts is emitted as a dedicated vendor chunk and the primary app bundle is reduced from ~1.5 MB to ~97 KB in production output.
- Preserved existing runtime behavior (no route/lifecycle logic changed); this is strictly build graph optimization.

## Theme: Session auditability and resume safety
- Added codex session artifacts:
  - `codex/PLAN.md`
  - `codex/SESSION_LOG.md`
  - `codex/DECISIONS.md`
  - `codex/CHECKPOINTS.md`
  - `codex/VERIFICATION.md`
  - `codex/CHANGELOG_DRAFT.md`
- Recorded baseline, step-level, and final verification outcomes, including explicit environment limitations for Tauri Linux packaging.
