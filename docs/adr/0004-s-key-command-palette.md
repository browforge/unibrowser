# ADR 0004: S-key command palette

- **Status:** Accepted
- **Date:** 2026-10-05

## Context

SOLIDWORKS users expect **S** to open command search; VS Code uses Ctrl+Shift+P. Unibrowser standardises on **S** when focus is not in a text entry (configurable later).

## Decision

- Global shortcut **S** toggles palette (skeleton); future: also Ctrl+Shift+P alias on Windows/Linux.
- **Fuzzy match** on title, command id, and aliases (case insensitive, subsequence scoring).
- List **all** commands; disabled entries shown dimmed with reason (not filtered out).
- Sort: match score, then recent/frequent (recency stubbed in skeleton).
- Executing a disabled command is a no-op with status message.

Implementation: `unibrowser-engine::palette::search_commands`.

## Consequences

- Palette logic is engine-side; Tauri/egui only renders results.
