# UI-15 evidence

Captured with `scripts/capture-ui-15.sh`. The script runs `npm run build`
before capture, rebuilds the embedded release frontend, enforces exact
dimensions, and rejects duplicate hashes. Four PNGs cover 1600x1000 and
1280x800 in light and dark themes.

## Literal image descriptions

### `unibrow-create-light-1600.png`

Top chrome says **Project** and exposes Add asset, Add title, Split, Delete clip,
Snap, Marker, Timeline zoom, Preview, LIVE, Record, and Export. Save remains in
the title bar. The fixed workbench has Sources left, a large centred 16:9 Stage,
Inspector right, and Timeline across the bottom. Stage shows the selected Title
composition with white `Unibrowser` text. Inspector contains only selected clip
Start and Duration fields. Timeline has one ruler, one playhead, and Video /
Image, Title, and Audio / MIDI tracks with blocks sized by duration. No region
overlaps or clips.

### `unibrow-create-dark-1600.png`

Same geometry and content in dark theme. Source names, programme title, clip
labels, duration labels, track names, and Inspector fields remain readable.

### `unibrow-create-light-1280.png`

At 1280x800 the Sources and Inspector rails narrow, Stage remains 16:9, and the
Timeline dock is shorter. Add asset and Add title move into CM overflow while
the Project family remains active. All four source rows and three tracks remain
visible.

### `unibrow-create-dark-1280.png`

Same compact geometry in dark theme. The title clip remains selected by a blue
outline; white clip text and Stage title remain readable. There is no horizontal
or vertical region overlap.

## Grade

| Evidence | Result | Basis |
| --- | --- | --- |
| Create light 1600 | Pass | Correct Project CM, large Stage, joined rails, NLE tracks, no overlap or clipping. |
| Create dark 1600 | Pass | Same geometry with readable dark-theme content. |
| Create light 1280 | Pass | Responsive rails and timeline; Project CM overflows lower-priority Add controls without clipping. |
| Create dark 1280 | Pass | Compact dark workbench remains readable and in flow. |

Totals: 4 Pass, 0 Drift, 0 Fail.
