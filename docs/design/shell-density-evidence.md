# Shell density evidence (Quiet Utility)

Capture: `scripts/capture-shell-density.sh` at **1280×800** (light + dark).  
Commit **e307a44** (layout/icons) plus follow-up CSS; PNGs refreshed after release rebuild.

## Literal checks (overlap/clipping/contrast)

### `new-tab-light-1280.png`

Start page with compact CM (File, Folder, Project, Recent with distinct Lucide icons). Recent rows use file-list badges (SVG, DOC, PDF, TS, ?, IMG, MD, TXT) with a single time column (no duplicate type). Downloads section sits above the status bar with bottom padding; both download rows and Open actions are fully visible. No overlap; path subtext ellipsizes on one line.

**Grade: Pass**

### `new-tab-dark-1280.png`

Same structure on dark canvas; badges and row spacing match light. Downloads and status bar do not overlap content.

**Grade: Pass**

### `file-browser-light-1280.png`

Browse context CM; demo-pack directory list uses full width between collapsed rail and right edge. Rows do not collide with chrome or status bar.

**Grade: Pass**

### `file-browser-dark-1280.png`

Same as light; list and chrome stack (~80px) unchanged.

**Grade: Pass**

### `markdown-light-1280.png`

Text CM over Monaco; editor flush to work area. No gutter card; status bar clear.

**Grade: Pass**

### `markdown-dark-1280.png`

Same; dark editor surface edge-to-edge.

**Grade: Pass**

### `pdf-light-1280.png`

PDF Annotation CM; page workspace fills width. No control overlap on canvas.

**Grade: Pass**

### `pdf-dark-1280.png`

Same layout on dark theme.

**Grade: Pass**

### `image-light-1280.png`

Image adjust CM; sample image centred in viewer without clipping.

**Grade: Pass**

### `image-dark-1280.png`

Same on dark theme.

**Grade: Pass**

### `workshop-light-1280.png`

Project CM with bookmark/play/circle-dot mode icons. Sources | Stage | Inspector over timeline dock; three tracks visible with internal timeline scroll. Audio/MIDI row sits immediately above the status bar (tight but not clipped). No stray border beside PREVIEW after stage-head cleanup.

**Grade: Pass** (timeline dock snug to status bar)

### `workshop-dark-1280.png`

Dark workshop grid; timeline tracks and stage preview readable; same dock height behaviour as light.

**Grade: Pass**

### `settings-light-1280.png`

Settings CM; search and section list scroll in work area without floating gutter card.

**Grade: Pass**

### `settings-dark-1280.png`

Same on dark theme.

**Grade: Pass**

### `mod-portal-light-1280.png`

Mods CM; Discover tab with three-column responsive catalog (`auto-fill`, min ~360px) and 12px gaps. Card footers stack metadata then action (no button on metadata). Light-theme meta/help text uses #454b4e on card/canvas (AA). No right-edge card clipping at 1280.

**Grade: Pass**

### `mod-portal-dark-1280.png`

Three-column catalog with spaced cards; badges and actions separated. Contrast adequate on dark cards.

**Grade: Pass**

## Matrix summary

| Scene | Light | Dark |
| --- | --- | --- |
| New tab | Pass | Pass |
| File browser | Pass | Pass |
| Markdown | Pass | Pass |
| PDF | Pass | Pass |
| Image | Pass | Pass |
| Workshop | Pass | Pass |
| Settings | Pass | Pass |
| Mod portal | Pass | Pass |

**Overall: Pass** for the 1280×800 matrix (no overlap/clipping failures in recaptured PNGs).

## Automation

- `npm run test:icons` — every `families.rs` control `icon` string maps in `CONTROL_ICON_MAP` (no generic fallback).
