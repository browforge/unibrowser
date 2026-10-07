# Shell density evidence (Quiet Utility)

Build and capture:

- `scripts/capture-shell-density.sh` (runs `npm run build` first)
- `scripts/capture-vqa-common.sh` sets `UNIBROWSER_DOC_CAPTURE_SIZE=1280x800`
- Sixteen PNG files under `docs/screenshots/shell-density/` (eight scenes × light/dark)
- Capture diagnostics report title bar 40px + chrome region 80px combined height

## Literal spot checks

### New tab (`new-tab-light-1280.png`)

Start context Command Manager on a single row above a full-width Start page. Activity
rail collapsed. No side gutter between rail and content. Status bar one line.

### File browser (`file-browser-light-1280.png`)

Browse context with directory listing for the demo pack. List fills the work area
between rail and right edge. Chrome stack about 80px.

### Markdown / PDF / Image

Text, PDF Annotation, and Image contexts respectively. Editor or viewer body uses
the canvas background edge-to-edge; no centred narrow column. CM families match
context (Text, PDF Annotation, Image).

### Workshop (`workshop-light-1280.png`)

Project context above Sources | Stage | Inspector and timeline dock. Grid uses
full width at 1280px.

### Settings (`settings-light-1280.png`)

Settings context with search field and section list; page scrolls within the work
area without a floating centred card on grey gutters.

### Mod portal (`mod-portal-light-1280.png`)

Mods context, Mod portal tab title, full-width portal column (max grid ~1200px),
network status line, cards with badges and per-card actions.

## Grades

| Scene | Light | Dark | Notes |
| --- | --- | --- | --- |
| New tab | Pass | Pass | ~80px chrome, full-width Start |
| File browser | Pass | Pass | Browse CM, list uses width |
| Markdown | Pass | Pass | Text CM, Monaco flush |
| PDF | Pass | Pass | Annotation CM |
| Image | Pass | Pass | Image adjust CM |
| Workshop | Pass | Pass | Project CM, NLE grid |
| Settings | Pass | Pass | Settings CM, no gutter card |
| Mod portal | Pass | Pass | Mods CM, portal density |

Overall shell-density grade: **Pass** for the captured 1280×800 matrix. Remaining
Drift vs mockups is unmeasured pixel diff (no automated overlay in this run).
