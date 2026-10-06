# ADR 0027: Command system capabilities and extensibility

- **Status:** Accepted
- **Date:** 2026-10-06
- **Supersedes in part:** ADR 0003, ADR 0005, ADR 0010

## Context

The current command system has stable command IDs, backend handlers, Enabled or Disabled enablement, command slots, Command Manager tabs, and floating bars. It does not yet model composite controls, reusable families, emulation, mixed selection state, deterministic contribution merge, or capability providers from mods.

The product needs a Command Manager that is infinitely extensible without becoming unstable. The same text family should recur across plain text, Markdown, rich documents, and code. A control may be directly supported, approximated by a mod, or unavailable, but its position must remain predictable.

The command system must follow [principles.md](../principles.md): trust the user, never block saving, remain local-first, behave consistently across formats, prefer durable mechanisms, and put format-specific behaviour in mods rather than core.

## Decision

### 1. Adopt a layered model

The model is:

```text
Command -> Control -> Group -> Family -> Command Manager tab -> Context
```

- A Command is a stable semantic action and dispatch ID.
- A Control is a host-rendered component bound to one or more commands and values.
- A Group is an indivisible named layout cluster.
- A Family is a reusable ordered semantic set with defaults and extension points.
- A Command Manager tab places families and groups.
- A Context is the active document, backend, selection, tool, permissions, and providers.

Control types include button, toggle, split button, dropdown, value stepper, colour, gallery, segmented, menu button, and indicator action. Controls have compact/large, icon/label, value, mixed, and capability variants.

### 2. Replace binary enablement with capability resolution

Each control resolves to:

- `supported`
- `emulated`
- `unsupported`
- `hidden_by_user`
- `locked` (licence or permission policy only; neutral wording; no upsell UI)

Mixed selection is orthogonal and may combine with supported or emulated capability. `locked` is distinct from `unsupported` and is never returned by format backends or mods.

Resolution starts from the family default, then applies context family overrides, winning per-control provider overrides, selection state, user visibility, and permission policy.

Unsupported controls remain in place and explain why. Mods and backends cannot return hidden for core controls. Only an explicit user customisation produces `hidden_by_user`.

### 3. Preserve family placement

Families own canonical control order and group boundaries. Every context that enables a family uses the same relative order. Unsupported controls retain geometry.

At constrained widths:

1. Controls use compact variants.
2. Complete low-priority groups move to overflow.
3. Complete low-priority families move to overflow.
4. Context and Following/Pin remain visible.

Groups never wrap, split, or create another row.

### 4. Expose declarative mod contributions plus runtime providers

`ModManifest` gains a versioned optional `contributes` object for command metadata, controls, groups, families, tabs, toolbars, and capability provider declarations.

The manifest remains metadata only. Native or WASM payloads register versioned runtime callbacks for selection-dependent capability, state, value, emulation, and execution.

Mods may extend declared core family extension points, add their own families/groups/toolbars, and suggest floatable or detachable toolbars. The host renders all controls and owns all placement.

### 5. Resolve conflicts deterministically

Provider precedence is:

1. User-selected provider.
2. Active backend.
3. Explicit mod provider by dependency order and priority.
4. Family emulation provider.
5. Unsupported fallback.

Ties use dependency order, numeric priority, then lexical mod ID. Duplicate incompatible IDs are rejected with resolver diagnostics. Load order alone never decides.

Core reserves `unibrowser.*`. Mods use their manifest name as namespace.

### 6. Keep Save outside capability negotiation

No mod, capability provider, family, control, diagnostic, selection state, licence check, or failed callback may disable, replace, cover, or gate Save, Save As, autosave, or recovery.

Host layout reserves content and Save geometry. In-window toolbars are clamped. Detached toolbars are host-owned platform windows and recover when monitors disappear.

### 7. Use one resolved control model on every surface

The engine returns `ResolvedControl` with capability, selection state, values, reasons, emulation note, labels, shortcuts, variants, provider, and context revision.

Command Manager, floating toolbar, palette, keyboard help, and context menus consume the same model. They do not each recompute enablement.

### 8. Persist by stable ID

Layout persistence stores stable context, family, group, control, toolbar, monitor, and provider IDs. It records user-hidden state separately from provider capability. Removed mod contributions remain orphaned customisation entries so they can be restored if the mod returns.

## Keyboard and accessibility

Every control is reachable through group roving focus, the command palette, and the host keybinding map. Disabled controls remain focusable for their explanation. Emulated and mixed states are exposed to assistive technology.

The standard command palette shortcut remains Ctrl+Shift+P. S is not reserved, but users may bind it.

## Trust and permissions

Host permissions separate command execution, capability provision, selection read, document edit, Command Manager contribution, toolbar contribution, and toolbar detachment.

Capability callbacks are read-only. Edits use host transactions and undo. T1 native mods require trust; T2 WASM receives only granted imports. A mod receives no Command Manager DOM access.

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes. Customisation, provider choice, hiding, and detachment remain user-owned |
| Never blocks save or work | Yes. Saving is explicitly outside capability negotiation |
| Local-first | Yes. Resolution and persistence require no account or cloud |
| Predictable across formats | Yes. Families and unsupported slots retain stable placement |
| Simple durable mechanism | Yes. Versioned data, deterministic merge, stable IDs |
| Mod not core | Yes. Core mediates; providers and emulations live in backends/mods |

## Consequences

Positive:

- Cross-format controls become predictable without pretending every format supports every operation.
- Mods can extend specialist workflows without arbitrary UI injection.
- Composite controls such as `14 pt` no longer need ad hoc frontend logic.
- Capability and accessibility behaviour can be tested without screenshots.
- Removed mods do not destroy user customisation.

Costs:

- Registry and DTOs become richer.
- Capability callbacks need revisioning, cancellation, timeouts, and diagnostics.
- Persistence needs schema migration.
- Existing command slots and enablement APIs need a compatibility adapter.
- Contribution conflict UX and provider selection require settings surfaces.

## Compatibility and migration

1. Existing commands become button controls by default.
2. Existing Enabled maps to supported; Disabled maps to unsupported with reason.
3. Existing `UiCommandSlot` layouts migrate into generated groups/families.
4. Existing floating bars become toolbars containing generated controls.
5. Existing `toolbars.json` remains readable and migrates once to a versioned layout file.
6. API compatibility follows ADR 0006 and ADR 0025; contribution schema versions separately.

## Resolved defaults (2026-10-06)

1. Markdown inline-HTML emulation on by default with visible emulation marker.
2. Emulation marker always visible, subtle, never colour alone; tooltip explains.
3. One Text Formatting family; groups (Font/Type, Emphasis, Baseline/Colour) movable separately; identical default geometry across contexts.
4. Hide applies per context by default; optional hide in all contexts.
5. User-picked provider outranks backend; reset to default provider exposed.
6. Detached toolbars: monitor identity (name/EDID) + geometry; nearest visible monitor fallback.
7. Code editor font/size in Editor Appearance at Text Formatting Type geometry.
8. Licensed controls use `locked`, not `unsupported`.

## Detailed design

See [Command System and Command Manager](../design/command-system.md).
