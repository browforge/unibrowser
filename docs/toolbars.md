# Floating toolbars and `toolbars.json`

Command Manager layouts (tabs, floating bars, context pin) persist in **`toolbars.json`** next to `keybindings.json`. See [ADR 0013](adr/0013-dual-audience-foss-and-everyday-users.md) for product rules.

## Drag-anywhere floating bars

- Use the **⋮⋮** handle on a floating bar to drag it anywhere on screen, including across monitors.
- Positions are stored as **screen coordinates** (`coord_space: "screen"`) so a bar can stay on a second monitor when you move the main window.
- Coordinates use the same CSS pixel space as `PointerEvent.screenX` / `screenY`, so they track monitor layout across typical DPI scaling (per-display scale is handled by the OS + WebView).
- Legacy layouts without `coord_space` use viewport (client) pixels.
- Right-click a floating bar for **Reset this toolbar** (default position near the window). Settings **Reset floating toolbars** still clears all floating bars.

## Detached OS windows

- Right-click **Detach to own window** opens a small Tauri window (`toolbar-{bar_id}`) so commands can live on another monitor without the main chrome.
- Layout stores `presentation: "detached_window"` plus the same `x` / `y` screen position (window placement on restore).
- **Dock in main window** from the detached window header or the bar context menu; closing the toolbar window docks automatically.

## User taste wins

- No confirmation dialogs when you park toolbars off-centre or on another display.
- Mod Portal and install hints must **not cover** the document you were viewing ([ADR 0013](adr/0013-dual-audience-foss-and-everyday-users.md)).
- Export/import layouts via Command Manager for backup or sharing between machines.
