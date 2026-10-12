# History panel UX

Specification for browsing and rolling back edit history when [ADR 0031](../adr/0031-edit-history-as-file-format.md) is implemented. The panel is **core chrome** in the bottom dock; labels and previews come from **plugins**.

## Placement

- **Region:** bottom dock, tab **History** (alongside Problems, Output where present).
- **Toggle:** command `unibrowser.panel.history.toggle`; default chord none (palette + View menu).
- **Width:** full dock width; min height 120 px; resizable with other bottom panels.
- **Read-only session:** panel stays available; rollback and redo disabled with reason “Install base-text 1.4.2 to edit history” (example).

## List content

Each row = one committed op (`seq` order, newest at top or bottom — match Problems panel: **newest at top**).

| Column | Source |
| --- | --- |
| Time | `timestamp` (local display) |
| Source | `plugin_id` short name |
| Summary | Plugin `summary` string or `op_type` fallback |
| Seq | `seq` for support copy |

Multi-select: not in v1. Single click selects; double-click or **Rollback to here** applies.

## Actions

| Action | Behaviour |
| --- | --- |
| **Rollback to here** | Set head to selected `seq`; document view updates; dirty if ahead ops discarded from head (redo stack) |
| **Redo** | Same as global redo |
| **Squash history…** | Opens inline confirm ([ADR 0031](../adr/0031-edit-history-as-file-format.md) §6); not a modal over content |
| **Clean before sharing…** | Wizard in panel or palette: squash + strip log; link to privacy note |

Keyboard: arrows move selection; Enter = rollback; Esc returns focus to document.

## Preview (optional v1.1)

Plugin hook `history.preview_op(seq)` may highlight range in viewer. If absent, list-only is enough for v1.

## Empty and error states

| State | Copy |
| --- | --- |
| No history file | “Edits in this session are not recorded to disk yet. Save to a Unibrowser document or enable a `.unibrow-history` sidecar.” |
| Replay stopped | Banner above list: last good `seq`, required plugin version, **Install** / **Read-only** |
| Foreign file, no sidecar | “Undo is for this session only. Save as .unibrow or enable history sidecar to keep edits.” |

## Visual language

- Reuse Problems panel table density and high-contrast tokens.
- Rollback is destructive to redo branch — confirm uses **inline** strip in panel, not `window.confirm`.
- Never cover Save or autosave status.

## Related commands

Existing ids: `unibrowser.document.undo`, `unibrowser.document.redo` ([commands-undo-redo.md](../commands-undo-redo.md)). History panel does not register duplicate undo buttons in the title bar (per [ui-implementation-plan.md](ui-implementation-plan.md) UI-04).
