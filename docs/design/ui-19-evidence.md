# UI-19 evidence

Build and capture:

- `scripts/capture-ui-19.sh` (150% high-contrast keyboard-focus baseline)
- `scripts/capture-ui-19-multiscale.sh` (100%, 125%, 150%, 200% × light/dark)
- Both scripts run `npm run build` and `ensure-release-desktop` before capture.
- PNG outputs are 1280×800 window composites at the requested UI scale tokens.
- Doc-capture aborts when Command Manager contrast or shell-density guards fail.

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

### Multi-scale matrix (`high-contrast-{light,dark}-{100,125,150,200}-1280.png`)

At each scale token the text workspace keeps a single CM row, visible Find focus
ring, and non-overlapping status segments. Automated capture diagnostics report
at least six contrasting visible CM controls in every scene (nine at 150% on
the text fixture).

## Automated checks (this environment)

- `crates/unibrowser-desktop/tests/ui19Multiscale.test.mjs` guards the
  multi-scale capture script and optional PNG matrix presence.
- `scripts/capture-ui-19-multiscale.sh` after `npm run build`: four UI scales,
  both themes, doc-capture contrast guards on each run.
- Keyboard path: demo token `ui19-focus-visible` focuses Find via `data-control-id`.
- Not run here: Windows display scaling, Linux 2x HiDPI, axe-core, screen reader.

## Workshop HC parity (shell-density)

`scripts/capture-shell-density.sh` also writes `workshop-{light,dark}-hc-1280.png`
with `high-contrast` demo tokens. Those captures pass the same shell contrast and
mod-card overlap guards as the standard matrix.

## Grade

UI-19 grade: Drift.

Linux 1x automated multi-scale and high-contrast captures pass clipping and CM
contrast guards. Windows 100/125/200% display scaling, Linux 2x, axe, and a
manual screen-reader pass were not run in this environment. Those gaps prevent Pass.
