# UI-19 evidence

Build and capture:

- `scripts/capture-ui-19.sh`
- The script ran `npm run build` before capture.
- Both PNG files are 1280x800 at the application's 150% UI scale.
- The two PNG files have distinct SHA-256 hashes.
- Capture diagnostics reported nine visible Command Manager controls and nine
  controls meeting the automated contrast threshold in each scene.

## Literal descriptions

### `high-contrast-light-150-1280.png`

The light high-contrast text workspace shows the title bar, one-row Text
Command Manager, activity rail, Monaco editor, and status bar. Find has keyboard
focus. A complete three-pixel green rectangle appears inside the Find control
and a white tooltip below names the command. The active tab has a yellow focus
edge. Text, line numbers, controls, and status values remain within their
regions at 150% scale.

### `high-contrast-dark-150-1280.png`

The dark high-contrast scene has the same geometry on black and dark-grey
surfaces. Find retains the complete green focus rectangle and labelled tooltip.
The editor contains six visible lines. No title-bar, Command Manager, editor, or
status text overlaps another region.

## Automated checks (this environment)

- `scripts/capture-ui-19.sh` after `npm run build`: 150% scale, high-contrast theme,
  Find control focused, green focus ring, contrast diagnostic on nine visible CM controls.
- Keyboard path: demo token `ui19-focus-visible` focuses Find via `data-control-id`.
- Not run here: Windows 100/125/200%, Linux 2x scaling, axe-core, screen reader.

## Grade

UI-19 grade: Drift.

The captured high-contrast 150% keyboard-focus path has no clipping and passes
the command contrast diagnostic. Windows 100%, 125%, and 200%, Linux 2x, axe,
and a manual screen-reader pass were not run in this Linux 1x virtual display.
Those missing platform and assistive-technology checks prevent Pass.
