#!/usr/bin/env bash
set -euo pipefail

echo "IncidentReview workspace setup (non-destructive)."
echo
echo "Tooling checks:"
command -v node >/dev/null 2>&1 && node -v || echo "node: missing"
command -v pnpm >/dev/null 2>&1 && pnpm -v || echo "pnpm: missing"
command -v rustc >/dev/null 2>&1 && rustc --version || echo "rustc: missing"
command -v cargo >/dev/null 2>&1 && cargo --version || echo "cargo: missing"
command -v xcodebuild >/dev/null 2>&1 && xcodebuild -version || echo "xcodebuild: missing"
command -v ollama >/dev/null 2>&1 && ollama --version || echo "ollama: missing"
echo
echo "Next commands (from /Users/d/Projects/IncidentReview/README.md):"
echo "  pnpm install"
echo "  pnpm tauri dev"

