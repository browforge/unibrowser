# UI-16 evidence

Captured with `scripts/capture-ui-16.sh`. The script runs `npm run build`
before capture, rebuilds the embedded release frontend, enforces exact
dimensions, and rejects duplicate hashes. Eight PNGs cover LIVE and Recording
at 1600x1000 and 1280x800 in light and dark themes.

## Literal image descriptions

### `unibrow-live-on-light-1600.png`

Project CM remains fixed. The same Sources / Stage / Inspector / Timeline grid
is visible. Stage header says **LIVE** with a restrained red rule and the Stage
says `Live programme`. Inspector switches from clip timing to Source health and
Mixer. Timeline geometry is unchanged and does not overlap the Stage.

### `unibrow-live-recording-light-1600.png`

Same grid and dimensions. Stage header says **REC** in a dark red label and the
Stage says `Recording programme`. Source health and Mixer remain in Inspector.

### `unibrow-live-on-dark-1600.png`

Dark theme version of LIVE. White programme text, green health/meter marks,
track labels, and clip text remain readable. No stacked live block exists.

### `unibrow-live-recording-dark-1600.png`

Dark theme Recording state. `REC` and `Recording programme` differ in text and
emphasis from LIVE while preserving identical geometry.

### `unibrow-live-on-light-1280.png`

Compact light LIVE layout. Stage remains 16:9 between narrow rails; all three
timeline tracks fit below. Project CM uses overflow without clipping.

### `unibrow-live-recording-light-1280.png`

Compact light Recording layout. `REC` is visible in the Stage header and
`Recording programme` is centred in the programme frame. Inspector remains
fully visible.

### `unibrow-live-on-dark-1280.png`

Compact dark LIVE layout. Source names, Stage state, Mixer, and timeline clips
are readable with no overlap.

### `unibrow-live-recording-dark-1280.png`

Compact dark Recording layout. Same fixed grid; REC state is visible and
distinct; no content is hidden behind another region.

## Grade

| State | Result | Basis |
| --- | --- | --- |
| LIVE light 1600 | Pass | In-grid live Stage, health/mixer Inspector, NLE timeline, no overlap. |
| Recording light 1600 | Pass | REC text and stronger state label; identical geometry. |
| LIVE dark 1600 | Pass | Readable dark surfaces and restrained LIVE state. |
| Recording dark 1600 | Pass | Recording state readable and distinct from LIVE. |
| LIVE light 1280 | Pass | Responsive fixed grid with Project CM and no clipping. |
| Recording light 1280 | Pass | REC state and Inspector remain visible. |
| LIVE dark 1280 | Pass | Compact dark layout remains readable. |
| Recording dark 1280 | Pass | Compact REC layout has no overlap or clipping. |

Totals: 8 Pass, 0 Drift, 0 Fail.

## Notes

Demo tokens seed visual workshop state without mutating the live backend. Normal
Project CM LIVE and Record commands still invoke the local live APIs.
