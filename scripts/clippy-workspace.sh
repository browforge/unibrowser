#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Strict on libraries added in the blitz hardening slice.
cargo clippy -p unibrowser-terminal -p unibrowser-ocr -p unibrowser-docx -p unibrowser-project -- -D warnings
# Full desktop clippy (legacy warnings) — not part of CI gate yet.
# cargo clippy -p unibrowser-desktop
