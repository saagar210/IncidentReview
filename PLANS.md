# IncidentReview Plans

This file is the living execution plan for this repository. It is updated when steps change.

## Goal

Implement **IncidentReview** as a local-first macOS desktop app (Tauri + Rust + React/TS) that ingests export artifacts (Jira CSV, Slack transcripts, docs notes), computes deterministic metrics, renders dashboards (ECharts), and generates a deterministic QIR Markdown report. Optional local AI is via **Ollama on 127.0.0.1 only** and must be evidence-cited.

## Binding Constraints (from `AGENTS.md`)

- No external AI APIs; AI is localhost-only via Ollama (`127.0.0.1`).
- Offline-by-default runtime; do not add network dependencies for normal app operation.
- Deterministic truth: metrics/timestamps/rollups are computed in Rust (`crates/qir_core`) only.
- No silent error handling; no silent defaults for unknown values (use NULL/None + validation warnings).
- Maintain module boundaries and repo structure exactly:
  - `crates/qir_core`: schema/migrations, ingest, validators, metrics, analytics payloads, report generation
  - `crates/qir_ai`: Ollama client/health checks, evidence store, chunking/embeddings/similarity search, citation enforcement, prompts
  - `src-tauri`: thin RPC only
  - `src`: UI only; must not compute metrics
  - required folders: `src/`, `src-tauri/`, `crates/qir_core/`, `crates/qir_ai/`, `migrations/`, `fixtures/`
- After each meaningful change-set: run verification commands and append a log entry to `HINSITE.md`.

## Verification Commands (must exist and be used)

These are required by `AGENTS.md`:

- `pnpm lint`
- `pnpm test`
- `pnpm tauri build`
- `cargo test -p qir_core`
- `cargo test -p qir_ai`

## Execution Plan (dependency order)

1. Phase 0: Scaffold + canonical verification
   - Initialize git repo (if needed).
   - Scaffold Tauri + React/TS frontend + Rust workspace to match required structure.
   - Add `package.json` scripts so `pnpm lint`, `pnpm test`, and `pnpm tauri build` are canonical and repeatable.
   - Add `HINSITE.md` and log the verification outcomes.

2. Phase 1: Deterministic foundation (`crates/qir_core`)
   - SQLite schema + migrations in `migrations/`, implemented/owned by `qir_core`.
   - Migration runner + repositories in `qir_core`.
   - Ingest:
     - Jira CSV import (mapping-driven; export-based).
     - Slack transcript import (file or paste) into structured events.
   - Validators:
     - timestamp ordering checks (nullable fields, no autocorrect)
     - percent field validation (nullable, 0..100)
     - dedupe and conflict surfacing (no silent merges)
   - Deterministic metrics engine (MTTD/MTTA/MTTR and related) + quarter rollups.
   - Fixtures under `fixtures/` (sanitized) + tests, including snapshot/golden tests where applicable.

3. Phase 2: Dashboards (analytics payloads + ECharts UI)
   - Define versioned analytics payloads in `qir_core` for cache invalidation.
   - Frontend renders ECharts dashboards from payloads only.
   - Drill-down to incident lists backing each chart and reconciliation to incident totals.

4. Phase 3: Report generation (`crates/qir_core`)
   - Deterministic Markdown generator with stable ordering.
   - Golden snapshot tests against sanitized fixtures.
   - UI trigger + Tauri wiring for export.

5. Phase 4: Thin Tauri RPC layer + AppError mapping
   - Implement Tauri commands as thin wrappers around `qir_core` (and later `qir_ai`).
   - Typed payloads (TS types) + runtime validation at the boundary.
   - Unified `AppError` (`code`, `message`, `details?`, `retryable`) end-to-end.
   - Frontend toasts with “Show details”; retry only if `retryable=true`.

6. Phase 5: Optional local AI (`crates/qir_ai`) only after core is solid
   - Ollama client + health checks (127.0.0.1 only).
   - Evidence chunk store + embeddings + similarity search.
   - Citation enforcement hard-fail: `AI_CITATION_REQUIRED` if missing citations.
   - Tauri endpoints + UI gating behind health checks.

## Progress Notes (implemented)

- Phase 0 scaffold is complete and all required verification commands are green.
- Phase 1 foundations are in place: migrations, Jira CSV ingest (fixture + test), Slack transcript ingest (fixture + test), validators, and deterministic per-incident metrics.
- Phase 2/3/4 initial vertical slice exists: versioned dashboard payloads + ECharts UI + drill-down table, deterministic Markdown report generation with a golden fixture, and thin Tauri commands.
- Phase 5 has started (foundations only): Ollama localhost-only health checks, file-based evidence store, and citation enforcement guardrail. Drafting/indexing is not implemented yet.
- Deliverable Set 1 is complete: Jira CSV import UX + mapping profile CRUD + profile-based import with inserted/updated/skipped + warnings + conflict surfacing.
- Deliverable Set 2 is complete: Slack ingest UX + deterministic timestamp normalization contract (raw preservation) + Validation/Anomalies UI baseline.
- Deliverable Set 3 is complete: dashboards expansion (Detection/Response/Vendor-Service) + deterministic QIR report v1 + incident drill-down detail UX.
- Deliverable Set 4 is complete: backup/restore + sanitized export + demo dataset tooling + git hygiene.

## Stop Conditions

Stop (and surface the minimal unblock) if:
- a required verification command fails and cannot be fixed in the same change-set, or
- a change would violate `AGENTS.md` invariants, or
- a product decision is required that is not covered by repository docs.
