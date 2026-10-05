# ADR 0012: Context switching UX — benchmarks and what we copy

- **Status:** Accepted
- **Date:** 2026-10-05

## Context

Unibrowser must **excel at context switching**: stable chrome, auto-relevant tools, grey-out-not-hide, user customisation, and eventually mixing formats (see [ADR 0011](0011-lazy-conversion-native-until-encode.md)). Bear’s benchmarks:

| Product | What to study | Primary link |
|---------|----------------|--------------|
| **SOLIDWORKS Command Manager** | Context tabs, grey-out-not-hide, more customisable than Office ribbon | [CommandManager overview](https://help.solidworks.com/2024/english/solidworks/sldworks/c_commandmanager.htm) |
| **GIMP** | Dockable, movable tool groups | [GIMP dockable dialogs](https://docs.gimp.org/3.0/en/gimp-concepts-docks.html) |
| **SOLIDWORKS + GIMP** | Drag-anywhere toolbars: dock, float, reorder, move between contexts | Unibrowser target: **match or beat** |
| **Affinity** (legacy personas + unified app) | Same studio feel while tools change per context | [Affinity unified app announcement](https://affinity.serif.com/en-us/whats-new/) |
| **Blender** | Workspaces, operators with `poll()`, tools stay discoverable | [Blender workspaces](https://docs.blender.org/manual/en/latest/interface/window_system/workspaces.html), [Operators poll](https://docs.blender.org/api/current/bpy.types.Operator.html#bpy.types.Operator.poll) |
| **DaVinci Resolve** | “Pages” (Media/Cut/Edit/Fusion/Color/Fairlight/Deliver) with shared shell | [Resolve workflows](https://documents.blackmagicdesign.com/UserManuals/DaVinci-Resolve-19-User-Manual.pdf) (Blackmagic user manual) |

## Decision — what we copy

### Stable chrome (Affinity, Resolve, SOLIDWORKS)

- **Fixed**: tab strip, omnibox, Command Manager **frame**, status line, palette entry point, keybinding layer.
- **Swaps**: Command Manager **tab selection**, slot contents, central **document surface** (Monaco, WebView, timeline, …).
- Avoid full-window mode swaps that hide navigation (contrast: some Resolve pages feel like separate apps — we keep one browser identity).

### Command Manager (SOLIDWORKS primary model)

- Context **tabs** (Text, Office, Code, Image, Audio, …) — [ADR 0010](0010-ui-commands-and-command-manager.md).
- Commands **grey out** when `Enablement::Disabled` — never remove core slots from the user’s layout.
- **Better than Office 365**: user-owned `toolbars.json`, reorder, floating bars — closer to SOLIDWORKS + GIMP than a fixed ribbon.

### Drag-anywhere toolbars (GIMP + SOLIDWORKS)

| Capability | Status |
|------------|--------|
| Reorder within active tab | **Shipped** (Tauri) |
| Docked “Quick” bar | **Shipped** |
| Float bar + position persist | **Shipped** (basic) |
| Drag between tabs | Roadmap |
| Drag out to **new** floating bar | Roadmap |
| Dock to edges / secondary monitors | Roadmap |

### Auto context + pin (Blender workspaces + SOLIDWORKS)

- **Auto (default)**: `CommandContext` from active document ([`infer_context`](../architecture.md)); Command Manager tab follows until pinned.
- **Pin tab (decided)**: separate **Pin tab** / **Follow context** control in the shell — not implicit on manual tab click. Pin sets `context_pin` in `toolbars.json` and freezes the selected tab; **Follow context** clears pin and resumes auto-follow.
- **Manual tab click**: temporary preview of another tab’s commands only; does **not** set pin or persist sticky behaviour.
- **Poll analogue**: handler `enable` predicates = Blender `poll()` — command visible, often greyed.

### Operator search (Blender, SOLIDWORKS search)

- Command palette = fuzzy search over registry; disabled commands listed dimmed ([ADR 0004](0004-s-key-command-palette.md)).

## What we avoid

- **Ribbon-only Office pattern** — fixed top-level groups, weak user reorder across contexts.
- **Hiding unavailable tools** — trains users to hunt when context changes.
- **Per-format separate executables** — breaks unified tab model ([ADR 0009](0009-browser-as-unified-shell.md)).
- **Import-time flatten** — see ADR 0011.

## Consequences

- UX reviews should score changes against **chrome stability** and **grey-out honesty**.
- New surfaces (timeline, audio mixer) add **contexts** and **slots**, not duplicate command systems.
- Benchmark docs live in [vision.md](../vision.md) and [ui-commands.md](../ui-commands.md); this ADR is the decision record.

## Open questions

- Resolve-style **page** switcher for future Video vs Browse vs Edit — one strip or nested?
- GIMP-style **single-column** tool palette vs horizontal Command Manager on small screens?
