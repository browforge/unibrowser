# Report screenshots (`docs/report-shots/`)

Placeholder folder for **Wednesday report** captures. Pair with [report-visuals.md](../report-visuals.md) and `./scripts/capture-report-screenshots.sh`.

Automated output from the capture script still lands in `./report-screenshots/` (gitignored). Copy finished PNGs here when you want them versioned with the repo.

## Intended shots

| File (placeholder) | Scene | Notes |
| --- | --- | --- |
| `01-ntp.png` | New tab page | Hero, recent files, browse actions |
| `02-files-folder.png` | Folder tab | Breadcrumbs, sort/filter, one PDF or image row |
| `03-command-palette.png` | Command palette | `Ctrl+Shift+P`, chord visible in placeholder |
| `04-settings.png` | `unibrowser://settings` | Theme + one distinctive preference block |
| `05-unibrow-project.png` | Notes template project | Board nodes + timeline strip |
| `06-epub-or-docx.png` | EPUB or DOCX viewer | Chapter nav or in-document search |
| `07-command-manager.png` | Command Manager | Floating bar on second monitor (optional) |
| `08-status-bar.png` | Status bar detail | Zoom controls + font pt on a text or reader tab |
| `09-downloads.png` | `unibrowser://downloads` | History row + retry/clear affordances |
| `10-diagnostics.png` | `unibrowser://diagnostics` | Mod list + WebView stack line |

Use **1920×1080** or a consistent laptop resolution; keep light or dark theme consistent across the set.

## Capture

```bash
./scripts/capture-report-screenshots.sh --guide-only
```

Interactive capture (graphical session):

```bash
./scripts/capture-report-screenshots.sh
```

Only capture after `cargo test --workspace` and `npm run build` pass.
