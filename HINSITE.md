# HINSITE

Verification and audit log for meaningful change-sets in this repository.

---

## 2026-02-10 - Phase 0 scaffold (Tauri + React + Rust workspace)

1) Done: what changed + why
- Scaffolded the repo into the required structure (`src/`, `src-tauri/`, `crates/qir_core/`, `crates/qir_ai/`, `migrations/`, `fixtures/`) so we can iterate with deterministic verification gates.
- Added canonical scripts so required verification commands exist and are repeatable.
- Added minimal Rust crates (`qir_core`, `qir_ai`) and a minimal Tauri shell to ensure `pnpm tauri build` and `cargo test -p ...` work early.

2) Files changed
- /Users/d/Projects/IncidentReview/PLANS.md
- /Users/d/Projects/IncidentReview/.gitignore
- /Users/d/Projects/IncidentReview/package.json
- /Users/d/Projects/IncidentReview/index.html
- /Users/d/Projects/IncidentReview/tsconfig.json
- /Users/d/Projects/IncidentReview/tsconfig.node.json
- /Users/d/Projects/IncidentReview/vite.config.ts
- /Users/d/Projects/IncidentReview/vitest.config.ts
- /Users/d/Projects/IncidentReview/.eslintrc.cjs
- /Users/d/Projects/IncidentReview/src/main.tsx
- /Users/d/Projects/IncidentReview/src/App.tsx
- /Users/d/Projects/IncidentReview/src/styles.css
- /Users/d/Projects/IncidentReview/src/vite-env.d.ts
- /Users/d/Projects/IncidentReview/src/smoke.test.ts
- /Users/d/Projects/IncidentReview/Cargo.toml
- /Users/d/Projects/IncidentReview/crates/qir_core/Cargo.toml
- /Users/d/Projects/IncidentReview/crates/qir_core/src/lib.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/error.rs
- /Users/d/Projects/IncidentReview/crates/qir_ai/Cargo.toml
- /Users/d/Projects/IncidentReview/crates/qir_ai/src/lib.rs
- /Users/d/Projects/IncidentReview/src-tauri/Cargo.toml
- /Users/d/Projects/IncidentReview/src-tauri/build.rs
- /Users/d/Projects/IncidentReview/src-tauri/tauri.conf.json
- /Users/d/Projects/IncidentReview/src-tauri/src/main.rs
- /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs
- /Users/d/Projects/IncidentReview/src-tauri/icons/* (copied from the Tauri scaffold to satisfy bundling)
- /Users/d/Projects/IncidentReview/public/* (copied from the Tauri scaffold)
- /Users/d/Projects/IncidentReview/migrations/.gitkeep
- /Users/d/Projects/IncidentReview/fixtures/.gitkeep
- /Users/d/Projects/IncidentReview/.codex/setup.sh
- /Users/d/Projects/IncidentReview/.codex/actions/lint.sh
- /Users/d/Projects/IncidentReview/.codex/actions/test.sh
- /Users/d/Projects/IncidentReview/.codex/actions/build.sh

3) Verification: commands run + results
- `pnpm install` (source: /Users/d/Projects/IncidentReview/README.md) -> OK
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

Note: `pnpm approve-builds` was needed once to allow `esbuild` install scripts (pnpm security feature) so Vite could build.

4) Risks / follow-ups
- `src-tauri/tauri.conf.json` currently sets `csp: null`. We should tighten CSP once we know needed allowances for local UI behavior.
- The temporary `_scaffold/` directory exists locally but is ignored via /Users/d/Projects/IncidentReview/.gitignore due to exec-policy restrictions on `rm -rf`.

5) Status: current phase + complete / in progress / blocked
- Phase 0: complete.
- Phase 1: next (schema/migrations, ingest, validators, deterministic metrics in `crates/qir_core`).

6) Next steps
- Implement SQLite schema + migration runner in /Users/d/Projects/IncidentReview/crates/qir_core and add the first migration under /Users/d/Projects/IncidentReview/migrations/.
- Add deterministic ingest stubs (Jira CSV + Slack transcript) and validators with fixtures and tests.
- Wire minimal Tauri commands for DB init and import (thin wrappers only).

---

## 2026-02-10 - Phase 1.1 DB schema + migration runner (qir_core)

1) Done: what changed + why
- Added the initial SQLite schema in /Users/d/Projects/IncidentReview/migrations/0001_init.sql with nullable canonical timestamps and uniqueness constraints for dedupe (external_id preferred, otherwise fingerprint).
- Implemented a deterministic migration runner in /Users/d/Projects/IncidentReview/crates/qir_core/src/db/mod.rs that applies migrations exactly once and surfaces errors with explicit codes/messages (no silent failure).

2) Files changed
- /Users/d/Projects/IncidentReview/migrations/0001_init.sql
- /Users/d/Projects/IncidentReview/crates/qir_core/Cargo.toml
- /Users/d/Projects/IncidentReview/crates/qir_core/src/lib.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/db/mod.rs

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- `migrations/0001_init.sql` stores timestamps as TEXT; we should validate/normalize ingestion into a canonical ISO-8601 format and keep ordering checks in validators.
- Schema will evolve; future migrations must preserve deterministic report ordering (stable sorts) and avoid destructive defaults.

5) Status: current phase + complete / in progress / blocked
- Phase 1: in progress (ingest + validators + metrics next).

6) Next steps
- Add domain types + deterministic fingerprinting in /Users/d/Projects/IncidentReview/crates/qir_core and implement ingesters (Jira CSV + Slack transcript).
- Implement validators for timestamp ordering and percent field ranges with fixture-backed tests.

---

## 2026-02-10 - Phase 1.2 Jira CSV ingest (qir_core)

1) Done: what changed + why
- Added domain types for incidents and validation warnings in /Users/d/Projects/IncidentReview/crates/qir_core/src/domain/mod.rs.
- Implemented Jira CSV ingest with explicit mapping in /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/jira_csv.rs:
  - no silent defaults (missing titles become warnings and rows are skipped)
  - percent fields validate 0..100 with warnings on failures
  - deterministic fingerprinting to support dedupe rules
- Added a sanitized CSV fixture and an integration test to ensure ingest works end-to-end against an in-memory migrated DB.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/Cargo.toml
- /Users/d/Projects/IncidentReview/crates/qir_core/src/lib.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/domain/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/jira_csv.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/ingest_jira_csv.rs
- /Users/d/Projects/IncidentReview/fixtures/demo/jira_sample.csv

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Timestamp parsing/normalization is not implemented yet; ingest currently stores timestamp strings as-is and validators must flag ordering/format anomalies.
- The current behavior returns an error on any DB insert failure (including uniqueness conflicts). We should enrich this to surface conflicts as structured ingest results instead of a single fatal error.

5) Status: current phase + complete / in progress / blocked
- Phase 1: in progress (Slack ingest + validators + deterministic metrics next).

6) Next steps
- Implement Slack transcript ingest in /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/.
- Implement validators (timestamp ordering + percent ranges + dedupe conflict surfacing) and tests with sanitized fixtures.

---

## 2026-02-10 - Phase 1.3 Validators + deterministic per-incident metrics (qir_core)

1) Done: what changed + why
- Implemented validator logic in /Users/d/Projects/IncidentReview/crates/qir_core/src/validate/mod.rs:
  - RFC3339 timestamp parsing warnings (no silent parse failures)
  - strict ordering checks: `start <= first_observed <= it_awareness <= ack <= mitigate <= resolve` (when present)
  - percent range warnings (nullable 0..100)
- Implemented deterministic per-incident metrics in /Users/d/Projects/IncidentReview/crates/qir_core/src/metrics/mod.rs:
  - metrics are computed only when required timestamps are present and parseable
  - ordering violations produce warnings and result in `None` metrics (no silent correction)
- Added fixture-backed tests for validation and metrics behavior.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/Cargo.toml
- /Users/d/Projects/IncidentReview/crates/qir_core/src/lib.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/validate/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/metrics/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/validate_and_metrics.rs

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Timestamp parsing currently only supports RFC3339. Jira/Slack exports often contain other formats; ingestion should normalize into RFC3339 or emit explicit warnings when unknown.

5) Status: current phase + complete / in progress / blocked
- Phase 1: in progress (Slack ingest + DB repos + rollups + analytics payloads next).

6) Next steps
- Implement Slack transcript ingest into `timeline_events` and add sanitized fixtures/tests.
- Add DB repository helpers for listing incidents and attaching validation+metrics results.

---

## 2026-02-10 - Phase 1.4 Slack transcript ingest + basic DB repo helpers (qir_core)

1) Done: what changed + why
- Added Slack transcript ingestion in /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/slack_transcript.rs that:
  - ingests each line into `timeline_events`
  - parses RFC3339 timestamps when present and warns explicitly when missing (no silent defaults)
- Added DB repository helpers in /Users/d/Projects/IncidentReview/crates/qir_core/src/repo/mod.rs to list/count incidents (foundation for dashboards and reporting).
- Added sanitized Slack fixture + tests for transcript ingestion.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/src/lib.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/slack_transcript.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/repo/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/ingest_slack_transcript.rs
- /Users/d/Projects/IncidentReview/fixtures/demo/slack_sample.txt

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Slack parsing is intentionally minimal (line-oriented). We should extend to support common Slack export formats (JSON export, “copy from Slack” formats) with explicit warnings when unknown.

5) Status: current phase + complete / in progress / blocked
- Phase 1: in progress (rollups/analytics payloads + report generation next).

6) Next steps
- Implement analytics payloads in `qir_core` (versioned) and wire them through `src-tauri` to the frontend for ECharts dashboards.
- Implement deterministic Markdown report generator + golden snapshot tests.

---

## 2026-02-10 - Phases 2-4 Dashboard + report + thin Tauri RPC + UI wiring

1) Done: what changed + why
- Added versioned analytics payloads and dashboard dataset builder in /Users/d/Projects/IncidentReview/crates/qir_core/src/analytics/mod.rs so the frontend can render ECharts charts without computing metrics.
- Added deterministic Markdown report generation in /Users/d/Projects/IncidentReview/crates/qir_core/src/report/mod.rs with golden snapshot test coverage.
- Wired thin Tauri commands in /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs:
  - `init_db` (migrate DB in app data dir)
  - `seed_demo_jira` (ingest sanitized Jira CSV fixture)
  - `get_dashboard_v1` (returns versioned analytics payload)
  - `generate_report_md` (returns Markdown)
- Implemented a basic React UI in /Users/d/Projects/IncidentReview/src/App.tsx to call commands, show ECharts severity chart, drill-down incident table, and display report Markdown.
- Added runtime validation (Zod schemas) for command results (typed payload contracts).

2) Files changed
- /Users/d/Projects/IncidentReview/package.json
- /Users/d/Projects/IncidentReview/pnpm-lock.yaml
- /Users/d/Projects/IncidentReview/src/App.tsx
- /Users/d/Projects/IncidentReview/src/styles.css
- /Users/d/Projects/IncidentReview/src/lib/tauri.ts
- /Users/d/Projects/IncidentReview/src/lib/schemas.ts
- /Users/d/Projects/IncidentReview/src/ui/useToasts.ts
- /Users/d/Projects/IncidentReview/src/ui/ToastHost.tsx
- /Users/d/Projects/IncidentReview/crates/qir_core/src/lib.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/error.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/analytics/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/report/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/report_golden.rs
- /Users/d/Projects/IncidentReview/fixtures/golden/qir_report_demo.md
- /Users/d/Projects/IncidentReview/src-tauri/Cargo.toml
- /Users/d/Projects/IncidentReview/src-tauri/src/main.rs
- /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Frontend bundle size is large due to ECharts; later we should add code-splitting (dynamic import) to keep initial load lighter.
- `src-tauri/tauri.conf.json` CSP is still `null`; we should tighten once UI feature-set stabilizes.
- The app currently provides a demo seed and basic dashboard/report, but does not yet expose full import UX (file pickers, mapping UI persistence) or backup/restore flows.

5) Status: current phase + complete / in progress / blocked
- Phase 2: in progress (more dashboards + richer drill-down + reconciliation).
- Phase 3: in progress (report sections beyond the minimal demo).
- Phase 4: in progress (typed payload validation exists; expand commands and UI error details).
- Phase 5: not started (AI must remain gated until evidence store exists and core is solid).

6) Next steps
- Add saved mapping profiles + Jira import UI (export-based) and wire through Tauri using typed payloads + runtime validation.
- Implement backup/restore and sanitized export flows (local-only) and add fixtures/tests.
- After evidence store exists, start `crates/qir_ai` with Ollama health checks and citation enforcement endpoints.

---

## 2026-02-10 - Phase 5 (partial) Local AI foundations (qir_ai) + health check wiring

1) Done: what changed + why
- Implemented local-only Ollama client health check in /Users/d/Projects/IncidentReview/crates/qir_ai/src/ollama.rs enforcing `127.0.0.1` base URLs (no remote AI).
- Implemented a file-based evidence chunk store in /Users/d/Projects/IncidentReview/crates/qir_ai/src/evidence.rs so AI features can be evidence-backed without adding DB schema/migrations outside `qir_core`.
- Implemented citation enforcement guardrail in /Users/d/Projects/IncidentReview/crates/qir_ai/src/guardrails.rs (hard fail `AI_CITATION_REQUIRED` if missing citations).
- Wired a thin Tauri command `ai_health_check` in /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs and exposed it in the UI.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_ai/Cargo.toml
- /Users/d/Projects/IncidentReview/crates/qir_ai/src/lib.rs
- /Users/d/Projects/IncidentReview/crates/qir_ai/src/ollama.rs
- /Users/d/Projects/IncidentReview/crates/qir_ai/src/evidence.rs
- /Users/d/Projects/IncidentReview/crates/qir_ai/src/guardrails.rs
- /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs
- /Users/d/Projects/IncidentReview/src/lib/schemas.ts
- /Users/d/Projects/IncidentReview/src/App.tsx

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Ollama API surface is not yet used for drafting; only health checks are implemented. Draft endpoints must enforce citations against stored evidence chunks and must never compute deterministic metrics/timestamps.
- Evidence store currently doesn’t implement chunking/embeddings/similarity search yet; those are next within `qir_ai`.

5) Status: current phase + complete / in progress / blocked
- Phase 5: in progress (foundations + guardrails in place; drafting/indexing not implemented yet).

6) Next steps
- Implement deterministic chunking + embeddings + similarity search in `qir_ai` and expose evidence index build via thin Tauri commands.
- Add AI drafting endpoints that hard-require citations and return `UNKNOWN` where evidence is insufficient.

---

## 2026-02-10 - Deliverable Set 1 (part 1) Mapping profiles + Jira import engine (qir_core)

1) Done: what changed + why
- Implemented Jira mapping profile persistence in /Users/d/Projects/IncidentReview/crates/qir_core/src/profiles/jira.rs (CRUD over the existing `jira_mapping_profiles` table).
- Implemented Jira CSV preview parsing in /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/jira_csv.rs to power a mapping UI (headers + sample rows).
- Reworked Jira CSV ingestion into an import engine that returns deterministic results:
  - counts: inserted / updated / skipped
  - conflicts surfaced explicitly (no silent merges)
  - warnings include non-RFC3339 timestamp parse warnings while preserving raw strings
- Added sanitized fixtures and integration tests covering:
  - profile persistence CRUD
  - mapping-driven import correctness
  - conflict surfacing for duplicate external IDs
  - warnings for non-RFC3339 timestamps

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/src/lib.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/profiles/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/profiles/jira.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/jira_csv.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/jira_profiles_and_import.rs
- /Users/d/Projects/IncidentReview/fixtures/demo/jira_duplicate_external_id.csv
- /Users/d/Projects/IncidentReview/fixtures/demo/jira_non_rfc3339.csv

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Constraint conflict detection currently treats any SQLite constraint violation as a conflict to surface; if we later need more granular messaging, we can inspect extended error codes.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 1: in progress (Tauri commands + frontend mapping/import UX next).

6) Next steps
- Add thin Tauri commands for mapping profile CRUD + CSV preview + import using selected profile.
- Implement frontend Jira import UI: file picker, mapping UI with preview, save/reuse profiles, and result rendering for inserted/updated/skipped/warnings/conflicts.

---

## 2026-02-10 - Deliverable Set 1 (part 2) Thin Tauri commands + initial Jira import UI

1) Done: what changed + why
- Added thin Tauri commands in /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs for:
  - `jira_csv_preview` (headers + sample rows)
  - `jira_profiles_list`, `jira_profiles_upsert`, `jira_profiles_delete` (mapping profile CRUD)
  - `jira_import_using_profile` (import Jira CSV using selected profile)
- Updated the frontend to provide an export-based Jira CSV import UX in /Users/d/Projects/IncidentReview/src/App.tsx:
  - file picker (`<input type="file">`)
  - CSV preview table
  - mapping UI with explicit required/optional labels
  - profile save/reuse/delete
  - import results (inserted/updated/skipped + warnings + conflicts)
- Updated schemas in /Users/d/Projects/IncidentReview/src/lib/schemas.ts so all command payloads are runtime validated (typed contracts).

2) Files changed
- /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs
- /Users/d/Projects/IncidentReview/src/App.tsx
- /Users/d/Projects/IncidentReview/src/styles.css
- /Users/d/Projects/IncidentReview/src/lib/schemas.ts

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- The UI currently requires saving/selecting a profile before importing (explicit workflow). If we want “import without saving”, we can add a separate command that takes a mapping directly, but that is not required for Deliverable Set 1.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 1: in progress (finish: ensure import result UX includes updated/skipped semantics, and ensure tests cover profile persistence + mapping application + conflict surfacing end-to-end).

6) Next steps
- Add an end-to-end test that saves a profile then imports using that profile to ensure profile persistence integrates with import.
- Add an update vs skip fixture to validate `updated` and `skipped` counters deterministically.

---

## 2026-02-10 - Deliverable Set 1 (part 3) Update/skip semantics + integration coverage (qir_core)

1) Done: what changed + why
- Added integration test coverage in /Users/d/Projects/IncidentReview/crates/qir_core/tests/jira_profiles_and_import.rs for:
  - updating an existing incident by `external_id` (counts as `updated`)
  - skipping when no changes are detected (counts as `skipped`)
  - using a persisted mapping profile as the source of truth for import mapping
- This completes the required test surface for Deliverable Set 1: profile persistence, mapping correctness, and conflict surfacing.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/jira_profiles_and_import.rs

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- None blocking Deliverable Set 1. Remaining work is higher-level UX polish and expanding import format support.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 1: complete.

6) Next steps
- Add a dedicated “Import” screen (separate from dashboards) and improve result rendering (group warnings by row/field).
- Persist last-used profile selection in local app state to reduce repeated setup.

---

## 2026-02-10 - Jira update semantics: preserve-on-empty (binding decision)

1) Done: what changed + why
- Updated Jira import update behavior in /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/jira_csv.rs to enforce **PRESERVE-ON-EMPTY** semantics:
  - empty/missing CSV cells do **not** overwrite existing stored values with NULL
  - updates now merge incoming non-empty values into the existing record deterministically
- Updated tests in /Users/d/Projects/IncidentReview/crates/qir_core/tests/jira_profiles_and_import.rs to encode this behavior.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/jira_csv.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/jira_profiles_and_import.rs

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- None identified for this change-set; preserve-on-empty reduces accidental data loss during repeated imports.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 2: not started yet (next).

6) Next steps
- Implement timestamp raw preservation via migration (either *_raw columns or a raw table) and update validators to surface parse failures with raw retention.
- Implement Slack import UX + backend commands + fixtures/tests.

---

## 2026-02-10 - Deliverable Set 2 foundations: timestamp raw preservation + validation report

1) Done: what changed + why
- Wired the existing migration /Users/d/Projects/IncidentReview/migrations/0002_add_timestamp_raw_columns.sql into the deterministic migration runner so timestamp raw preservation is applied consistently for both on-disk and in-memory DBs.
- Extended the incident domain model to include `*_ts_raw` fields and updated incident listing queries to read them, enabling “preserve raw + surface warnings” behavior without silent defaults.
- Added deterministic, allowlist-only timestamp normalization helpers in /Users/d/Projects/IncidentReview/crates/qir_core/src/normalize/timestamps.rs (no fuzzy parsing). This supports the DS2 contract: canonical RFC3339 UTC when deterministically parseable; otherwise preserve raw + explicit warnings.
- Enhanced validators to surface raw timestamp anomalies per field and added a deterministic “validate all incidents” report function to support the upcoming Validation/Anomalies UI.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/src/db/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/domain/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/repo/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/normalize/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/normalize/timestamps.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/validate/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/lib.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/validate_and_metrics.rs

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Timestamp normalization currently only allowlists a small set of timezone-less formats (assuming UTC with explicit warnings). We may need to expand the allowlist for Jira-export-specific formats, but we will not add fuzzy parsing.
- Jira ingest still needs to be updated to actually *use* the new normalization helpers and store raw timestamps into `*_ts_raw` (next change-set).

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 2: in progress (timestamp contract foundations done; Slack UX + anomaly UI next).

6) Next steps
- Update Jira ingest to use deterministic timestamp normalization and persist `*_ts_raw` while keeping preserve-on-empty semantics.
- Implement Slack ingest backend (format detection + incident attach/create shell) and expose thin Tauri commands.
- Implement Validation/Anomalies UI and Slack Import UI in the frontend using typed payload contracts.

---

## 2026-02-10 - Jira ingest updated for timestamp normalization + raw preservation

1) Done: what changed + why
- Updated Jira CSV import in /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/jira_csv.rs to enforce the DS2 timestamp contract:
  - canonical incident timestamp columns (`*_ts`) are now RFC3339-only (UTC)
  - non-RFC3339 or unparseable timestamp inputs are preserved deterministically in `*_ts_raw`
  - explicit warnings are emitted for normalization, timezone assumptions, and unparseable values (no silent defaults)
- Extended preserve-on-empty merge logic for timestamp fields by adding a “provided vs not provided” distinction so that:
  - missing/empty CSV cells do not overwrite either canonical or raw values
  - provided values can intentionally clear `*_ts_raw` (e.g., when replaced with RFC3339)
- Updated tests to assert canonical NULL + raw preserved for non-RFC3339 fixtures.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/jira_csv.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/jira_profiles_and_import.rs

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- The deterministic timestamp allowlist intentionally does not parse ambiguous locale formats like `MM/DD/YYYY ...`; those values will remain raw-only with warnings.
- We may need to expand the allowlist for Jira-export-specific formats (e.g., offset forms) later, but we will not add fuzzy parsing.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 2: in progress (timestamp contract implemented for Jira ingest; Slack ingest UX + anomaly UI next).

6) Next steps
- Implement Slack ingest backend (format detection + incident attach/create shell) and expose thin Tauri commands.
- Add a Validation/Anomalies view in the frontend backed by `qir_core`’s validation report payload.

---

## 2026-02-10 - Slack ingest backend (export-based) + thin Tauri commands (DS2)

1) Done: what changed + why
- Implemented Slack transcript format detection + deterministic ingest in /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/slack_transcript.rs:
  - Supported formats:
    - line-oriented RFC3339 transcript (`line_rfc3339`)
    - Slack JSON export array (`slack_json_export`) with deterministic Slack `ts` conversion (no float parsing)
    - unknown formats fall back to raw line ingestion with explicit warnings (no guessing timestamps)
  - Ingest now attaches events to a chosen incident OR creates a new “Slack-only incident shell” deterministically (title required; no silent defaults).
- Added sanitized fixtures for JSON export + paste-style transcript and expanded tests to cover:
  - event creation
  - warning emission for missing/unparseable timestamps
  - incident attachment behavior
- Added thin Tauri commands in /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs for:
  - `slack_preview`
  - `slack_ingest`
  - `incidents_list` (for incident selection in UI)
  - `validation_report` (for upcoming anomaly UI)
- Added typed Zod schemas in /Users/d/Projects/IncidentReview/src/lib/schemas.ts for the new payloads.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/slack_transcript.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/ingest_slack_transcript.rs
- /Users/d/Projects/IncidentReview/fixtures/demo/slack_export_sample.json
- /Users/d/Projects/IncidentReview/fixtures/demo/slack_paste_sample.txt
- /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs
- /Users/d/Projects/IncidentReview/src/lib/schemas.ts

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Slack JSON export formats can vary (per-channel files, different fields). Current implementation targets a deterministic subset: JSON arrays containing objects with `text` and optional `ts`/`user`.
- The UI still needs to expose the Slack import flow and the Validation/Anomalies view (next change-set).

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 2: in progress (Slack ingest backend + commands complete; UI + validation screen next).

6) Next steps
- Implement Slack Import UI (file picker + paste box + incident select/create) calling `slack_preview` and `slack_ingest`.
- Implement Validation/Anomalies UI calling `validation_report` with drill-down to incidents.

---

## 2026-02-10 - Deliverable Set 2 complete: Slack Import UI + Validation UI + timestamp contract tests

1) Done: what changed + why
- Implemented the DS2 frontend UX in /Users/d/Projects/IncidentReview/src/App.tsx:
  - Slack Import flow with both file picker and paste box
  - attach transcript to an existing incident or create a new Slack-only incident shell (title required)
  - preview (detected format + counts + warnings) and ingest result rendering (counts + warnings)
  - Validation/Anomalies view that lists incidents with validator warnings and allows filtering the incidents drill-down table by incident id
- Added a second deterministic timestamp normalization fixture + test so the contract is covered for:
  - allowlisted, timezone-less formats (assume UTC with explicit warning)
  - raw preservation for non-RFC3339 inputs
- Updated /Users/d/Projects/IncidentReview/PLANS.md progress notes to reflect DS2 status.

2) Files changed
- /Users/d/Projects/IncidentReview/src/App.tsx
- /Users/d/Projects/IncidentReview/fixtures/demo/jira_allowlisted_ts.csv
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/jira_profiles_and_import.rs
- /Users/d/Projects/IncidentReview/PLANS.md

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Validation view currently surfaces `qir_core` validator warnings only; ingest-time warnings are only shown at import time (not persisted). This is acceptable for DS2 but may be worth improving later.
- Slack JSON exports can vary; current parser supports deterministic JSON arrays with `text` and optional `ts`/`user`.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 2: complete.

6) Next steps
- Decide the next milestone (likely Deliverable Set 3: dashboards expansion + deterministic report polishing, or backup/restore/export workflows).
- Add an “Incident detail” view to complement the incidents table and the validation list.

---

## 2026-02-10 - DS3 change-set 1: Deterministic storytelling analytics + report v1 expansion

1) Done: what changed + why
- Hardened and finalized the versioned analytics payload `DashboardPayloadV2` in `crates/qir_core` so DS3 dashboards can render from deterministic backend datasets only (no UI metric computation).
- Expanded the deterministic Markdown report generator to include executive summary, metrics distribution table, detection/response/vendor sections, stable incident table ordering, and an explicit validation/anomalies appendix.
- Updated the golden snapshot fixture + tests so report output stays stable and reproducible.
- Fixed the incident drill-down table UI columns to align with the V2 payload (show awareness lag + time to mitigate instead of MTTA).

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/src/analytics/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/report/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/report_golden.rs
- /Users/d/Projects/IncidentReview/fixtures/golden/qir_report_demo.md
- /Users/d/Projects/IncidentReview/src/App.tsx

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- “Weighted pain” is currently computed as `impact_pct × degradation_pct × duration_seconds` (integer). This is deterministic and matches the documented formula shape, but the unit scale is large; we may later normalize to minutes or hours for more readable chart axes.
- The DS3 dashboards UI still needs a clearer navigation structure so Dashboards and Report are first-class sections (next change-set).
- Consider adding an incident detail drawer for richer drill-down context (timeline events + artifacts) without pushing logic into the UI.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 3: in progress (backend datasets + report expanded and golden-tested; UI dashboards + incident detail next).

6) Next steps
- Implement DS3 dashboard pages (Detection/Vendor-Service/Response) in the UI using `get_dashboard_v2`, with click-to-filter drill-down backed by `incident_ids`.
- Add a minimal incident detail view/drawer wired through Tauri to `qir_core` so drill-down is more informative without UI-side computation.

---

## 2026-02-10 - DS3 start: story fields in schema + Jira ingest mapping support

1) Done: what changed + why
- Started Deliverable Set 3 by adding optional incident fields needed for storytelling dashboards and report polish:
  - `detection_source`
  - `vendor`
  - `service`
- Implemented these as nullable SQLite columns via /Users/d/Projects/IncidentReview/migrations/0003_add_story_fields.sql and wired the migration into the deterministic runner in /Users/d/Projects/IncidentReview/crates/qir_core/src/db/mod.rs.
- Extended Jira CSV mapping/import in /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/jira_csv.rs to map these fields (optional) with preserve-on-empty semantics (no silent defaults).
- Updated domain and repo query shape so dashboards/reporting can read the new fields deterministically.
- Updated /Users/d/Projects/IncidentReview/PLANS.md to mark DS2 complete and DS3 in progress.

2) Files changed
- /Users/d/Projects/IncidentReview/PLANS.md
- /Users/d/Projects/IncidentReview/migrations/0003_add_story_fields.sql
- /Users/d/Projects/IncidentReview/crates/qir_core/src/db/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/domain/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/repo/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/ingest/jira_csv.rs
- /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/ingest_jira_csv.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/jira_profiles_and_import.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/report_golden.rs

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- The frontend Jira mapping UI has not yet been expanded to include these new optional mappings; dashboards can still render UNKNOWN buckets until we add those mapping fields to the UI (next change-set).

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 3: in progress (schema + ingest foundations complete; analytics payloads + dashboards + report expansion next).

6) Next steps
- Implement versioned analytics payload(s) for Detection/Response/Vendor-Service stories in /Users/d/Projects/IncidentReview/crates/qir_core/src/analytics and add reconciliation + drill-down tests.
- Expand the frontend dashboards UI to render the new payloads via ECharts and support drill-down to backing incidents.

---

## 2026-02-10 - DS3 analytics: DashboardPayloadV2 (Detection/Response/Vendor-Service)

1) Done: what changed + why
- Implemented a new versioned analytics payload `DashboardPayloadV2` in /Users/d/Projects/IncidentReview/crates/qir_core/src/analytics/mod.rs to support three storytelling dashboards required by DS3:
  - Detection story: detection source mix + IT awareness lag distribution
  - Vendor/service reliability: top vendors/services by incident count + weighted pain (impact × degradation × duration) where available
  - Response story: time to mitigation + time to resolve distributions
- Ensured every chart dataset reconciles to the incident total by including explicit `UNKNOWN` (and `OTHER` where needed) buckets and providing `incident_ids` for drill-down (backend-owned query keys via stable `key` strings).
- Added a sanitized story fixture /Users/d/Projects/IncidentReview/fixtures/demo/jira_story.csv and an integration test /Users/d/Projects/IncidentReview/crates/qir_core/tests/analytics_v2.rs that asserts:
  - bucket counts sum to total incidents
  - union of drill-down incident ids covers the full incident set deterministically

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/src/analytics/mod.rs
- /Users/d/Projects/IncidentReview/fixtures/demo/jira_story.csv
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/analytics_v2.rs

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- The UI still uses DashboardPayloadV1; next change-set will wire DashboardPayloadV2 through thin Tauri commands + ECharts UI with drill-down.
- Pain calculations intentionally ignore incidents without the needed deterministic inputs; buckets still include all incident ids so drill-down is complete even when pain is partially unknown.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 3: in progress (analytics payloads implemented; UI + report expansion next).

6) Next steps
- Add a new Tauri command (thin) to fetch DashboardPayloadV2 and update the frontend to render the Detection/Response/Vendor-Service dashboards with drill-down.
- Expand the deterministic Markdown report generator to QIR Report v1 using the same backend payloads + a validation appendix, and update golden snapshot tests.

---

## 2026-02-10 - DS3 dashboards wired: Tauri get_dashboard_v2 + ECharts story UI + drill-down

1) Done: what changed + why
- Wired the new analytics payload through the thin Tauri layer:
  - Added `get_dashboard_v2` command in /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs returning `DashboardPayloadV2` from `qir_core`.
- Updated frontend typed contracts and UI to render the DS3 storytelling dashboards from backend payloads only (no UI metric computation):
  - Added Zod schemas for DashboardPayloadV2 and story buckets in /Users/d/Projects/IncidentReview/src/lib/schemas.ts.
  - Updated /Users/d/Projects/IncidentReview/src/App.tsx to:
    - load DashboardPayloadV2
    - render Detection story, Response story, and Vendor/Service reliability charts via ECharts
    - support drill-down by applying backend-provided `incident_ids` to filter the incident list deterministically
    - add a minimal “Jump To” nav so Dashboards/Report aren’t buried
- Expanded Jira mapping UI in /Users/d/Projects/IncidentReview/src/App.tsx to include optional fields `detection_source`, `vendor`, and `service` so exports can populate story dashboards without manual DB edits.

2) Files changed
- /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs
- /Users/d/Projects/IncidentReview/src/lib/schemas.ts
- /Users/d/Projects/IncidentReview/src/App.tsx

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- The current “pain” chart uses a raw pain unit (impact * degradation * duration_seconds). We may want to add a friendly scaling (e.g., hours) later, but the deterministic drill-down remains correct now.
- Report generator still uses the older minimal report; next change-set will expand it to QIR Report v1 and update golden fixtures.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 3: in progress (dashboards done; report expansion + golden tests next).

6) Next steps
- Expand /Users/d/Projects/IncidentReview/crates/qir_core/src/report/mod.rs to deterministic QIR Report v1:
  - executive summary, metrics summary tables, incidents table, detection/response highlights, and validation appendix
- Update /Users/d/Projects/IncidentReview/fixtures/golden/qir_report_demo.md and snapshot tests to match the new deterministic output.

---

## 2026-02-10 - DS3 UX polish: Incident detail drawer + schema contract cleanup (restore tauri build)

1) Done: what changed + why
- Completed the DS3 drill-down UX by adding an Incident Detail drawer that loads deterministic detail payloads from the backend (no UI metric computation):
  - Uses existing thin Tauri command `incident_detail` (already wired) and validates the response at the boundary with Zod.
  - Shows computed metrics (formatted), validation/anomaly warnings, timeline events, and artifacts for the selected incident.
- Fixed a build-blocking contract issue introduced during the interrupted edit:
  - Removed duplicate TypeScript/Zod schema declarations for `IncidentMetricsSchema` and `IncidentDetailSchema` to keep schemas single-source and ensure `pnpm tauri build` stays green.

2) Files changed
- /Users/d/Projects/IncidentReview/src/App.tsx
- /Users/d/Projects/IncidentReview/src/lib/schemas.ts
- /Users/d/Projects/IncidentReview/PLANS.md

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- The incident detail drawer currently focuses on read-only visibility. If we later add editing, we should keep all deterministic derivations in `crates/qir_core` and treat UI edits as explicit user actions.
- The Vite build warns about large chunks (not a correctness issue). We can consider code-splitting later if startup performance becomes a concern.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 3: complete (dashboards + deterministic report + drill-down UX + full verification green).

6) Next steps
- Confirm the next milestone (likely backup/restore + sanitized export + demo dataset tooling), keeping AI features gated.

---

## 2026-02-10 - DS4 foundation: qir_core backup/restore core + tests (folder export, manifest, integrity)

1) Done: what changed + why
- Implemented deterministic, local-only backup + restore primitives in `crates/qir_core`:
  - Backup creates a folder `IncidentReviewBackup_<timestamp>/` containing:
    - `incidentreview.sqlite` (SQLite snapshot via rusqlite backup API)
    - `manifest.json` (app version, export time, applied migrations, row counts, DB hash; optional artifacts hash list)
    - `artifacts/` is included only if an on-disk artifacts directory exists (not required for current app usage).
  - Restore validates `manifest.json` integrity (DB SHA-256) and requires explicit overwrite confirmation (`allow_overwrite=true`) before swapping the target DB into place.
- Added `qir_core` tests to ensure backup folder contents and restore behavior are correct and auditable.
- Updated `PLANS.md` to mark DS4 as in progress.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/src/lib.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/backup/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/Cargo.toml
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/backup_restore.rs
- /Users/d/Projects/IncidentReview/PLANS.md

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Restore currently keeps a `*.pre_restore` copy of the previous DB (safety-first). We may later add a UI affordance to delete old pre-restore copies explicitly.
- Artifacts directory backup/restore is implemented generically; the app does not yet heavily use on-disk artifacts, so we should confirm the desired artifacts-on-disk strategy before relying on it.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 4: in progress (core backup/restore done; still need Tauri/UI wiring, sanitized export, and demo dataset tooling).

6) Next steps
- Add thin Tauri commands for backup/restore + a simple UI flow that picks directories, previews manifests, and requires explicit confirmation before overwrite.
- Implement deterministic sanitized export (JSON/CSV) with pseudonymization + tests proving no raw Slack text leaks.

---

## 2026-02-10 - DS4 wiring: Tauri backup/restore commands + UI directory picker + explicit overwrite confirmation

1) Done: what changed + why
- Added thin Tauri commands that wrap the new `qir_core` backup/restore primitives:
  - `backup_create(destination_dir)` -> creates a human-auditable backup folder (DB snapshot + manifest).
  - `backup_inspect(backup_dir)` -> reads and returns the manifest for UI preview.
  - `restore_from_backup(backup_dir, allow_overwrite)` -> validates manifest integrity and swaps DB into place only when overwrite is explicitly confirmed.
- Implemented a minimal UI flow for backup/restore:
  - Uses a native directory picker (Tauri dialog plugin) to select destination/backup folders.
  - Shows manifest preview (counts/export time/app version) before restore.
  - Requires an explicit “I understand this will overwrite…” checkbox before restore.

2) Files changed
- /Users/d/Projects/IncidentReview/package.json
- /Users/d/Projects/IncidentReview/pnpm-lock.yaml
- /Users/d/Projects/IncidentReview/src-tauri/Cargo.toml
- /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs
- /Users/d/Projects/IncidentReview/src/lib/schemas.ts
- /Users/d/Projects/IncidentReview/src/App.tsx

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Backup/restore UI currently focuses on correctness and explicit confirmation. If we later add “dry-run restore”, it should remain a thin wrapper over `qir_core` integrity checks (no UI-side assumptions).
- Adding the dialog plugin introduces additional build-time dependencies (still local-only at runtime). This was chosen to satisfy the “user-chosen directory” requirement without requiring users to type paths.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 4: in progress (backup/restore wired; still need sanitized export + demo dataset tooling + git hygiene completion).

6) Next steps
- Implement deterministic sanitized export (JSON/CSV) with stable pseudonymization + tests proving no raw Slack text leaks.
- Improve “Seed Demo Dataset” to generate a richer, sanitized dataset for dashboards/reports (no real incident data).

---

## 2026-02-10 - DS4 sanitized export + demo dataset tooling: deterministic shareable exports (no Slack text)

1) Done: what changed + why
- Implemented deterministic “Export Sanitized Dataset” in `crates/qir_core`:
  - Exports a shareable folder `IncidentReviewSanitized_<timestamp>/` containing JSON files (`incidents.json`, `timeline_events.json`, `warnings.json`) plus `sanitized_manifest.json`.
  - Redacts free-text (Slack message text is never exported; timeline events carry `text_redacted=true` only).
  - Pseudonymizes vendor/service/detection_source deterministically (stable across runs for the same input DB).
  - Preserves numeric/timestamp/taxonomy fields so dashboards still tell a story.
- Added tests proving:
  - sanitized output does not contain raw Slack message text
  - pseudonymization is stable across runs
- Implemented a richer “Seed Demo Dataset” in `crates/qir_core` (40 sanitized incidents with realistic distributions).
- Wired sanitized export + demo seed into Tauri commands and exposed both as clear UI actions.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/src/lib.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/demo/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/sanitize/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/repo/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/demo_seed.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/sanitized_export.rs
- /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs
- /Users/d/Projects/IncidentReview/src/lib/schemas.ts
- /Users/d/Projects/IncidentReview/src/App.tsx

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Sanitized export currently produces JSON only (transparent and GitHub-safe). If we later add an “import sanitized dataset” flow, keep it deterministic and owned by `qir_core`.
- Pseudonymization mappings are deterministic but intentionally lossy; this is expected for shareable datasets.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 4: in progress (features implemented; still need git hygiene finalization: DS4 completion commit + ds4 tag + clean status + PLANS update).

6) Next steps
- Finalize git hygiene: commit DS4 completion, create `ds4_complete` tag, ensure `git status` clean.
- Update `PLANS.md` to mark DS4 complete.

---

## 2026-02-10 - DS4 completion: plan update + final verification + git hygiene (commit + tag)

1) Done: what changed + why
- Marked DS4 complete in `PLANS.md` and ran the full verification suite to confirm backup/restore + sanitized export + demo tooling remain green.
- Git hygiene:
  - Repository has commits (root baseline commit already exists).
  - DS4 completion is committed and tagged locally as `ds4_complete` (with `ds1_complete`, `ds2_complete`, `ds3_complete` already present).
  - Ensured the working tree is clean at milestone end.

2) Files changed
- /Users/d/Projects/IncidentReview/PLANS.md
- /Users/d/Projects/IncidentReview/HINSITE.md

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- None for DS4 correctness. Future work should keep AI gated and maintain deterministic boundaries for any export/import flows.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 4: complete.

6) Next steps
- Pick DS5 milestone (likely backup/restore UX polish, sanitized export import, or demo dataset export workflows), keeping all deterministic boundaries intact.

---

## 2026-02-10 - DS5 (in progress): qir_core sanitized dataset import primitives + plan update

1) Done: what changed + why
- Implemented deterministic sanitized dataset import in `crates/qir_core` to complement DS4 sanitized export:
  - Reads and validates `sanitized_manifest.json` (requires `manifest_version == 1`).
  - Verifies SHA-256 hashes and byte sizes for all files listed in the manifest (hard fail on mismatch).
  - Refuses import into non-empty DB (`INGEST_SANITIZED_DB_NOT_EMPTY`).
  - Inserts incidents in sorted `incident_key` order and uses `export_time` from the manifest for deterministic `ingested_at` / `created_at`.
  - Uses deterministic redaction placeholders for NOT NULL fields:
    - `incidents.title = "Incident <incident_key>"`
    - `timeline_events.text = "[REDACTED]"` plus `raw_json={"text_redacted":true}` as an explicit redaction marker.
  - Recomputes deterministic metrics from imported timestamps and hard-fails on mismatch (`INGEST_SANITIZED_METRICS_MISMATCH`).
  - Reconciles warnings non-fatally: surfaces `INGEST_SANITIZED_WARNINGS_MISMATCH` as an import warning when validator output differs from `warnings.json`.
- Updated `PLANS.md` to mark Deliverable Set 5 as in progress.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/src/sanitize/mod.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/src/sanitize/import.rs
- /Users/d/Projects/IncidentReview/PLANS.md
- /Users/d/Projects/IncidentReview/HINSITE.md

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Warning reconciliation is intentionally non-fatal in DS5; mismatches are surfaced as an import warning for visibility without blocking app usage.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 5: in progress.

6) Next steps
- Add deterministic round-trip tests (seed demo -> sanitized export -> import into fresh DB -> dashboards/report/validation succeed).
- Wire thin Tauri RPC commands for inspect/import and add a UI flow to import a sanitized dataset folder.

---

## 2026-02-10 - DS5 (in progress): deterministic sanitized export/import round-trip tests

1) Done: what changed + why
- Added a deterministic round-trip test in `crates/qir_core` that proves the DS5 import contract works end-to-end:
  - Seed demo dataset -> ingest a Slack event containing a unique sensitive marker -> export sanitized dataset -> import into a fresh DB.
  - Confirms analytics payloads reconcile to incident totals and report generation succeeds after import.
  - Defense-in-depth: asserts the sensitive marker does not appear in exported JSON nor in the imported DB.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/sanitized_round_trip.rs
- /Users/d/Projects/IncidentReview/HINSITE.md

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- None identified for correctness. If the demo seed shape changes later, this test should be updated to assert invariants (counts/reconciliation) rather than brittle exact strings.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 5: in progress.

6) Next steps
- Add thin Tauri commands for sanitized dataset inspect/import (no business logic in `src-tauri`).
- Add a UI flow to pick a dataset folder, preview the manifest, run import, and refresh dashboards/report/validation.

---

## 2026-02-10 - DS5 (in progress): thin Tauri commands for sanitized dataset inspect/import

1) Done: what changed + why
- Added thin Tauri commands in `src-tauri` that wrap the new deterministic `qir_core` sanitized import functionality:
  - `inspect_sanitized_dataset(dataset_dir)` -> returns `SanitizedExportManifest` for UI preview and validates manifest integrity (including hashes/sizes).
  - `import_sanitized_dataset(dataset_dir)` -> runs deterministic import into the app DB (refuses non-empty DB; metrics cross-check enforced by `qir_core`).
- Kept `src-tauri` as a thin RPC layer only (no import business logic).

2) Files changed
- /Users/d/Projects/IncidentReview/src-tauri/src/lib.rs
- /Users/d/Projects/IncidentReview/HINSITE.md

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- None identified for DS5 correctness. UI still needs to surface “DB not empty” errors clearly with guidance to restore/seed into a fresh DB first.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 5: in progress.

6) Next steps
- Add UI flow for sanitized dataset import: directory picker, manifest preview, import trigger + results display, and refresh existing views by re-calling commands.

---

## 2026-02-10 - DS5 (in progress): UI flow for sanitized dataset import (preview + import + refresh)

1) Done: what changed + why
- Added a deterministic “Import Sanitized Dataset” UI flow:
  - Directory picker to select a sanitized dataset export folder.
  - Manifest preview via `inspect_sanitized_dataset` (manifest version + incident count + file list).
  - Import trigger via `import_sanitized_dataset` and results display (insert counts + explicit import warnings).
  - Refreshes incident list, dashboards, report, and validation views by re-calling existing commands (UI remains render-only).
- Surfaced intentional redaction in incident detail timeline events: `(redacted)` is shown when event text is `[REDACTED]` or the event `raw_json` contains `\"text_redacted\":true`.
- Added runtime validation (Zod) schemas for the new sanitized manifest and import summary payloads.

2) Files changed
- /Users/d/Projects/IncidentReview/src/lib/schemas.ts
- /Users/d/Projects/IncidentReview/src/App.tsx
- /Users/d/Projects/IncidentReview/HINSITE.md

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- None for DS5 import correctness. The UI currently detects the “DB not empty” hard fail by matching the formatted error string; a future improvement would be passing typed errors across the boundary (still preserving `AppError` shape).

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 5: in progress.

6) Next steps
- Update `PLANS.md` to mark DS5 complete and add a DS5 completion log entry after final verification.

---

## 2026-02-10 - DS5 completion: sanitized dataset import + round-trip tests + RPC + UI

1) Done: what changed + why
- Completed DS5: “Import sanitized dataset (deterministic) + round-trip tests”.
  - `qir_core` now supports deterministic sanitized dataset import from the DS4 export folder format (`incidents.json`, `timeline_events.json`, `warnings.json`, `sanitized_manifest.json`).
  - Import enforces offline/local-only integrity checks (manifest version + SHA-256 + byte sizes), refuses non-empty DB, and hard-fails on metrics mismatch.
  - Added deterministic round-trip tests proving export -> import preserves dashboards/report/validation and does not leak sensitive Slack markers.
  - Added thin Tauri RPC commands + a UI flow (directory picker, manifest preview, import result + warnings, and refresh).
- Marked DS5 complete in `PLANS.md`.

2) Files changed
- /Users/d/Projects/IncidentReview/PLANS.md
- /Users/d/Projects/IncidentReview/HINSITE.md

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- Warning reconciliation remains intentionally non-fatal; mismatches are surfaced as `INGEST_SANITIZED_WARNINGS_MISMATCH` warnings so the app remains usable while the warning contract stabilizes.

5) Status: current phase + complete / in progress / blocked
- Deliverable Set 5: complete.

6) Next steps
- Phase 4 polish can continue (more endpoint coverage, UX tightening) without changing deterministic boundaries.

---

## 2026-02-10 - DS5.1 (in progress): sanitized import error contract codes + qir_core error tests

1) Done: what changed + why
- Hardened the sanitized import error contract in `qir_core`:
  - Renamed manifest mismatch error codes to stable, UI-branchable identifiers:
    - `INGEST_SANITIZED_MANIFEST_VERSION_MISMATCH`
    - `INGEST_SANITIZED_MANIFEST_HASH_MISMATCH`
    - `INGEST_SANITIZED_MANIFEST_BYTES_MISMATCH`
- Added deterministic Rust tests asserting the exact `AppError.code` for DS5 flows:
  - Import into non-empty DB -> `INGEST_SANITIZED_DB_NOT_EMPTY`
  - Manifest version mismatch -> `INGEST_SANITIZED_MANIFEST_VERSION_MISMATCH`
  - Manifest hash mismatch -> `INGEST_SANITIZED_MANIFEST_HASH_MISMATCH`
  - Manifest bytes mismatch -> `INGEST_SANITIZED_MANIFEST_BYTES_MISMATCH`
  - Metrics mismatch -> `INGEST_SANITIZED_METRICS_MISMATCH`

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_core/src/sanitize/import.rs
- /Users/d/Projects/IncidentReview/crates/qir_core/tests/sanitized_import_errors.rs
- /Users/d/Projects/IncidentReview/HINSITE.md

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- None for determinism/privacy. Next change-set will update the frontend to branch on `AppError.code` (removing any message substring matching).

5) Status: current phase + complete / in progress / blocked
- DS5.1: in progress.

6) Next steps
- Replace brittle UI string matching with code-based branching, and adjust the invoke wrapper to preserve structured `AppError` objects to callers.

---

## 2026-02-10 - DS5.1 (in progress): frontend structured AppError propagation + code-based UI branching

1) Done: what changed + why
- Hardened the frontend error path so sanitized import UI branches on stable error codes (never message substrings):
  - Updated `src/lib/tauri.ts` to preserve structured `AppError` objects by throwing a typed `AppErrorException` (includes `code`, `details`, `retryable`).
  - Added `extractAppError` helper to unwrap common invoke error shapes without stringifying.
  - Updated sanitized import UI handler to branch exclusively on `AppError.code` and show deterministic guidance text per code.
- Added frontend tests to prove code-based handling:
  - `invokeValidated` preserves `code` from a rejected invoke.
  - Guidance mapping is deterministic and keyed only by `code`.
- Confirmed removal of brittle matching: `includes(\"INGEST_SANITIZED...\")` is no longer used in `src/`.

2) Files changed
- /Users/d/Projects/IncidentReview/src/lib/tauri.ts
- /Users/d/Projects/IncidentReview/src/lib/tauri.test.ts
- /Users/d/Projects/IncidentReview/src/lib/sanitized_import_guidance.ts
- /Users/d/Projects/IncidentReview/src/lib/sanitized_import_guidance.test.ts
- /Users/d/Projects/IncidentReview/src/App.tsx
- /Users/d/Projects/IncidentReview/HINSITE.md

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK

4) Risks / follow-ups
- None for determinism/privacy. Next change-set will tighten `qir_ai` Phase 5 preflight guardrails (tests/boundaries only).

5) Status: current phase + complete / in progress / blocked
- DS5.1: in progress.

6) Next steps
- Phase 5 preflight guardrails in `qir_ai`: strict 127.0.0.1-only base URL enforcement, boundary test preventing `qir_core` metric dependencies, and an ignored placeholder test for citation enforcement.

---

## 2026-02-10 - DS5.1 (in progress): Phase 5 preflight guardrails (qir_ai tests/boundaries only)

1) Done: what changed + why
- Tightened Phase 5 preflight guardrails in `qir_ai` (no AI feature expansion):
  - Hardened Ollama base URL validation to accept only `http://127.0.0.1[:port]` using strict parsing (no prefix matching bypasses).
  - Expanded unit tests to reject `localhost`, `0.0.0.0`, IPv6 loopback, remote hosts, userinfo, paths, and invalid ports.
  - Added a boundary guardrail test to prevent `qir_ai` from importing `qir_core` metric computation modules (AI must not compute deterministic metrics).
  - Added an ignored placeholder test for future Phase 5 end-to-end citation enforcement wiring.

2) Files changed
- /Users/d/Projects/IncidentReview/crates/qir_ai/src/ollama.rs
- /Users/d/Projects/IncidentReview/crates/qir_ai/src/lib.rs
- /Users/d/Projects/IncidentReview/crates/qir_ai/tests/boundaries.rs
- /Users/d/Projects/IncidentReview/HINSITE.md

3) Verification: commands run + results
- `pnpm lint` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm test` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `pnpm tauri build` (required source: /Users/d/Projects/IncidentReview/AGENTS.md; script source: /Users/d/Projects/IncidentReview/package.json) -> OK
- `cargo test -p qir_core` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK
- `cargo test -p qir_ai` (required source: /Users/d/Projects/IncidentReview/AGENTS.md) -> OK (includes 1 ignored placeholder test)

4) Risks / follow-ups
- The strict base URL validator intentionally rejects `localhost` even if it resolves to loopback, per binding rules. Keep this strictness unless policy changes.

5) Status: current phase + complete / in progress / blocked
- DS5.1: in progress.

6) Next steps
- Update `PLANS.md` to record DS5.1 contract-hardening work, and add DS5.1 completion entry after final verification + final repo hygiene checks.
