#!/usr/bin/env bash
# Unibrowser desktop manual QA checklist (Wed report).
# Run automated gates first, then walk through sections in the Tauri app.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "== Automated gates =="
cargo test --workspace
(cd crates/unibrowser-desktop && npm run build)
echo "OK: tests + UI build"

cat <<'CHECKLIST'

== Manual smoke (check each) ==

Launch
  [ ] App opens; home folder or restored session (if opt-in enabled)
  [ ] New tab (+) shows friendly start page (files, recent, new project)
  [ ] Bookmark bar: Home, Downloads, New project; Pin adds current location
  [ ] Compare: open .txt, Compare → pick second file → diff view → Close compare
  [ ] Recent panel: auto-save checkbox off by default; enable only if you want it
  [ ] Status bar: click path to copy; Ctrl+Shift+V paste path in omnibox
  [ ] Status bar: encoding (text), line/col (text), zoom (PDF/image/web)
  [ ] Ctrl+K then Ctrl+S saves text (sequence sketch)

Tabs and files
  [ ] Open .txt in Monaco; edit shows dirty dot; close asks to save
  [ ] Ctrl+S saves; Recent lists the file
  [ ] Drag-drop files onto window opens tabs
  [ ] Tab search (Ctrl+Shift+T) finds open tabs

Viewers and zoom
  [ ] PDF: page nav, zoom +/-, Copy selection, print
  [ ] CSV: Copy table button
  [ ] PNG/image: zoom, pan, rotate
  [ ] https:// page: web zoom +/- changes page text size
  [ ] Markdown preview + print preview

Shell
  [ ] Omnibox autocomplete; split view; theme cycle
  [ ] Command palette Ctrl+Shift+P; toasts on errors
  [ ] Session restore checkbox (off by default); quit/relaunch when enabled

Formats (spot check)
  [ ] CSV large file: capped rows + status line
  [ ] EPUB chapter + print; DOCX read-only preview

Mods (optional)
  [ ] Mod portal opens; VSIX install disabled without network opt-in

Spellcheck
  [ ] Deferred: Monaco has no built-in spellcheck; browser textarea spellcheck N/A for editor (see docs/qa-desktop.md)

CHECKLIST
