# UI commands and Command Manager

## Mental model

1. **Command** — `unibrowser.font.size.increase`, `unibrowser.text.save`, … registered once with title, icon, keybinding.
2. **Handler** — mod provides enable + execute (`poll` / grey-out — see [ADR 0012](adr/0012-context-switching-ux-benchmarks.md)).
3. **UI command slot** — `{ "command_id": "…" }` on a Command Manager tab or floating toolbar.
4. **Surfaces** — manager tab, floating bar, palette, (future) context menu — **same command id**, no duplicate widgets.

Enablement is always computed by the registry (**grey out, never hide** core slots in the user’s layout).

## Benchmarks (why this shape)

| Product | What we take from it |
|---------|----------------------|
| [SOLIDWORKS Command Manager](https://help.solidworks.com/2024/english/solidworks/sldworks/c_commandmanager.htm) | **Primary model** — context tabs, grey-out-not-hide, user-customisable (better than Office ribbon) |
| [GIMP docks](https://docs.gimp.org/3.0/en/gimp-concepts-docks.html) | Dock, float, drag tool groups — we target **match or beat** with `floating_bars` |
| [Affinity](https://affinity.serif.com/en-us/whats-new/) | Stable studio chrome while tools change |
| [Blender workspaces](https://docs.blender.org/manual/en/latest/interface/window_system/workspaces.html) | Workspaces + operator search; tools stay findable |
| [DaVinci Resolve](https://documents.blackmagicdesign.com/UserManuals/DaVinci-Resolve-19-User-Manual.pdf) | Page/context switch with consistent shell (future timeline page) |

Full decision record: [ADR 0012](adr/0012-context-switching-ux-benchmarks.md).

## Stable chrome vs what swaps

| Stable | Swaps with context |
|--------|---------------------|
| Tab strip, omnibox, palette shortcut | Command Manager **selected tab** (or pinned tab) |
| Command Manager **frame** | Slots on active tab (resolved UI commands) |
| Status line | Central surface (Monaco, WebView, …) |

## Command Manager tabs (rollout)

Order: **Text → Office → Code → Image → Audio** (plus Browse/Web from shell).

| Tab | Phase | Notes |
|-----|-------|-------|
| Text | active | Save, font size, Monaco |
| Office | planned | DOCX/XLSX mods |
| Code | active layout | Shares text commands until LSP/format |
| Image | active | Font on text layers; raster tools later |
| Audio | planned | MIDI/audio native-until-encode ([ADR 0011](adr/0011-lazy-conversion-native-until-encode.md)) |

**Auto context:** engine sets `auto_context` from the active document (`infer_context`).

**Pin:** optional `context_pin` in `toolbars.json` — when set, auto context updates but the selected tab does not change until pin is cleared. Tauri: `pin_command_manager_context`, `clear_command_manager_context_pin` (UI toggle not shipped yet).

## Config file: `toolbars.json`

Path: same directory as [keybindings.md](keybindings.md).

```json
{
  "selected_tab": "text",
  "context_pin": null,
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

`anchor`: `docked` | `floating` (position updated on drag).

## Tauri API

| Command | Purpose |
|---------|---------|
| `get_command_manager` | Tabs, resolved commands, `context_pinned` |
| `select_command_manager_tab` | User tab click (does not set pin) |
| `save_command_manager_layout` | Persist reorder / float move |
| `pin_command_manager_context` | Pin current tab to `context_pin` |
| `clear_command_manager_context_pin` | Follow auto context again |

## Drag-anywhere toolbar roadmap

| Feature | Status |
|---------|--------|
| Reorder commands on active tab | Shipped |
| Move floating bar | Shipped |
| Drag command to another tab | Planned |
| Drag command out to **new** floating bar | Planned |
| Dock bars to edges / second window | Planned |

## Adding a command to the UI

1. Register `CommandMeta` + handler in a mod.
2. Add the id to factory default or user `toolbars.json`.
3. Optional keybinding in `KeybindingMap`.

Mods do not ship bespoke toolbar widgets — only command ids and enablement.

## Relation to mixed-format / conversion

Commands operate on the **native** backend selection. Preview/export pipelines ([ADR 0011](adr/0011-lazy-conversion-native-until-encode.md)) are separate from slot layout; timeline mods will add contexts and commands, not a second toolbar system.
