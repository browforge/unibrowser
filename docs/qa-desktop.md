# Desktop manual QA

Automated gate:

```bash
./scripts/smoke-desktop-qa.sh
```

The script runs `cargo test --workspace` and `npm run build`, then prints the checklist for a Wednesday-style report.

## Settings (`unibrowser://settings`)

Open from the bookmark bar, Recent → **Open settings**, or the omnibox. All prefs save inline (no modal).

| Setting | Default | Check |
| --- | --- | --- |
| Theme | System | Light / dark / system updates chrome immediately |
| Downloads | Open in new tab | Save to disk → demo download → file saved, no new tab |
| Auto-save | Off | Enable + interval → edit `.txt`/SVG → toast after interval |
| Restore session | Off | Enable → restart → tabs return |
| Board snap to grid | Off | Enable → drag `.unibrow` cards snap to 24px grid |
| Crash recovery | Off | Enable → edit text/SVG → backups list on Settings/About; Restore / Dismiss (inline, no modal) |
| Workspace layout | (none) | Settings → save preset → includes bookmarks + split → Load restores window, toolbars, pins, split |

## Downloads

- **Open in new tab**: Demo download or `unibrowser-download` from web content opens Example (or target) in a tab.
- **Save to disk**: Same flows write under chosen path; Recent preference persists in config JSON.
- See [downloads.md](./downloads.md) for Linux WebKit vs Windows WebView2 status.

## `.unibrow` board and timeline

- New project seeds **Notes** and **References** on the board (`project.json` `x`/`y`) plus a **timeline** clip list stub.
- Alt+drag pan, Ctrl+wheel zoom; double-click card opens asset; drag moves cards (optional grid snap in Settings).
- Timeline: add clip from asset, remove, reorder (↑↓ or drag clip row); type badge (TXT/IMG/…); persisted in `project.json`.
- List below still supports Open / Rename / Remove and add flows.

## MIDI

- Open a `.mid` / `.midi` file: viewer shows SMF format, track count, ticks per quarter, and first note-on events (scrub stub).
- `.unibrow` timeline: scrub a **MIDI** clip — note table + status bar hint (“MIDI stays MIDI until encode”); optional scrub beep in Settings → Media.

## Accessibility

- Command Manager tab row: arrows, Home/End, Enter/Space ([ux-a11y.md](./ux-a11y.md)).
- Command palette: **Escape** closes.

## Export / print

- DOCX / Markdown: **Export print HTML** or print preview (printer-friendly CSS).
- `.unibrow` timeline: **Export MP4 (H.264)** with fast/balanced preset; progress toasts when ffmpeg runs.

## VSIX

- With an extension installed: Command Manager → **Extensions** tab; run a contributed command (stub message).
- Web tab: Increase/decrease font persists per origin in `web-font-prefs.json`; reload tab restores size.
- Settings / About: crash recovery list + restore (debounced writes under config `crash-recovery/`).
- `unibrowser://about` shows Apache-2.0; run `./scripts/generate-notice.sh` for NOTICE.md.
- Large PDF: set `UNIBROWSER_PROFILE_OPEN=1` and open a PDF; stderr logs resolve+open timing.
- PDF viewer: canvas thumbnails (first 12 eager, rest lazy on scroll).
- DOCX: Find / Prev / Next search in Mammoth HTML preview.
- PNG/image: Crop overlay; **Save cropped as…** writes new PNG; original unchanged.
- Markdown / DOCX preview: `https://` and `file://` links navigate via omnibox (blocked: `javascript:`, `data:`).
- Settings → **Open another window** (independent tabs per window; shared prefs on disk).
- Undo stacks capped per document and total tracked paths; large read/sync limits enforced.
- About: Privacy section states no telemetry by default.

## Command manager

- Drag commands between toolbar and floating bars (existing).
- **Export layout** / **Import layout** → JSON round-trip; palette commands still resolve after import.

## Bookmarks, compare, session

- Bookmark bar: Home, Downloads, New project; Pin adds current location.
- Compare: text file → Compare → second file → Monaco diff → Close compare.
- Session restore opt-in (see table above).

## Spellcheck

Monaco has no built-in dictionary. **Advanced → Experimental browser spellcheck** mirrors editor text in a read-only `textarea` with native `spellcheck` (en-NZ). SVG source uses the same flag when enabled. Full in-editor markers remain future work.

## OCR

Image viewer **Extract text (OCR)** is disabled until a Tesseract sidecar ships ([ocr-sidecar.md](./ocr-sidecar.md)).
