# Testing Guide

This guide covers running tests for IncidentReview, from unit tests to stress testing at scale.

## Quick Start

```bash
# Run all frontend tests
pnpm test

# Run all Rust tests (except those requiring Ollama)
cargo test -p qir_core
cargo test -p qir_ai --lib

# Run stress tests at scale
bash scripts/profile_metrics.sh
```

## Test Structure

```
IncidentReview/
├── src/__tests__/              Frontend tests (Vitest)
│   ├── features/              Feature module tests
│   ├── lib/                   Utility function tests
│   └── ...
├── crates/qir_core/tests/     Backend integration tests
│   ├── report_golden.rs       Snapshot testing
│   ├── backup_restore.rs
│   ├── edge_cases_unicode.rs  Unicode/emoji handling
│   ├── edge_cases_malformed.rs Malformed data handling
│   ├── stress_large_dataset.rs Performance at scale
│   └── ...
└── crates/qir_ai/tests/       AI feature tests
    ├── boundaries.rs          Localhost-only enforcement
    ├── index_build.rs
    ├── draft.rs               Citation enforcement
    └── stress_large_embeddings.rs Performance at scale
```

## Frontend Tests (React / TypeScript)

### Running Frontend Tests

```bash
# Run all frontend tests once
pnpm test

# Run tests in watch mode
pnpm test --watch

# Run tests matching a pattern
pnpm test MyComponent

# Run with coverage
pnpm test --coverage
```

### Frontend Test Files

Located in `src/__tests__/`:

- **Feature tests**: Smoke tests for each feature section (Workspace, Jira, Slack, etc.)
- **Utility tests**: Tests for `format.ts`, `workspace_guidance.ts`, `ai_gating.ts`, etc.
- **State machine tests**: Workspace lifecycle, error handling, state transitions

### Example Frontend Test

```typescript
// src/__tests__/lib/format.test.ts
import { describe, it, expect } from 'vitest';
import { formatDuration, formatTimestamp } from '../../lib/format';

describe('format utilities', () => {
  it('formats duration in hours', () => {
    const ms = 3600 * 1000; // 1 hour
    expect(formatDuration(ms)).toBe('1h 0m');
  });

  it('formats RFC3339 timestamp', () => {
    const ts = '2025-01-15T10:00:00Z';
    expect(formatTimestamp(ts)).toBe('Jan 15, 2025 10:00');
  });
});
```

## Backend Tests (Rust)

### Running Core Logic Tests

```bash
# Run all tests in qir_core
cargo test -p qir_core

# Run a specific test file
cargo test -p qir_core --test backup_restore

# Run with output
cargo test -p qir_core -- --nocapture

# Run in release mode (faster)
cargo test -p qir_core --release
```

### Running AI Tests

```bash
# Run AI library tests (no external dependencies)
cargo test -p qir_ai --lib

# Run AI integration tests (requires Ollama)
cargo test -p qir_ai --test boundaries -- --nocapture
```

### Backend Test Files

Located in `crates/qir_core/tests/` and `crates/qir_ai/tests/`:

| Test File | Purpose | Duration | Dependencies |
|-----------|---------|----------|--------------|
| `report_golden.rs` | Snapshot test vs golden fixture | <1s | None |
| `backup_restore.rs` | Full backup/restore cycle | <2s | None |
| `sanitized_round_trip.rs` | Export → import consistency | <2s | None |
| `jira_profiles_and_import.rs` | Jira CSV parsing + import | <2s | None |
| `analytics_v2.rs` | Dashboard payload generation | <1s | None |
| `validate_and_metrics.rs` | Validation + metric computation | <2s | None |
| `ingest_jira_csv.rs` | Jira CSV parsing edge cases | <1s | None |
| `ingest_slack_transcript.rs` | Slack transcript parsing | <1s | None |
| `edge_cases_unicode.rs` | Unicode, emoji, special chars | <2s | None |
| `edge_cases_malformed.rs` | Malformed CSV, invalid data | <2s | None |
| `stress_large_dataset.rs` | 10K+ incidents performance | 30-60s | None |
| `boundaries.rs` (qir_ai) | Localhost-only enforcement | <1s | None |
| `draft.rs` (qir_ai) | Citation-enforced drafting | 10-30s | Ollama |
| `evidence_context.rs` (qir_ai) | Evidence retrieval | 10-30s | Ollama |
| `stress_large_embeddings.rs` (qir_ai) | 100K+ chunks performance | 2-5 min | Ollama |

## Integration Tests

### Running All Integration Tests

```bash
# Run all tests (no Ollama required)
cargo test -p qir_core --all-features

# Run with output
cargo test -p qir_core --all-features -- --nocapture --test-threads=1
```

### What Integration Tests Verify

1. **Database migrations**: Schema creation and upgrades
2. **Jira ingestion**: CSV parsing, profile CRUD, import
3. **Slack ingestion**: Transcript format detection, parsing
4. **Validation**: Timestamp ordering, anomaly detection
5. **Metrics**: MTTD, MTTA, MTTR, pain scoring
6. **Analytics**: Dashboard v1/v2 payload generation
7. **Reports**: Deterministic Markdown generation
8. **Backup/Restore**: Full backup and atomic restore
9. **Sanitized data**: Export → import round-trip
10. **AI features**: Evidence store, embeddings, drafting

## Performance Testing & Profiling

### Stress Tests (Large Datasets)

```bash
# Run stress tests with performance output
bash scripts/profile_metrics.sh

# Run individual stress test
cargo test -p qir_core --test stress_large_dataset stress_test_metrics_computation_10k_incidents --ignored --release -- --nocapture

# Run all stress tests
cargo test -p qir_core --test stress_large_dataset --ignored --release -- --nocapture
```

### What Stress Tests Measure

**qir_core:**
- Metrics computation on 1K, 10K incidents
- Dashboard rendering on 1K, 10K incidents
- Report generation on 1K, 10K incidents
- Memory usage estimation
- Concurrent read performance

**qir_ai (requires Ollama):**
- Evidence store with 10K, 100K chunks
- Embedding index build time
- Similarity search latency
- Incremental index updates
- Concurrent search performance

### Expected Performance

| Operation | 1K Incidents | 10K Incidents |
|-----------|--------------|---------------|
| Metrics computation | <2s | <5s |
| Dashboard rendering | <1s | <3s |
| Report generation | <5s | <10s |
| Memory usage | <10 MB | <50 MB |

| Operation | 10K Chunks | 100K Chunks |
|-----------|-----------|------------|
| Evidence store insert | <2s | <30s |
| Embedding index build | <10s | <5 min |
| Similarity search | <500ms | <500ms |

**Note**: Times vary by machine. Above are benchmarks for modern MacBook.

## AI Testing (Ollama)

### Setting Up Ollama for Testing

1. **Install Ollama**
   ```bash
   # macOS
   brew install ollama
   # or download from https://ollama.ai
   ```

2. **Start Ollama** (in separate terminal)
   ```bash
   ollama serve
   ```

3. **Download a model** (one-time setup)
   ```bash
   ollama pull mistral
   # or: ollama pull neural-chat, llama2, etc.
   ```

4. **Verify Ollama is running**
   ```bash
   curl http://127.0.0.1:11434/api/tags
   ```

### Running AI Tests

```bash
# With Ollama running in background, run AI tests
cargo test -p qir_ai --all-features -- --nocapture

# Run specific AI test
cargo test -p qir_ai --test draft -- --nocapture

# Run AI stress tests (requires Ollama)
cargo test -p qir_ai --test stress_large_embeddings --ignored --release -- --nocapture
```

### AI Tests That Require Ollama

| Test | Time | Models Tested |
|------|------|---------------|
| `boundaries.rs` | <1s | N/A (Ollama connectivity only) |
| `index_build.rs` | 10-30s | mistral, neural-chat |
| `draft.rs` | 10-30s | mistral, neural-chat |
| `evidence_context.rs` | 10-30s | mistral, neural-chat |
| `retrieval.rs` | 10-30s | mistral, neural-chat |
| `stress_large_embeddings.rs` | 2-5 min | Chosen model |

### Why AI Tests Are Separate

- **Ollama is optional**: IncidentReview works without AI
- **Tests require local Ollama**: Prevents CI/CD slowdown
- **Model downloads are slow**: First-time setup takes 5-10 minutes
- **Tests marked `#[ignore]`**: Run only with `--ignored` flag

## Edge Case Tests

### Unicode & Special Characters

```bash
cargo test -p qir_core --test edge_cases_unicode -- --nocapture
```

Tests:
- Emoji in incident titles (🚨, 🔥, 💥, etc.)
- Unicode usernames (José, 李, Μαρία, etc.)
- Multi-byte emoji (👨‍👩‍👧‍👦, 🏳️‍🌈)
- Mixed scripts (中文 + English + emoji)

### Malformed Data

```bash
cargo test -p qir_core --test edge_cases_malformed -- --nocapture
```

Tests:
- Missing required CSV columns
- Invalid RFC3339 timestamps
- Out-of-range percentages (>100, <0)
- Non-numeric values in numeric fields
- Slack transcript format mismatches
- Partial corruption (mixed valid/invalid lines)
- CSV with BOM, inconsistent column counts
- Null bytes and special characters

## Code Coverage

```bash
# Generate coverage report
cargo tarpaulin -p qir_core --out Html

# View in browser
open tarpaulin-report.html
```

## Linting and Code Quality

### Frontend Linting

```bash
# Run ESLint
pnpm lint

# Fix lint errors automatically
pnpm lint --fix
```

### Rust Formatting and Clippy

```bash
# Check formatting
cargo fmt -- --check

# Format code
cargo fmt

# Run Clippy (linter)
cargo clippy -p qir_core -- -D warnings
cargo clippy -p qir_ai -- -D warnings
```

## CI/CD Testing

### GitHub Actions Workflows

See `.github/workflows/` for automated testing:

- **ci.yml**: Runs on every PR and push to main
  - `pnpm lint`
  - `pnpm test` (frontend)
  - `cargo test -p qir_core` (backend)
  - `cargo test -p qir_ai --lib` (AI, no Ollama)
  - `pnpm tauri build` (verify DMG build)

### Running CI Locally

```bash
# Run the same checks as GitHub Actions
pnpm lint
pnpm test
cargo test -p qir_core
cargo test -p qir_ai --lib
pnpm tauri build
```

## Debugging Tests

### Verbose Output

```bash
# Show println! output from tests
cargo test -p qir_core -- --nocapture

# Show test names as they run
cargo test -p qir_core -- --nocapture --test-threads=1
```

### Debug a Failing Test

```bash
# Run single test with output
cargo test -p qir_core validate_and_metrics::test_timestamp_ordering -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test -p qir_core --test validate_and_metrics -- --nocapture

# Run with full backtrace
RUST_BACKTRACE=full cargo test -p qir_core --test validate_and_metrics -- --nocapture
```

### Frontend Test Debugging

```bash
# Run tests in watch mode for rapid iteration
pnpm test --watch

# Debug in Chrome DevTools
pnpm test --inspect-brk
```

## Test Fixtures and Golden Data

### Golden Fixtures

Located in `fixtures/`:

- **qir_v1_report.md**: Expected Markdown report output (snapshot testing)
- **demo_incidents.csv**: Sample Jira export with 10 incidents
- **slack_transcript_sample.txt**: Sample Slack transcript
- **sanitized_export_sample.json**: Anonymized dataset example
- **large_dataset.fixture**: Synthetic 10K incident fixture (auto-generated)

### Using Fixtures in Tests

```rust
#[test]
fn test_with_fixture() {
    let fixture = std::fs::read_to_string("fixtures/demo_incidents.csv").unwrap();
    let result = parse_jira_csv(&fixture, &profile).unwrap();
    assert_eq!(result.rows.len(), 10);
}
```

## Best Practices

### Writing Tests

1. **Name tests clearly**: `test_<feature>_<scenario>_<expected_result>`
   ```rust
   #[test]
   fn test_jira_csv_with_emoji_in_title() { ... }
   ```

2. **Test one thing**: Each test should verify one behavior
   ```rust
   #[test]
   fn test_metrics_computation_returns_positive_mttd() { ... }
   ```

3. **Use descriptive assertions**: Include expected vs actual
   ```rust
   assert_eq!(
       result.mttd,
       2.5,
       "MTTD should be 2.5 hours, got {}", result.mttd
   );
   ```

4. **Test both happy and sad paths**:
   - Happy: Valid data, expected behavior
   - Sad: Invalid data, error handling

### Test Organization

- Group related tests in modules
- Use setup functions for common initialization
- Keep fixtures in `fixtures/` directory
- Mark Ollama-dependent tests with `#[ignore]`

## Troubleshooting

### "Tests hang or timeout"

```bash
# Run with single thread
cargo test -p qir_core -- --test-threads=1

# Set timeout (Bash)
timeout 30 cargo test -p qir_core --test stress_large_dataset
```

### "Ollama test fails but Ollama is running"

```bash
# Verify Ollama is accessible
curl http://127.0.0.1:11434/api/tags

# Restart Ollama
pkill ollama
sleep 2
ollama serve
```

### "Edge case test fails on certain machines"

- Platform differences (macOS vs Linux)
- Unicode handling differences
- Timing-dependent tests (use relaxed bounds)

### "Stress test is slow"

- Run in release mode: `cargo test --release`
- Skip on slow machines: `#[ignore]` + document requirement
- Profile with `cargo profiling`

## Contributing Tests

When adding new features:

1. Write tests first (TDD)
2. Ensure tests pass: `pnpm test && cargo test -p qir_core`
3. Check coverage: `cargo tarpaulin`
4. Document test purpose and expected behavior
5. Add to appropriate test file or create new one
6. Update this guide if new test categories added

---

**Questions?** See README.md, PLANS.md, or [open an issue](https://github.com/saagar210/IncidentReview/issues).
