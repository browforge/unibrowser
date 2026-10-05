# Key chord sequences (sketch)

VS Code-style **sequences** (e.g. `Ctrl+K` then `Ctrl+S`) are not yet user-configurable. V1 behaviour:

## Implemented sketch

- `KeySequenceMatcher` in `unibrowser-engine` holds at most one pending step.
- **`Ctrl+K` → `Ctrl+S`** within the same session dispatches `unibrowser.text.save` (same as Save command).
- Single chords still win via `KeybindingMap::command_for_chord` (palette, font size, etc.).
- If the second key does not complete a known sequence, the pending step is cleared.

## Deferred

- Timeout (2.5s) between steps
- `keybindings.json` sequence syntax (`"when": "sequence"`)
- Display in palette (“chord of chords”)
- macOS `Cmd` variants for sequences

## Rationale

Sequences need clear UX (status hint “Waiting for second key…”) and must not fight the command palette’s `Ctrl+K` usage. The sketch proves dispatch plumbing before committing to full VS Code parity.
