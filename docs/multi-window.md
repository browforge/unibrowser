# Multi-window behaviour

Unibrowser can open additional windows from **Settings → Open another window**.

## Per-window tab state

Each window label (`main`, `browser-2`, …) owns its own:

- Tab list and active tab
- Split view configuration
- Per-document undo stacks
- File compare state
- Ephemeral chrome notices

Preferences on disk (theme, downloads, bookmarks, workspace layouts, toolbars) remain **shared** across windows.

## Web content overlay

Each window has its own embedded `content-{label}` webview for `https://` tabs. Bounds are synced from that window's `#content-host` region.

## Session restore

Quit-time session snapshots apply to the **main** window only.
