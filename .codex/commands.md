# IncidentReview .codex command map

| Action | Command | Source |
| --- | --- | --- |
| setup deps | `pnpm install` | `README.md` |
| lint | `pnpm lint` | `README.md`, `package.json` |
| typecheck | `pnpm typecheck` | `package.json` |
| test (frontend) | `pnpm test` | `README.md`, `package.json` |
| test coverage | `pnpm test:coverage` | `package.json` |
| integration tests | `pnpm test:integration` | `package.json` |
| contract tests | `pnpm test:contracts` | `package.json` |
| e2e smoke tests | `pnpm test:e2e:smoke` | `package.json` |
| docs generate | `pnpm docs:generate` | `package.json` |
| docs drift check | `pnpm docs:check` | `package.json` |
| policy gate (tests/docs) | `pnpm policy:require-tests-docs` | `package.json` |
| test (rust core) | `cargo test -p qir_core --all-features` | `.codex/verify.commands` |
| test (rust ai) | `cargo test -p qir_ai --all-features` | `.codex/verify.commands` |
| build | `pnpm tauri build` | `README.md` |
| lean dev | `pnpm run dev:lean` | `README.md`, `package.json` |
