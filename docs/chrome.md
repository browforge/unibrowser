# Browser chrome (tabs, omnibox, status)

Shell behaviours that are not tied to a single file format. Command Manager pin semantics live in [ui-commands.md](ui-commands.md); status bar fields in [commands-undo-redo.md](commands-undo-redo.md).

## Tabs

| Action | Behaviour |
| --- | --- |
| Click tab | Activate |
| Shift+click tab | Set split pane to this tab ([split-view.md](split-view.md)) |
| Ctrl+click tab | Toggle **pin** (★ prefix; middle-click close blocked until unpinned) |
| Middle-click tab | Close tab (respects unsaved confirm; skipped when pinned) |
| × on tab | Close (same rules as middle-click) |
| Drag tab label | Reorder tabs in the strip (`reorder_tabs`) |
| Drop files on browse tab | Copy or move into that folder (Shift = move) |
| + | New home tab |

Pinned tabs are session-local today (not yet stored in session restore JSON).

## Omnibox

| Action | Behaviour |
| --- | --- |
| Type | Suggestions: pinned **Places**, **Recent** files, cwd listing, **URL** guesses (`https://…`) |
| ↑ / ↓ | Move selection in the list |
| Tab | Fill active suggestion without navigating |
| Enter | Apply active suggestion (if any), then **navigate** (`navigate_omnibox`) |
| Ctrl+Shift+V | Paste clipboard path/URL and navigate |

Accepted inputs: `file://`, absolute paths, `https://`, `unibrowser://` internal pages.

## Status bar

Path (click to copy), Command Manager context mode, unsaved label, encoding, editor cursor, font pt, zoom, encode progress. See shipped slice notes in [blitz-status.md](blitz-status.md).

## Command Manager layout (export / import)

Pinned tabs, command order, and floating toolbar positions are stored in `toolbars.json` under the config dir. To move a layout between machines or profiles:

| Control | Location | Behaviour |
| --- | --- | --- |
| **Export layout** | Command Manager header | Save JSON (`command-manager.json`) via system save dialog |
| **Import layout** | Command Manager header | Pick JSON; replaces in-memory layout and writes `toolbars.json` |

Import applies immediately and refreshes the Command Manager strip. Pin semantics and context pin are described in [ui-commands.md](ui-commands.md).

## Remote web loading (https tabs)

While the content webview loads, a light overlay spinner appears over `#content-host` until the page **finishes loading** or the document **title** updates. After **30 seconds** without either, the spinner is replaced by a short timeout message (connection hint). Overlay is per-window and only for the active https tab.

## Related

- [session-restore.md](session-restore.md) — tab URLs on launch
- [multi-window.md](multi-window.md) — per-window tab sets
- [security.md](security.md) — trust banners in document viewers
