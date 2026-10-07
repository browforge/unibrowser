# Shell density evidence (Quiet Utility)

Capture: `scripts/capture-shell-density.sh` (runs `npm run build`, `npm run test:contrast`, release rebuild, 16×1280×800 PNGs).  
Doc-capture aborts if `validateShellDensityContrastForCapture()` fails (computed AA ≥ 4.5:1 on workshop headers/labels/badges and mod meta).

## Literal checks (legibility, overlap, clipping)

### `workshop-light-1280.png`

Panel headers **Sources**, **Stage**, **Inspector**, and **Timeline** use **#202223** on **#f1f0ec** (readable, not pale-on-pale). Track labels (**Video / Image**, **Title**, **Audio / MIDI**) and Sources badges (**TTL**, **IMG**, **PDF**, **MID**) use **#3d4448** on light panels; path sub-labels (**logo.svg**, etc.) use **#454b4e**. Stage preview and timeline clips unchanged; no overlap with status bar.

**Grade: Pass** (header/label legibility)

### `workshop-dark-1280.png`

Headers and track labels remain **#e9edef / #c9d0d4** on **#22272b** panels (dark theme unchanged). No regression vs prior dark capture.

**Grade: Pass**

### `mod-portal-light-1280.png`

Discover catalogue uses a **16px** grid gap; cards no longer share a double border. Portal padding **16px** left/right so the right column is inset from the scroll edge. Suggestion card **Browse installable mods** has a compact footer (metadata then **Open**, no large dead band).

**Grade: Pass**

### `mod-portal-dark-1280.png`

Three-column grid with visible gutters; card text contrast unchanged.

**Grade: Pass**

### Remaining scenes (unchanged tokens)

| PNG | Legibility / layout note | Grade |
| --- | --- | --- |
| `new-tab-light-1280.png` | Start CM and recent badges; downloads above status | Pass |
| `new-tab-dark-1280.png` | Same | Pass |
| `file-browser-light-1280.png` | Browse list full width | Pass |
| `file-browser-dark-1280.png` | Same | Pass |
| `markdown-light-1280.png` | Monaco flush | Pass |
| `markdown-dark-1280.png` | Same | Pass |
| `pdf-light-1280.png` | PDF workspace | Pass |
| `pdf-dark-1280.png` | Same | Pass |
| `image-light-1280.png` | Image viewer | Pass |
| `image-dark-1280.png` | Same | Pass |
| `settings-light-1280.png` | Settings list | Pass |
| `settings-dark-1280.png` | Same | Pass |

**Overall: Pass** for the 1280×800 matrix.

## Automation

- `npm run test:contrast` — token pairs for light, dark, and HC workshop surfaces (≥ 4.5:1).
- Doc-capture — live `getComputedStyle` sampling on workshop / mod portal surfaces.
