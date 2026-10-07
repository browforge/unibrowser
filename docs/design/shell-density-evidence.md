# Shell density evidence (Quiet Utility)

Capture: `scripts/capture-shell-density.sh` (`npm run test:contrast`, release rebuild, 16×1280×800).  
Doc-capture runs `validateShellDensityContrastForCapture()` (AA ≥ 4.5:1 on scoped labels) and `validateModCardTextOverlap()` on mod cards.

## Literal checks (legibility, overlap, clipping)

### `mod-portal-light-1280.png`

Discover tab: suggestion card **Browse installable mods** shows **mod id** on one line and the **reason** (`.ubunknown` support copy) on the next; no stacked garble. Catalogue grid keeps **16px** gutters; right inset matches left padding. Meta/actions on catalogue cards remain separated.

**Grade: Pass**

### `mod-portal-dark-1280.png`

Same card stacking and grid gutters on dark canvas.

**Grade: Pass**

### `workshop-light-1280.png`

Panel headers and track labels remain **#202223 / #3d4448** on light panels (readable). Inspector field labels (**Clip label**, **Type**, **Title track**, timing fields, **Title text**) use **`--ub-label` (#454b4e)** on **#f1f0ec**. Title-bar breadcrumb uses **chrome text** on the dark title strip (not body text colour).

**Grade: Pass**

### `workshop-dark-1280.png`

Inspector labels use `--ub-label` / `--ub-text-2` on dark panels; headers unchanged from prior pass.

**Grade: Pass**

### Remaining matrix (1280×800)

| PNG | Legibility / layout | Grade |
| --- | --- | --- |
| `new-tab-light-1280.png` | Start rows, badges, downloads clearance | Pass |
| `new-tab-dark-1280.png` | Same | Pass |
| `file-browser-light-1280.png` | `.files-meta` uses `--ub-label` | Pass |
| `file-browser-dark-1280.png` | Same | Pass |
| `markdown-light-1280.png` | Editor flush | Pass |
| `markdown-dark-1280.png` | Same | Pass |
| `pdf-light-1280.png` | PDF workspace | Pass |
| `pdf-dark-1280.png` | Same | Pass |
| `image-light-1280.png` | Image viewer | Pass |
| `image-dark-1280.png` | Same | Pass |
| `settings-light-1280.png` | `.settings-content label` uses `--ub-label` | Pass |
| `settings-dark-1280.png` | Same | Pass |

**Overall: Pass**

## Automation

- `npm run test:contrast` — token pairs + `rectsOverlap` unit test.
- Doc-capture — computed contrast on workshop/files/settings labels when present; mod-card sibling text overlap check when mod portal is open.
