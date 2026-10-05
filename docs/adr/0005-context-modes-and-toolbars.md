# ADR 0005: Context modes and toolbars

- **Status:** Accepted
- **Date:** 2026-10-05

## Context

SOLIDWORKS CommandManager: modes switch tabs; tools grey out instead of disappearing. Affinity studios inspired user-custom layouts.

## Decision

- **Modes** are enumerated (`Browse`, `Text`, `Image`, `Vector`, `Document`, `Web`, …). Active mode derived from backend + selection; user may override (pin) later.
- **Toolbar tabs** map 1:1 to modes for defaults. On mode change, **selected tab** updates automatically; command buttons on the tab keep **fixed positions**.
- **Enablement** refreshes on selection/document change; inapplicable commands `enabled = false` (grey), never `visible = false` for core commands.
- **User layouts** stored as JSON:

```json
{
  "toolbar_tabs": [
    { "id": "text", "label": "Text", "commands": ["unibrowser.font.size.increase", "..."] }
  ],
  "selected_tab": "text"
}
```

Mods contribute **suggested** command groups via manifest; users can reorder (Phase 1 persistence).

## Consequences

- UI code must distinguish disabled styling from hidden widgets.
- Auto tab switch must not fight user-pinned mode (future flag).
