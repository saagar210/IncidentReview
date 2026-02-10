# IncidentReview

Local-first macOS desktop app for Quarterly Incident Reviews (QIR): ingest Jira + Slack + docs exports, compute deterministic metrics (MTTD/MTTA/MTTR and more), render storytelling dashboards (ECharts), and generate an executive-ready QIR packet. Optional local AI via Ollama (localhost only) for evidence-backed drafting and theme synthesis.

## Why this exists

Quarterly reviews are one of the few recurring moments where incident programs can:
- turn timelines into trends,
- convert data into decisions,
- and demonstrate operational maturity.

**IncidentReview** reduces the time spent assembling QIR materials and increases the quality and consistency of the outputs, without sending internal data to third parties.

## Core principles

- **Local-first**: no cloud services required.
- **Deterministic truth layer**: all metrics are computed via audited logic (no AI arithmetic).
- **Evidence-first AI**: AI may draft summaries and recommendations, but must cite evidence chunks or explicitly mark UNKNOWN.
- **Offline by default**: the app runs without network access. Optional local AI uses **only** `127.0.0.1` (Ollama).
- **Portable**: designed to move between personal and work machines without changing the workflow.

---

## Features

### Ingest
- Jira incident export ingestion (CSV, mapping UI with saved mappings)
- Slack transcript ingestion (file or paste), timeline extraction into structured events
- Optional artifact attachments (docs notes, files) with SHA-256 hashing and dedupe
- Validation report for missing/contradictory timestamps and out-of-range values

### Deterministic metrics
Per incident:
- MTTD (start → first observed)
- IT awareness lag (first observed → IT awareness)
- MTTA (awareness → ack)
- Time to mitigation (ack → mitigate)
- MTTR / time to resolve (start/observed → resolve)

Quarter rollups:
- incident volume and severity/impact distributions
- repeat offenders by vendor/service
- weighted “pain” scores (impact × degradation × duration) with configurable weights

### Dashboards (ECharts)
Storytelling dashboards with drill-down:
- Quarter at a glance (KPIs + deltas)
- Detection story (source mix, awareness lag)
- Response story (mitigation vs resolution, distributions)
- Vendor/service reliability (heatmaps, recurrence)
- Communications story (time to first comms, cadence, SLA adherence)
- Action effectiveness (completion rate, aging, categories)

### QIR packet
- One-click generation of a QIR report (Markdown)
- Optional PDF export (added only if it doesn’t introduce toolchain fragility)
- Stable ordering for snapshot testing and reproducibility

### Local AI (optional, Ollama)
- Incident narrative drafts (what/impact/mitigation/learned), with citations
- Theme naming and description (themes derived from embeddings-first clustering)
- Draft next-quarter action plan (People/Process/Tech/Vendor/Monitoring/Docs), with evidence links

> AI is additive. The app remains fully usable without Ollama.

---

## Non-goals (initially)

- Live Jira/Slack API integrations (export-based ingestion first)
- Slide deck export (PPTX/Google Slides)
- Fine-tuning models (not required for strong results)
- Multi-tenant shared database or cloud sync

---

## Tech stack

- **Tauri** (desktop shell)
- **Rust** (core engine: ingest, normalize, validate, metrics, analytics, report)
- **React + TypeScript** (UI)
- **SQLite** (storage)
- **ECharts** (dashboard rendering)
- **Ollama** (optional local LLM + embeddings via localhost HTTP)

---

## Architecture overview

### Module boundaries
- `crates/qir_core`: domain model, storage, ingest/parsers, validators, metrics, analytics datasets, report generation
- `crates/qir_ai`: Ollama client, chunking/embeddings, evidence store, prompt templates, guardrails (citation enforcement)
- `src-tauri`: thin command layer (RPC), error mapping, state wiring (DB pool, caches)
- `src`: frontend UI + ECharts chart builders; no metric computation

### Data flow
1. **Ingest** (CSV/transcripts/notes) → normalize into canonical schema
2. **Validate** (ordering, missing fields, anomalies)
3. **Compute** deterministic metrics and cached rollups
4. **Render** dashboards from analytics payloads
5. **Generate** QIR report (Markdown; optional PDF)
6. **Optional AI**: build evidence index, draft narratives/themes/actions with citations

---

## Data location and privacy

All data is stored locally. Default path on macOS:

- SQLite DB: `~/Library/Application Support/IncidentReview/incidentreview.sqlite`
- Artifact store (if copy-into-store enabled): `~/Library/Application Support/IncidentReview/artifacts/`

The repository must never contain real incident data. Use the sanitized fixtures and demo dataset generator for screenshots and public sharing.

---

## Repository structure

```
IncidentReview/
  AGENTS.md
  README.md

  src/                # Frontend (React/TS)
  src-tauri/          # Tauri shell + commands (thin)
  crates/
    qir_core/         # deterministic engine
    qir_ai/           # Ollama + evidence + guardrails
  migrations/         # SQL migrations
  fixtures/           # sanitized demo + golden datasets
  scripts/            # helper scripts (optional)
```

---

## Development setup (macOS)

### Prerequisites
- Node.js (LTS recommended)
- pnpm
- Rust toolchain (stable)
- Tauri prerequisites for macOS builds

> Keep installs explicit: do not auto-install tooling in scripts without developer approval.

### Install dependencies
```bash
pnpm install
```

### Run in dev mode
```bash
pnpm tauri dev
```

### Run tests
```bash
pnpm test
pnpm lint
cargo test -p qir_core
cargo test -p qir_ai
```

### Build
```bash
pnpm tauri build
```

---

## Local AI via Ollama (optional)

1. Install and run Ollama.
2. Pull a model of your choice (example only; pick based on local policy/perf):
   - a smaller fast model for drafts
   - a larger model for higher-quality summaries

IncidentReview will only call:
- `http://127.0.0.1:<ollama_port>/...`

If Ollama is not running, AI features are disabled and the app remains fully functional.

---

## Demo mode and fixtures

- `fixtures/demo/`: sanitized dataset for UI screenshots
- `fixtures/golden/`: golden dataset used for snapshot tests

The app must provide:
- “Seed demo dataset”
- “Export sanitized dataset”
- “Backup/restore database”

---

## Verification standards

- No silent error handling
- No “default to zero” for unknown values; prefer NULL + validation warnings
- All dashboards must reconcile to incident table totals
- AI sections must be evidence-cited or explicitly marked UNKNOWN
- Stable, reproducible report outputs for snapshot tests

---

## License

TBD.
