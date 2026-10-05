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

## Related

- [session-restore.md](session-restore.md) — tab URLs on launch
- [multi-window.md](multi-window.md) — per-window tab sets
- [security.md](security.md) — trust banners in document viewers
