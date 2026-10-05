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
| Workspace layout | (none) | Settings → save named preset → resize window / rearrange toolbars → Load restores both |

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
- Web tab: Increase/decrease font persists per origin in `web-font-prefs.json`; reload tab restores size.
- Settings / About: crash recovery list + restore (debounced writes under config `crash-recovery/`).
- `unibrowser://about` shows Apache-2.0; run `./scripts/generate-notice.sh` for NOTICE.md.
- Large PDF: set `UNIBROWSER_PROFILE_OPEN=1` and open a PDF; stderr logs resolve+open timing.
- PDF viewer: canvas thumbnails (first 12 eager, rest lazy on scroll).
- DOCX: Find / Prev / Next search in Mammoth HTML preview.
- PNG/image: Crop overlay (non-destructive); Clear crop resets overlay.
- About: Privacy section states no telemetry by default.

## Command manager

- Drag commands between toolbar and floating bars (existing).
- **Export layout** / **Import layout** → JSON round-trip; palette commands still resolve after import.

## Bookmarks, compare, session

- Bookmark bar: Home, Downloads, New project; Pin adds current location.
- Compare: text file → Compare → second file → Monaco diff → Close compare.
- Session restore opt-in (see table above).

## Spellcheck

Monaco in the desktop shell does not ship with a spellchecker extension. Enabling real spellcheck needs either a Monaco contribution (dictionary + markers) or moving plain prose to a native control. **Deferred** until extension-host or a small bundled dictionary mod is justified.
