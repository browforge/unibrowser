# UI-20 evidence

Verification:

- `npm run build` passed.
- `cargo test --workspace` passed after the shared WASM fixture tests were
  serialised.
- `scripts/self-test.sh` passed 3 of 3 runs. Each run completed 11 scenarios,
  including shell chrome, unknown-file support, the project workspace, LIVE,
  and docked AI.
- `scripts/capture-ui-15.sh` ran `npm run build` before producing four final
  workshop PNG files.
- The final PNG files have the expected 1600x1000 and 1280x800 dimensions and
  no duplicate SHA-256 hashes.

## Literal descriptions

### `unibrow-create-light-1600.png`

The light-theme project workspace has the Project Command Manager above one
joined Sources, Stage, and Inspector grid. The Stage contains a centred 16:9
title preview. A timeline dock spans the bottom with a ruler, one playhead, and
Video/Image, Title, and Audio/MIDI tracks. Four duration-proportional clips are
visible. The selected title clip has a blue edge and its two numeric timing
fields are in Inspector. No region overlaps or clips another.

### `unibrow-create-dark-1280.png`

The dark-theme 1280 workspace keeps the same structure at the narrower width.
The Sources labels, stage title, Inspector values, track names, clip labels,
playhead, and status values remain readable. Lower-priority Project commands
move into overflow rather than wrapping. No region overlaps or clips another.

## Grade

UI-20 grade: Drift.

The active shell has no legacy duplicate chrome according to 3 of 3 shell
audits, the superseded workshop shell CSS was removed, and the final matrix
passes the visible integrity gates. The inactive `renderUnibrowLegacy`
implementation and additional old `.unibrow-*` timeline rules still remain in
`documentViewer.ts` and `styles.css`. That incomplete source cleanup prevents
Pass even though the legacy renderer is not mounted.
