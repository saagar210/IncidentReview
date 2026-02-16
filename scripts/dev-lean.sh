#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -z "${TMPDIR:-}" ]]; then
  TMP_BASE="/tmp"
else
  TMP_BASE="${TMPDIR%/}"
fi

EPHEMERAL_TARGET_DIR="${TMP_BASE}/incidentreview-target-${USER:-user}"
export CARGO_TARGET_DIR="${EPHEMERAL_TARGET_DIR}"

cleanup() {
  if [[ "${CARGO_TARGET_DIR}" == "${TMP_BASE}/incidentreview-target-"* ]]; then
    rm -rf "${CARGO_TARGET_DIR}"
  fi
}

trap cleanup EXIT INT TERM

echo "Running Tauri dev with temporary Rust build cache: ${CARGO_TARGET_DIR}"
echo "Build cache will be removed automatically when this process exits."

cd "${ROOT_DIR}"
pnpm tauri dev
