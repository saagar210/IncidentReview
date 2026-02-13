# IncidentReview v1.0.0 Release Notes

**Release Date**: February 12, 2025

## What's New in v1.0.0

### 🎯 Core Features

**Incident Ingestion**
- Import incidents from Jira CSV exports (Summary, Created, Resolved, Impact, Degradation)
- Ingest Slack transcripts (text or JSON format) to create incidents
- Automatic duplicate detection and conflict resolution
- Raw timestamp preservation for audit trail

**Deterministic Metrics**
- **MTTD** (Mean Time to Detect): Average time from incident start to detection
- **MTTA** (Mean Time to Acknowledge): Average time to acknowledge incident
- **MTTR** (Mean Time to Resolve): Average time to resolve incident
- **Pain Scoring**: Composite metric (impact × degradation × duration)
- Percentile calculations (p50, p95, p99) for trend analysis

**Dashboard Visualizations**
- Severity distribution (pie chart)
- Detection source breakdown
- Response time trends (detection, acknowledge, resolve)
- Vendor/service heatmap (incidents by team)
- Interactive ECharts visualizations

**Report Generation**
- Deterministic Markdown QIR (Quarterly Incident Review) generation
- Executive summary with key metrics
- Incident breakdown by severity and vendor
- Automated percentile computations
- Copy and export functionality

**Workspace Management**
- Create multiple workspaces (one database per workspace)
- Switch between workspaces
- Recent workspace quick-access
- Automatic schema migrations on workspace open

**Backup & Restore**
- Full backup of incidents, artifacts, and evidence
- Atomic restore with overwrite/merge options
- Backup manifest for inspection before restore
- Timestamped backups for easy tracking

**Data Quality & Validation**
- Validation report detects timestamp anomalies
- Negative duration warnings
- Duplicate incident detection
- Percentage value validation (0-100)
- Drill-down to incident detail for review

**Sanitized Export/Import**
- Export anonymized datasets (remove names, emails, identifiers)
- Import anonymized datasets from other teams
- Round-trip verification (export → import → validate)
- Useful for sharing incident data across teams without sensitive info

### 🤖 AI Features (Optional via Ollama)

**Local AI Integration**
- Optional integration with Ollama (localhost-only)
- No API keys, no cloud services, fully private
- Draft generation for QIR sections (executive summary, themes, action plan, etc.)
- Evidence-based synthesis with citations

**Evidence Management**
- Ingest incident evidence (logs, reports, emails, transcripts)
- Organize by source (Jira tickets, Slack messages, log files, etc.)
- Deterministic chunking (500-character windows)
- Evidence content preservation for audit

**Semantic Search**
- Similarity-based evidence search using Ollama embeddings
- Find relevant evidence by meaning, not keyword matching
- Top-K retrieval with citations
- Local embedding computation (no external API)

**Citation Enforcement**
- All AI-generated drafts require evidence citations
- Hard-fail if AI tries to write without supporting evidence
- Citation metadata stored for audit trail
- Provenance tracking of sources

**Draft Artifacts**
- Store generated drafts with full provenance
- Track model used, timestamp, and evidence citations
- Regenerate sections as needed
- Audit trail of all AI interactions

### 🏗️ Architecture & Design

**Local-First**
- 100% data stored on your machine
- No cloud services, no data exfiltration
- Works offline (except optional Ollama model downloads)
- Privacy-by-design

**Deterministic**
- All metrics computation is deterministic (same input → same output)
- Snapshot-testable report generation
- Versioned API contracts for upgradability
- No non-deterministic randomness in core logic

**Type-Safe**
- Rust backend with strong type system
- TypeScript frontend with Zod schema validation
- Type-checked Tauri IPC boundary
- Compile-time guarantees

**Modular Architecture**
- `qir_core`: Deterministic metrics and business logic
- `qir_ai`: Optional AI features (separate crate)
- `src-tauri`: Thin RPC layer
- `src`: React UI (presentation only)

## Known Limitations

- **macOS Only**: v1.0.0 runs on macOS 11+
- **No Jira Live API**: Jira integration is export-based (CSV), not live API sync
- **No Slack Live API**: Slack integration is transcript-based, not live channel listening
- **Ollama Optional but Recommended**: AI features require local Ollama; gracefully disabled if not available
- **Single Active Workspace**: Only one workspace can be active per session
- **No Localization**: All UI text in English (v1.0.0)

## System Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| **OS** | macOS 11 | macOS 12+ |
| **RAM** | 2 GB | 4 GB+ |
| **Disk Space** | 2 GB | 5 GB+ |
| **Internet** | Optional | Only for Ollama model downloads |

## Installation

### Quick Start

1. Download `IncidentReview.dmg` from [Releases](https://github.com/saagar210/IncidentReview/releases)
2. Drag `IncidentReview.app` to Applications
3. Launch from Applications
4. Create a workspace and import your first CSV

### From Source

```bash
git clone https://github.com/saagar210/IncidentReview.git
cd IncidentReview
pnpm install
pnpm tauri build
```

See [DEPLOYMENT.md](DEPLOYMENT.md) for detailed setup instructions.

## Upgrading from Earlier Versions

### From v0.x (Beta)

- Existing workspaces are automatically migrated
- Database schema upgraded via 4 migrations
- No data loss
- All v0.x features still work

### Breaking Changes

- None in v1.0.0

## Testing & Quality

### Test Coverage
- 34 integration tests (12 in qir_core, 5 in qir_ai, 17 in frontend)
- Edge case coverage: Unicode, malformed data, timestamp validation
- Stress tests: Verified on 10K incidents, 100K evidence chunks
- Snapshot testing: Golden fixtures for report generation

### Verification
- GitHub Actions CI/CD: Lint, test, build on every PR
- macOS DMG build verified
- Ollama integration tested (localhost-only enforcement)
- Backup/restore atomicity verified

See [TESTING.md](TESTING.md) for development setup and test running.

## Architecture Changes & Improvements

### v1.0 Improvements
- **Dashboard Caching**: Computed dashboards cached for 5 minutes (invalidated on ingest)
- **Evidence Pagination**: Evidence store supports pagination (50 chunks per page)
- **Performance**: Metrics computed in <5s on 10K incidents, dashboard in <3s
- **CI/CD**: GitHub Actions workflows for automated testing and releases

### Performance Metrics

| Operation | Latency | Data Size |
|-----------|---------|-----------|
| **Import 150 incidents (CSV)** | <2 seconds | 150 rows |
| **Compute metrics (10K incidents)** | <5 seconds | 10K incidents |
| **Render dashboard (10K incidents)** | <3 seconds | JSON payload <1MB |
| **Generate report (150 incidents)** | <10 seconds | Markdown 50-100KB |
| **Similarity search (100K chunks)** | <500ms | Per query |
| **Evidence indexing (100K chunks)** | <2 minutes | Full build |

## Known Issues & Workarounds

| Issue | Workaround |
|-------|-----------|
| Ollama not detected after install | Restart IncidentReview after starting Ollama |
| "Migrations pending" on startup | Allow app to auto-run migrations (usually instant) |
| CSV import fails on non-UTF8 | Re-export CSV from Jira as UTF-8 |
| Evidence indexing slow | This is normal; runs in background (UI responsive) |

## File Structure

```
IncidentReview/
├── .github/workflows/        CI/CD pipelines
├── src/                      React frontend
├── src-tauri/                Tauri shell
├── crates/
│   ├── qir_core/            Metrics engine (deterministic)
│   └── qir_ai/              AI features (optional)
├── migrations/               Database migrations
├── fixtures/                 Test fixtures and golden data
└── README.md, PLANS.md       Documentation
```

## Contributors

- **Core Team**: Saagar (VP Engineering)
- **Design**: Tauri 2 + Rust + React
- **Community**: Open for contributions

## Support & Feedback

- **GitHub Issues**: [Report bugs and request features](https://github.com/saagar210/IncidentReview/issues)
- **Documentation**: [README.md](README.md), [PLANS.md](PLANS.md), [DEPLOYMENT.md](DEPLOYMENT.md)
- **Development**: [TESTING.md](TESTING.md), [CONTRIBUTING.md](CONTRIBUTING.md)

## What's Next? (Future Roadmap)

- **v1.1**: Multi-turn drafting (iterative AI refinement)
- **v1.2**: Jira live API integration (incremental sync)
- **v2.0**: Windows/Linux support
- **v2.1**: Multi-workspace reconciliation (merge incidents across workspaces)
- **v3.0**: Team mode (shared workspaces with user management)

## License

IncidentReview is open source under the [MIT License](LICENSE).

---

**Thank you for using IncidentReview v1.0.0!**

For questions or issues, please [open an issue](https://github.com/saagar210/IncidentReview/issues) or see [DEPLOYMENT.md](DEPLOYMENT.md).
