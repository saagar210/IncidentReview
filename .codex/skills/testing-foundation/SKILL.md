---
name: testing-foundation
description: Enforce meaningful test coverage for all production changes. Use for any feature/refactor/fix that touches runtime code.
---

# Testing Foundation

## Trigger
Use whenever changed files include production paths (`src/`, `src-tauri/`, `crates/`).

## Required outcomes
- Add or update tests in the same change.
- Cover behavior matrix: success + edge + invalid/error.
- Keep tests deterministic (seeded data, isolated fixtures).
- Run repository verification commands from `.codex/verify.commands`.

## Behavior matrix checklist
- Primary behavior is asserted.
- Boundary condition is asserted.
- Invalid input or failure-path is asserted.
- Authorization/authentication path is asserted when relevant.
- Idempotency/concurrency path is asserted when relevant.

## Test quality rules
- Prefer behavior tests over implementation details.
- Mock only external boundaries (network/time/random/third-party SDK).
- Do not mock the unit under test.
- Avoid snapshot-only tests unless paired with semantic assertions.
- Use stable selectors (`getByRole`, `getByLabel`, `getByTestId` as fallback).

## UI-specific rules
- Cover loading, empty, error, success.
- Cover disabled and focus-visible for interactive controls.
- Use boundary-level mocks only for IPC and file picker interactions.

## Command-contract rules
- Use integration tests for Tauri commands and schema-level contracts.
- Contract tests must fail on schema drift.
- Command/API changes require OpenAPI docs refresh.

## E2E rules
- Keep PR e2e suite smoke-focused and deterministic.
- Use retries only in CI and collect trace on first retry.
- Prefer critical user journeys: workspace setup, imports, dashboards, report.

## Reviewer/fixer integration
1. Invoke read-only reviewer in QA critic mode.
2. Accept high-confidence findings.
3. Invoke fixer to apply findings in priority order.
4. Re-run reviewer and verification gates.

## Deliverable format
Return:
1) Behavior matrix covered
2) Tests added/updated with paths
3) Commands run and pass/fail
4) Residual risk and suggested next hardening step
