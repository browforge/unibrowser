# UI commands and Command Manager

## Mental model

1. **Command** — `unibrowser.font.size.increase`, `unibrowser.text.save`, … registered once with title, icon, keybinding.
2. **Handler** — mod provides enable + execute for a backend.
3. **UI command slot** — `{ "command_id": "…" }` placed on a Command Manager tab or floating toolbar.
4. **Surfaces** — the same slot can appear on the manager, a floating bar, the palette, or (future) context menus.

Enablement is always computed by the registry (grey out, never hide core commands).

## Command Manager tabs (rollout)

Default tabs (in order): **Text**, **Office**, **Code**, **Image**, **Audio**.

- **Text** — active: save, font size.
- **Code** — shares text commands until code-specific commands land.
- **Image** — font commands for text layers in PNG/SVG.
- **Office** / **Audio** — `phase: planned` until format mods ship.

The engine sets `auto_context` from the active tab’s document; the shell switches the selected tab unless the user picks another.

## Config file: `toolbars.json`

Path: same directory as [keybindings.md](keybindings.md) (`unibrowser` under user config).

```json
{
  "selected_tab": "text",
  "tabs": [
    {
      "id": "text",
      "label": "Text",
      "phase": "active",
      "commands": [
        { "command_id": "unibrowser.text.save" },
        { "command_id": "unibrowser.font.size.increase" }
      ]
    }
  ],
  "floating_bars": [
    {
      "id": "quick-text",
      "label": "Quick",
      "anchor": "docked",
      "x": 0,
      "y": 0,
      "commands": [{ "command_id": "unibrowser.text.save" }]
    }
  ]
}
```

`anchor`: `docked` (in chrome) or `floating` (draggable overlay; `x`/`y` updated on drag).

## Tauri API

| Command | Purpose |
|---------|---------|
| `get_command_manager` | Tabs, resolved UI commands, floating bars |
| `select_command_manager_tab` | User tab click |
| `save_command_manager_layout` | Persist after drag-reorder or float move |

## Adding a command to the UI

1. Register `CommandMeta` + handler in a mod (or engine builtins).
2. Add the id to the appropriate tab in `CommandManagerFile::factory_default` or user `toolbars.json`.
3. Optional: default keybinding in `KeybindingMap`.

No separate “toolbar button” type — only command ids.
