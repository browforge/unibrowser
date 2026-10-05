# Floating toolbars and `toolbars.json`

Command Manager layouts (tabs, floating bars, context pin) persist in **`toolbars.json`** next to `keybindings.json`. See [ADR 0013](adr/0013-dual-audience-foss-and-everyday-users.md) for product rules.

## Drag-anywhere floating bars

- Use the **⋮⋮** handle on a floating bar to drag it anywhere on screen, including across monitors.
- Positions are stored as **screen coordinates** (`coord_space: "screen"`) so a bar can stay on a second monitor when you move the main window.
- Legacy layouts without `coord_space` use viewport (client) pixels.

## User taste wins

- No confirmation dialogs when you park toolbars off-centre or on another display.
- Mod Portal and install hints must **not cover** the document you were viewing ([ADR 0013](adr/0013-dual-audience-foss-and-everyday-users.md)).
- Export/import layouts via Command Manager for backup or sharing between machines.
