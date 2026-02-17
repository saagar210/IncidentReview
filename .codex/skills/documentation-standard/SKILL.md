---
name: documentation-standard
description: Keep docs as living artifacts generated and validated from code changes. Use for API, architecture, or workflow updates.
---

# Documentation Standard

## Trigger
Use whenever changes affect command contracts, public modules, architecture, setup, or operations.

## Required outcomes
- Regenerate OpenAPI artifact from source schemas.
- Regenerate reference docs from TS exports.
- Sync README generated sections from canonical sources.
- Add ADR for architecture-impacting decisions.
- Run `docs:check` to enforce no drift.

## Documentation map
- `openapi/openapi.generated.json`: generated command contract artifact
- `docs/reference/`: generated API/module docs
- `docs/adr/`: architecture decisions
- `docs/architecture/overview.md`: system boundaries and data flow
- `README.md`: setup, commands, env table, troubleshooting

## Authoring rules
- Document why and boundaries, not line-by-line what code does.
- Keep module docs concise and decision-focused.
- Keep generated blocks fenced with markers; never hand-edit generated ranges.
- When behavior changes, include request/response examples.

## README sync rules
- `.env.example` is source of truth for env vars.
- Auto-generate env var table in README between markers.
- Keep setup commands validated by CI.

## ADR rules
- Add ADR for cross-cutting changes (data model, migration path, command contracts, architecture).
- Include context, decision, consequences, alternatives.
- Link PR and follow-up tasks.

## CI rules
- `docs:generate` must be deterministic.
- `docs:check` must fail if generated docs are stale.
- Command-contract changes without docs updates are blocking.

## Reviewer/fixer integration
1. Reviewer checks for missing/outdated docs in changed scope.
2. Fixer updates docs artifacts and templates.
3. Re-run docs checks and reviewer confirmation.

## Deliverable format
Return:
1) Docs artifacts generated/updated
2) Files changed with paths
3) Drift checks run and result
4) Gaps intentionally deferred with explicit owner/date
