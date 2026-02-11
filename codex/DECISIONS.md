# Decisions

## 2026-02-10 — Prefer config-level chunking over runtime lazy imports
- Context: Build output warns about oversized chunks during `pnpm tauri build`.
- Decision: Implement deterministic `manualChunks` in `vite.config.ts` first.
- Rationale: Minimal, reversible, and lower behavioral risk than changing feature loading semantics.
- Alternatives considered:
  - Add dynamic imports per feature route (rejected for this pass due to broader app behavior impact).
  - Ignore warning (rejected; misses incremental hardening opportunity).

## 2026-02-10 — Remove `vendor-react` split to avoid circular chunk coupling
- Context: First manual chunk strategy introduced Rollup circular chunk warning (`vendor-misc` ↔ `vendor-react`).
- Decision: Collapse react and other general deps into `vendor-misc`; keep dedicated `vendor-echarts` only.
- Rationale: avoids fragile cross-chunk cycles while still achieving meaningful chunk-size reduction.
