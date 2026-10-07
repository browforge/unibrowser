# UI-18 evidence

Build and capture:

- `scripts/capture-ui-18.sh`
- The script ran `npm run build` before capture.
- Both PNG files are 1280x800.
- The two PNG files have distinct SHA-256 hashes.
- `cargo test -p unibrowser-desktop workspace_layouts --lib` passed: 1 test,
  0 failed.

## Literal descriptions

### `layout-edit-light-1280.png`

The light-theme text editor remains visible behind an explicit layout-edit
overlay. A bounded toolbar near the top contains a layout name input, Save
layout, and Done. Three dashed blue targets mark the left, right, and bottom
dock regions. Each target names the panel it opens. The targets do not cover
the title bar, Command Manager, activity rail, or status bar.

### `layout-edit-dark-1280.png`

The dark-theme capture has the same geometry. The edit toolbar uses a dark
panel surface and the three dock targets use a recessed blue surface. Editor
text remains visible behind the overlay. No target, control, or label is
clipped.

## Grade

UI-18 grade: Drift.

The explicit edit mode, functional dock targets, named preset save, and
workspace-layout import/export are implemented. Command movement is disabled
outside edit mode. The required detached-toolbar and two-monitor restart proof
was not captured because this environment exposes one virtual display. That
missing evidence prevents Pass.
