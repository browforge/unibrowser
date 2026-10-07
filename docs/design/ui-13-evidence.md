# UI-13 evidence

Captured with `scripts/capture-ui-13.sh` after the 2026-10-07 drift pass. The
script uses real theme, contrast, and UI scale preferences, requires the
requested full-window dimensions, and rejects duplicate PNG hashes. Each image
below was opened after the final capture (frontend dist rebuilt before capture).

## Literal image descriptions

### `files-browser-preview-light-100.png`

1600x1000, light theme, standard contrast, 100% scale. Top: `demo-pack` tab,
folder path, Save, and 80 px title plus Command Manager labelled Browse (New
folder through Delete visible; Sort, Direction, Hidden files, Set workspace, and
Clear workspace in the overflow chevron). Left: collapsed activity rail and Places
with recent files on single lines (ellipsis where long). Centre: quiet
breadcrumbs without a raised Set workspace control; light bordered filter field;
selection note; file table with sortable Name, Size, and Date modified headers
(Size and Modified right-aligned); type badge and name left-aligned on one row
per file (no inline PNG thumbnails). `sample.svg` is selected. Right: joined Preview
dock with SVG thumb, metadata, path, and Open. Status bar: Browse context.

### `files-browser-preview-dark-100.png`

Same Browse layout on joined dark surfaces. Recent list stays one item per line.
No duplicate sort or hidden-files row in the content pane.

### `files-browser-preview-hc-100.png`

Same structure with high-contrast borders, white content, and yellow focus on the
active tab. Filter field uses a light surface and defined border.

### `files-browser-preview-light-150.png`

150% interface scale widens Places and Preview; table shows fewer rows. Metadata
path wraps; Open stays visible.

### `markdown-actions-light-100.png`

`sample.md` with Text Command Manager and flat Preview / Export HTML / Print
preview strip above Monaco (pixel gate visible).

### `settings-developer-*`

Unchanged from prior pass: Settings-only CM, category rail, Developer inventory,
quiet controls, and theme or scale variants as named in each filename.

## Grade against Quiet Utility and design rules

| Evidence | Result | Basis |
| --- | --- | --- |
| File browser light 100% | Pass | Browse CM is the only sort/hidden/workspace control surface; names beside badges; uniform rows; Places ellipsis list; quiet filter and breadcrumbs. |
| File browser dark 100% | Pass | Same Browse shell on joined dark surfaces. |
| File browser high contrast 100% | Pass | Distinct HC borders and focus; light filter field. |
| File browser light 150% | Pass | Real 150% scale without clipped Open or labels. |
| Markdown actions light 100% | Pass | Flat contextual actions and Monaco gate. |
| Settings captures (7) | Pass | Prior grades stand; captures re-run with same integrity checks. |

Totals: 11 Pass, 0 Drift, 0 Fail for this pass.
