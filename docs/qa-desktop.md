# Desktop manual QA

Automated gate:

```bash
./scripts/smoke-desktop-qa.sh
```

The script runs `cargo test --workspace` and `npm run build`, then prints the checklist for a Wednesday-style report.

## Preferences (Recent panel)

| Setting | Default | Check |
| --- | --- | --- |
| Restore open tabs on launch | Off | Enable → restart → tabs return; disable → fresh home |
| Auto-save edited files | Off | Enable + interval → edit `.txt`/SVG → toast after interval |
| Downloads | Open in new tab | Switch to **Save to disk** → demo download → save dialog / saved path toast, no new tab |

## Downloads

- **Open in new tab**: Demo download or `unibrowser-download` from web content opens Example (or target) in a tab.
- **Save to disk**: Same flows write under chosen path; Recent preference persists in config JSON.
- See [downloads.md](./downloads.md) for Linux WebKit vs Windows WebView2 status.

## `.unibrow` board

- New project seeds **Notes** and **References** on the board (`project.json` `x`/`y`).
- Drag cards on the board; positions persist after refresh.
- List below still supports Open / Rename / Remove and add flows.

## Command manager

- Drag commands between toolbar and floating bars (existing).
- **Export layout** / **Import layout** → JSON round-trip; palette commands still resolve after import.

## Bookmarks, compare, session

- Bookmark bar: Home, Downloads, New project; Pin adds current location.
- Compare: text file → Compare → second file → Monaco diff → Close compare.
- Session restore opt-in (see table above).

## Spellcheck

Monaco in the desktop shell does not ship with a spellchecker extension. Enabling real spellcheck needs either a Monaco contribution (dictionary + markers) or moving plain prose to a native control. **Deferred** until extension-host or a small bundled dictionary mod is justified.
