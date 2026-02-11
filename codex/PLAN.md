# Delta Plan

## A) Executive Summary

### Current state (repo-grounded)
- Frontend build is Vite-based with a single default bundling strategy and no manual chunking strategy configured in `vite.config.ts`.
- The production build currently emits a large-chunk warning (`~1.5 MB` minified JS), which is visible in `pnpm tauri build` output.
- `echarts` and `echarts-for-react` are direct runtime dependencies and are likely major contributors to initial bundle size.
- Frontend tests are healthy (`vitest` suite passing).
- Rust core and AI crates have passing tests and strong deterministic + guardrail coverage.
- Tauri build is partially blocked in this environment due to missing Linux GLib system dependencies.

### Key risks
- Over-aggressive code splitting can degrade runtime startup if chunk graph is fragmented incorrectly.
- Any Vite config change can affect both desktop (`pnpm tauri build`) and browser build behavior.
- Hidden dependency coupling in chunks could break lazy loading if chunk names are unstable.

### Improvement themes (prioritized)
1. Reduce frontend bundle risk by introducing deterministic manual chunk boundaries for heavy vendor libraries.
2. Verify warning reduction via existing build command without altering runtime behavior.
3. Maintain auditable session artifacts and restart-safe checkpoints.

## B) Constraints & Invariants (Repo-derived)

### Explicit invariants
- Keep `src-tauri` as thin RPC and avoid moving domain logic out of `qir_core`/`qir_ai`.
- No external AI APIs; local-only behavior remains unchanged.
- Use existing verification commands (`pnpm lint`, `pnpm test`, `pnpm tauri build`, `cargo test -p qir_core`, `cargo test -p qir_ai`).

### Implicit invariants (inferred)
- UI behavior is validated primarily by unit/component tests and should remain stable.
- Build output should remain deterministic and production-capable.

### Non-goals
- No architectural refactor of React feature modules.
- No Rust-layer modifications.
- No ECharts rendering logic changes.

## C) Proposed Changes by Theme (Prioritized)

### Theme 1: Bundle splitting for heavy dependencies
- Current approach: default Rollup chunking in Vite config.
- Proposed change: add `build.rollupOptions.output.manualChunks` to split `echarts`, `react`, and remaining vendor deps into stable chunks.
- Why: reduce primary chunk size warning and improve maintainability of bundle composition.
- Tradeoffs: more network/file requests at startup; acceptable for desktop app with cached assets.
- Scope boundary: only `vite.config.ts` build config; no app code changes.
- Migration approach: additive config change with immediate build verification.

### Theme 2: Verification + audit trail hardening
- Current approach: verification exists but per-session trail may be fragmented.
- Proposed change: maintain codex logs + checkpoints and update `HINSITE.md` with completed work and verification.
- Why: supports interruption/resume and compliance with repository process.
- Tradeoffs: documentation overhead.
- Scope boundary: markdown logs only.

## D) File/Module Delta (Exact)

### ADD
- `codex/SESSION_LOG.md` — chronological execution notes.
- `codex/PLAN.md` — this delta plan.
- `codex/DECISIONS.md` — explicit judgment calls.
- `codex/CHECKPOINTS.md` — restart-safe checkpoints.
- `codex/VERIFICATION.md` — commands/results.
- `codex/CHANGELOG_DRAFT.md` — delivery draft.

### MODIFY
- `vite.config.ts` — manual chunk strategy.
- `HINSITE.md` — completion + verification record.

### REMOVE/DEPRECATE
- None.

### Boundary rules
- Allowed: Vite build config and process logs.
- Forbidden: changes to metrics/validation/AI logic, DB schema, Tauri command contracts.

## E) Data Models & API Contracts (Delta)
- Current: no data contract changes needed; schemas remain in Rust + TS.
- Proposed: none.
- Compatibility: fully backward compatible.
- Migrations: none.
- Versioning: unchanged.

## F) Implementation Sequence (Dependency-Explicit)

1. **Step S1 — Create session artifacts + record baseline**
   - Files: `codex/*.md`
   - Preconditions: baseline commands executed.
   - Dependencies: none.
   - Verify: none (documentation only).
   - Rollback: remove `codex/` files.

2. **Step S2 — Add manual chunking in Vite config**
   - Files: `vite.config.ts`
   - Preconditions: S1 complete.
   - Dependencies: existing Vite config.
   - Verify immediately: `pnpm lint`, `pnpm test`, `pnpm tauri build`.
   - Rollback: revert `vite.config.ts`.

3. **Step S3 — Update repository progress logs**
   - Files: `HINSITE.md`, `codex/SESSION_LOG.md`, `codex/DECISIONS.md`, `codex/VERIFICATION.md`, `codex/CHANGELOG_DRAFT.md`, `codex/CHECKPOINTS.md`
   - Preconditions: S2 verification complete.
   - Dependencies: S2 outcomes.
   - Verify immediately: `git diff --stat` sanity check.
   - Rollback: revert documentation changes.

4. **Step S4 — Final full suite + delivery packaging**
   - Files: none or log updates.
   - Preconditions: S3 complete.
   - Dependencies: all previous steps.
   - Verify immediately: full baseline suite re-run.
   - Rollback: if regressions, revert offending step.

## G) Error Handling & Edge Cases
- Current pattern: structured `AppError` with `code`, `message`, `details`, `retryable`.
- Proposed improvements: none in runtime code.
- Edge case: manual chunk function must return stable chunk names and avoid undefined behavior.
- Tests: rely on build/lint/test verification to catch config regressions.

## H) Integration & Testing Strategy
- Integration points: Vite build pipeline + Tauri prebuild frontend bundle.
- Unit tests: unchanged.
- Regression check: ensure large chunk warning is reduced/removed in build output.
- DoD:
  - Lint/tests pass.
  - Rust tests remain green.
  - Tauri build reaches same environment limitation point without new regressions.

## I) Assumptions & Judgment Calls
- Assumption: bundle warning is actionable and worth reducing.
- Assumption: chunk splitting in config is low risk versus application code changes.
- Judgment call: avoid adding dynamic imports to feature code for now; prefer centralized build-level chunking as reversible first increment.
