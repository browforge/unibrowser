# Split view (experimental)

Side-by-side **two tabs in one window** with a vertical divider between panes.

## How to use

1. Click **Split** in the toolbar, or Shift+click a tab to pin it as the secondary pane.
2. Primary pane: active tab. Secondary pane: the split tab (highlighted on the tab strip).
3. Click **Split** again to turn split off.

## Behaviour

- Secondary pane shows the same viewers as the primary tab (document viewer, read-only text preview, or a status line when preview is unavailable).
- Layout: horizontal flex row (`#split-primary` | `#split-secondary`). The divider is the secondary pane’s left border.
- State is per-window (`SplitViewState`) and can be saved in workspace layout presets.

## Limits (experimental)

- No draggable split ratio yet (50/50 flex).
- Secondary text tabs are read-only preview, not a second Monaco instance.
- HTTP tabs use one shared content webview (primary only); split does not duplicate the webview.

## Related

- [multi-window.md](multi-window.md) for a second window instead of split.
