# Multi-window behaviour

Unibrowser can open additional top-level windows that share one **profile** on disk but keep **separate tab strips**.

## Open another window

| Entry | Action |
| --- | --- |
| Settings | **Open another window** |
| Command palette | **New window** (`unibrowser.window.new`) |

Each new window gets a label `browser-2`, `browser-3`, … and loads the same shell (`index.html`) with its own content webview (`content-{label}`).

## Shared vs per-window

| Shared (one profile) | Per window |
| --- | --- |
| `preferences.json` (theme, downloads, session toggle, …) | Tab list and active tab |
| `keybindings.json`, `toolbars.json` | Split view, compare, undo stacks |
| Bundled mods and VSIX registry (single `AppState`) | Ephemeral chrome notices |
| Recent files, Places, workspace layout presets | `https://` content webview bounds |

Mods and settings load once; every window sees the same Command Manager catalog and preferences.

## Session restore

When **Restore tabs on launch** is enabled, quit captures **all** open windows via `session.json` format v2 (`windows[]` with labels, tab omnibox strings, active index, geometry). On launch, `main` restores first, then any `browser-N` entries.

See [session-restore.md](session-restore.md).

## OS file drop

Dragging files from the desktop onto **any** window opens them in **that** window's tabs: first file replaces the active tab, additional files open as new tabs. Invalid `.unibrow` projects show a friendly error instead of a blank tab.

## Related

- [browser-shell.md](browser-shell.md)
- [split-view.md](split-view.md) (second pane inside one window)
