# ADR 0003: Command, handler, and enablement

- **Status:** Accepted
- **Date:** 2026-10-05

## Context

One user-facing command must map to many backends (VS Code contributions, Eclipse commands/handlers, Blender `Operator.poll()`, Qt `QAction::setEnabled`, KParts action merge).

## Decision

- **Command** = stable string ID (`unibrowser.font.size.increase`), metadata (title, icon, default keybinding), registered by host or mod contributions.
- **Handler** = mod registers `(command_id, backend_id, execute_fn, enable_fn)`.
- **Dispatcher** chooses the handler for the **active document backend**; if none, command stays visible with enablement false.
- **Enablement** = predicate `fn(&HostContext) -> Enablement` where result is `Enabled | Disabled { reason }`. UI always shows the control; disabled = greyed + tooltip reason.

Context includes:

- Active mode and document type
- `SelectionSnapshot` (domains: text, path, bitmap, …)
- Capability flags from backend (`font_size`, …)

Expression-style keys (VS Code `when` clauses) may be added later; skeleton uses Rust predicates for clarity and testability.

## Consequences

- Tests can assert enablement without UI.
- Partial support (e.g. PDF with no editable text) is honest grey-out, not hidden menu items.
