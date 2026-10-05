# Keybindings

Unibrowser uses a **VS Code–style** keybinding map: defaults ship with the engine; you override them in JSON.

## Config file

| Platform | Path |
|----------|------|
| Linux | `$XDG_CONFIG_HOME/unibrowser/keybindings.json` (default `~/.config/unibrowser/keybindings.json`) |
| Windows | `%APPDATA%\unibrowser\keybindings.json` |
| macOS | `~/Library/Application Support/unibrowser/keybindings.json` (planned; desktop currently uses Linux-style paths on non-Windows builds) |

Create the `unibrowser` directory if it does not exist.

## Format

Object mapping **command id** to **chord string** (modifiers joined with `+`, key last):

```json
{
  "bindings": {
    "unibrowser.palette.toggle": "Ctrl+Shift+P",
    "unibrowser.font.size.increase": "Ctrl+=",
    "unibrowser.font.size.decrease": "Ctrl+-"
  }
}
```

Supported modifiers: `Ctrl`, `Alt`, `Shift`, `Cmd` / `Meta` / `Super`.

`unibrowser.palette.toggle` is the pseudo-command for the command palette (same role as VS Code “Show All Commands”).

## Defaults

| Command | Windows / Linux | macOS |
|---------|-----------------|-------|
| Command palette | `Ctrl+Shift+P` | `Cmd+Shift+P` |
| Increase font size | `Ctrl+=` | same |
| Decrease font size | `Ctrl+-` | same |

The Tauri shell resolves chords through `dispatch_key_chord`; the egui harness uses the same `KeybindingMap` (user file loading in the harness is optional until a shared config path is wired there).

## Future

- In-app keybinding editor and conflict warnings
- Chords with multiple steps (e.g. `Ctrl+K` then `Ctrl+S`)
- Per-context bindings (editor vs browser surface)
