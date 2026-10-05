# ADR 0004: Command palette and keybindings

- **Status:** Accepted (amended 2026-10-05)
- **Supersedes:** S-key as the default palette shortcut

## Context

SOLIDWORKS uses **S** as one example of command search; most desktop editors use **standard chords** (VS Code: Ctrl+Shift+P / Cmd+Shift+P). Users expect to **rebind** shortcuts like VS Code, SOLIDWORKS, Blender, and Affinity.

## Decision

1. **Command palette** — fuzzy search over all commands; disabled entries visible and dimmed; engine: `palette::search_commands`.
2. **Default palette chord** — **`Ctrl+Shift+P`** on Windows and Linux; **`Cmd+Shift+P`** when macOS is supported (same role as VS Code “Show All Commands”).
3. **No hard-coded S key** in the product shell. S may appear only as a user-defined binding.
4. **Keybinding model** (`unibrowser-engine::keybinding`):
   - Commands (and `unibrowser.palette.toggle`) map to a normalised **chord** (`Ctrl+Shift+P`, `Ctrl+=`, …).
   - **Defaults** ship with the engine; **user overrides** load from config (e.g. `keybindings.json` under the app config dir), same merge idea as VS Code `keybindings.json`.
   - Mods contribute command metadata; they do not own global shortcut registration without going through the host map.
5. Shells (Tauri, egui harness) listen for chords resolved from `KeybindingMap`, not ad hoc keys.

## Consequences

- Palette logic stays in the engine; UI only renders and sends chord events.
- Future: keybinding editor UI, conflict detection, and export/import of binding sets.
