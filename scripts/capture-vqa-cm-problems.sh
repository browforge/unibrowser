#!/usr/bin/env bash
# VQA evidence for populated Command Manager and Problems panel.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/vqa"

capture_vqa "$OUT" "cm-problems-markdown-light" \
  "capture-ready,theme-light,ai-sidebar,document-mockup-problems,rail-collapsed" \
  "/workspace/examples/demo-pack/sample.md"
capture_vqa "$OUT" "cm-problems-markdown-dark" \
  "capture-ready,theme-dark,ai-sidebar,document-mockup-problems,rail-collapsed" \
  "/workspace/examples/demo-pack/sample.md"
capture_vqa "$OUT" "cm-problems-typescript-light" \
  "capture-ready,theme-light,ai-sidebar,document-mockup-problems,rail-collapsed" \
  "/workspace/examples/demo-pack/sample.ts"
capture_vqa "$OUT" "cm-problems-typescript-dark" \
  "capture-ready,theme-dark,ai-sidebar,document-mockup-problems,rail-collapsed" \
  "/workspace/examples/demo-pack/sample.ts"
capture_vqa "$OUT" "cm-problems-pdf-light" \
  "capture-ready,theme-light,ai-sidebar,document-mockup-problems,rail-collapsed" \
  "/workspace/examples/demo-pack/sample.pdf"
capture_vqa "$OUT" "cm-problems-pdf-dark" \
  "capture-ready,theme-dark,ai-sidebar,document-mockup-problems,rail-collapsed" \
  "/workspace/examples/demo-pack/sample.pdf"

echo "CM + Problems captures complete: $OUT"
