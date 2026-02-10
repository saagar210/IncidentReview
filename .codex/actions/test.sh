#!/usr/bin/env bash
set -euo pipefail
pnpm test
cargo test -p qir_core
cargo test -p qir_ai

