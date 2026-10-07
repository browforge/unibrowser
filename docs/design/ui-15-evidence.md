# UI-15 evidence

Captured with `scripts/capture-ui-15.sh` after `npm run build` and release
desktop rebuild. One 1600x1000 PNG, light theme, standard contrast, 100% scale,
compact Command Manager, collapsed activity rail. Demo state `ui15-unibrow-title`
focuses the title board card (`node-1`). Hash and dimension checks pass.

## Literal image descriptions

### `unibrow-create-light-1600.png`

1600x1000. Tab `sample.unibrow` with Save in the title bar. Collapsed **Live
preview** disclosure at the top (studio header reads Preview when closed).

Joined **Create** workbench grid:

- **Sources (left):** heading Sources; list of four project nodes (Unibrowser
  title, Logo, Sample page, Chime) with type and asset hints; add-asset controls
  and drop target in the rail.
- **Preview (centre):** recessed grid canvas with four board cards; **Unibrowser**
  title card selected (blue outline); pan/zoom hint above the well.
- **Inspector (right):** Label Unibrowser, Type title, Asset —, Position 40 × 48,
  Title text Unibrowser.

**Timeline (bottom dock):** ruler 0–8s, playhead scrubber, markers, clip blocks on
the track, and per-clip rows (Title, Logo, PDF, MIDI) with timing fields.

Below the timeline: export/share/encode actions (contextual export; global Save
remains in the title bar). Status bar shows project summary.

Command Manager still follows **Browse** while the path is the project folder
(no dedicated project family yet).

## Grade

| Evidence | Result | Basis |
| --- | --- | --- |
| Create workspace 1600 | Drift | Sources / preview / inspector grid and bottom timeline match UI-15; title card selected in inspector; Browse CM on project tab and export stack below timeline are known gaps. |

Totals: 0 Pass, 1 Drift, 0 Fail.
