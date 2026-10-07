# Command System and Command Manager

Status: design proposal

This document defines the component model, capability negotiation, stable placement, mod contribution model, keyboard access, and accessibility contract for commands in Unibrowser.

The governing product idea is trustworthy repetition: the same job has the same command ID, the same family, and the same place wherever it appears. A context may support a control directly, approximate it, or explain why it cannot act. It does not silently rearrange the interface.

## Goals

1. Keep command meaning independent from its visual control.
2. Let one family recur predictably across file types and selections.
3. Let engines and mods declare honest support at family and control level.
4. Make extension broad enough for specialist tools without letting a mod cover work or interfere with saving.
5. Expose the same commands through the Command Manager, floating toolbars, context menus, keyboard bindings, and the command palette.
6. Keep the engine small. Core owns contracts, resolution, placement, persistence, and safety. Format behaviour belongs in backends and mods.

## Non-goals

- Letting mods inject arbitrary HTML, CSS, or WebView overlays into the Command Manager.
- Allowing a toolbar to claim document space without the host layout manager.
- Treating a JSON manifest as executable behaviour.
- Hiding unsupported controls to make a context look cleaner.
- Allowing validation, diagnostics, or command providers to gate Save, Save As, or autosave.

## Existing foundation

The current implementation already has useful seams:

- `CommandRegistry` owns stable IDs, metadata, handlers, and enablement.
- `UiCommandSlot` places a command ID on a surface.
- `CommandManagerState` owns tabs, auto context, pinning, and floating bars.
- `CommandSurface` identifies Command Manager, floating toolbar, palette, and context menu.
- `ModManifest` carries metadata and points to a compiled native or WASM payload.
- `HostContext` carries mode, document type, backend, selection, and a small capability set.

The main gap is that a slot currently means one button and enablement has only Enabled or Disabled. The target model separates command behaviour from control composition and gives capability state enough fidelity for direct support, emulation, mixed selection, and user hiding.

## Layered model

The layers are ordered from behaviour to placement.

### 1. Command

A command is a stable semantic action.

```text
Command
  id
  label
  short_label
  description
  icon
  aliases
  default_keybinding
  permissions
  execution_contract
```

Examples:

- `unibrowser.text.bold`
- `unibrowser.text.strikethrough`
- `unibrowser.text.baseline.subscript`
- `unibrowser.pdf.annotate.highlight`
- `acme.cad.section.flip`

Commands are not buttons. One command may appear in a button, split button menu, gallery item, palette row, keyboard binding, or context menu.

Command execution remains host-dispatched to the active provider. Execution must recheck capability state immediately before acting because selection state may have changed after rendering.

### 2. Control

A control is a host-rendered interaction bound to one or more commands and optional values.

| Type | Purpose | Example |
| --- | --- | --- |
| `button` | One immediate action | Undo |
| `toggle` | Boolean state | Bold |
| `split_button` | Default action plus variants | Export plus format menu |
| `dropdown` | Choose one named value | Font family |
| `value_stepper` | Current value plus step up/down | 14 pt |
| `colour` | Current colour plus palette | Text colour |
| `gallery` | Visual choice set | Styles, stamps |
| `segmented` | Small mutually exclusive set | Left, centre, right |
| `menu_button` | Opens commands without a default action | More annotation tools |
| `indicator_action` | State plus action | Page 3 of 12 |

Every control declares:

```text
Control
  id
  type
  command_ids[]
  value_source?
  state_source?
  variants
  labels
  icon
  overflow_behaviour
  accessibility
```

Control IDs are stable placement identities. Command IDs remain stable behaviour identities. This distinction allows `text.size` to remain one value stepper while internally invoking increase, decrease, set, and reset commands.

#### Visual variants

- Density: `compact`, `large`.
- Content: `icon_label`, `icon_only`, `label_only`, `value`.
- Width: `fixed`, `content`, `wide`.
- State: `supported`, `emulated`, `unsupported`, `hidden_by_user`, `locked`.
- Selection: `off`, `on`, `mixed`, `indeterminate`, `value`, `mixed_value`.
- Attention: `normal`, `changed`, `warning`. Warning is reserved for actual consequence, not unsupported capability.

Compact controls use a 16 px icon and short label in a 30 px target. Large controls use a 20 px icon and label in the 64 px Command Manager row. Icon-only is allowed only for familiar actions or user customisation, and always retains an accessible name and tooltip.

#### Mixed state

Mixed is selection state, not capability state. A Bold toggle may be `supported + mixed`; a font size stepper may be `supported + mixed_value`. The control remains actionable. Activating a mixed toggle applies one deterministic result to the entire selection, normally On. A value control displays `Mixed` and applies the chosen value to all selected objects.

### 3. Group

A group is a named, indivisible layout unit containing controls.

```text
Group
  id
  label
  controls[]
  order
  overflow_priority
  min_variant
```

Groups never wrap and never split across the main row and overflow. A divider belongs between groups, not between every control.

Examples:

- History: Undo, Redo
- Type: Font, Size, Format
- Emphasis: Bold, Italic, Underline, Strikethrough
- Baseline: Subscript, Superscript
- Annotation: Highlight, Note, Area, Ink

### 4. Family

A family is a reusable semantic set that keeps its internal order across contexts.

The text-formatting family is:

1. Font
2. Size
3. Bold
4. Italic
5. Underline
6. Strikethrough
7. Subscript
8. Superscript
9. Text colour

The family supplies:

```text
Family
  id
  label
  default_capability
  control_order[]
  default_groups[]
  context_defaults{}
  extension_points[]
  overflow_priority
```

Family identity is the unit of trustworthy repetition. If Text Formatting is enabled in plain text, Markdown, rich documents, and code, the same controls occupy the same relative positions. Unsupported controls stay in place.

A mod may extend a core family only at declared extension points, such as `after:underline` or `end`. It cannot reorder core controls unless the user explicitly customises that family.

### 5. Command Manager tab

A tab is an ordered collection of family placements and context-specific groups.

```text
CommandManagerTab
  id
  label
  family_placements[]
  standalone_groups[]
  follows_contexts[]
```

Text, PDF, Image, CAD/3D, and other tabs are views of families, not parallel registries. The active context chooses the default tab. Pinning changes which tab is shown, not the active document provider.

### 6. Context

A context is the host snapshot used to resolve capability and state:

```text
Context
  document_type
  backend_id
  mode
  selection_domains[]
  selection_count
  editable
  active_tool
  provider_ids[]
  permission_grants[]
```

Context changes include active tab, active backend, selection, editability, active tool, and mod availability. Capability resolution is pure for a given snapshot and can be tested without UI.

## Capability negotiation

### Capability states

Each control resolves to exactly one availability state:

| State | Meaning | Presentation |
| --- | --- | --- |
| `supported` | Active provider performs the declared semantics directly | Normal control |
| `emulated` | A provider supplies a documented approximation | Active control with optional subtle approximation marker |
| `unsupported` | No valid provider can perform or approximate it | Greyed in place with reason |
| `hidden_by_user` | User explicitly removed it from this placement | Absent; reserved placement persists in customisation data |
| `locked` | Licensed or permission-gated; not a format limitation | Greyed in place with neutral wording; no upsell popups |

`hidden_by_user` is not a provider result. Providers may never hide a core control. `locked` is host policy only; mods and backends cannot return it.

An emulated result includes:

```json
{
  "state": "emulated",
  "provider": "markdown-baseline",
  "note": "Approximated with font scaling",
  "fidelity": "visual",
  "round_trip": "best_effort"
}
```

The optional marker is a small dotted corner or approximation glyph. It must not compete with warnings. Tooltip and screen-reader description always disclose the emulation.

### Resolution order

For every family placement:

1. Start with the family default capability for the active context.
2. Apply the context's family override.
3. Apply the winning per-control provider override.
4. Resolve selection state and value.
5. Apply the user's visibility choice.
6. Recheck permission and trust policy.
7. Return one resolved control model to every surface.

The result is never inferred from whether a command handler happened to register. Missing providers resolve to unsupported with a reason.

### Provider precedence

Providers are considered in this order:

1. User-selected provider for this family or control.
2. Active document backend provider.
3. Explicit mod provider ordered by declared priority.
4. Family emulation provider.
5. Unsupported fallback.

Ties use dependency order, then numeric priority, then lexical mod ID. The resolver records all candidates and the reason the winner was selected. Silent last-loaded-wins behaviour is forbidden.

Core does not automatically outrank a mod. The active backend has first claim because it owns the document representation. A user may select another compatible provider.

### Family default and control override example

Markdown declares the Text Formatting family as supported. It overrides:

- Bold: supported.
- Italic: supported.
- Underline: emulated through inline HTML.
- Strikethrough: supported.
- Subscript: emulated through `<sub>` or a font-scaling fallback, depending on Markdown policy.
- Superscript: emulated through `<sup>` or a font-scaling fallback.
- Font: unsupported because portable Markdown has no font family semantic.
- Text colour: emulated through inline HTML when allowed.

The family remains complete and stable. The context changes state, not geometry.

### Unsupported reasons

Reasons describe the active situation and a remedy where one exists:

> Strikethrough  
> Plain text has no strikethrough formatting. Save as Markdown or use a rich document to preserve it.

Reasons must not blame the user. Unsupported controls remain keyboard focusable for explanation but do not execute.

### Runtime updates

Static manifest flags provide the initial capability set. Runtime callbacks update selection-dependent state, values, and reasons. The host:

- coalesces updates per animation frame;
- cancels stale responses by context revision;
- imposes a short timeout and retains the last safe state;
- treats provider failure as unsupported with a provider failure reason;
- never lets capability evaluation block Save or document input.

## Stable placement: trustworthy repetition

### Canonical order

Each family owns canonical control order and default group boundaries. A context may enable a subset of families but cannot reorder a family's controls. Unsupported controls occupy their canonical slots.

The default Text tab order is:

1. History
2. Text Formatting
3. Find
4. Proofing
5. Export

Within Text Formatting, Font and Size precede emphasis and baseline controls in every context.

### Collapse and overflow

1. Controls select compact variants before any group moves to overflow.
2. A complete low-priority group moves to overflow.
3. A complete low-priority family moves to overflow.
4. The context control and Following/Pin remain visible.
5. No control wraps and no third row is created.

Priority is deterministic:

- `critical`: Save-adjacent contextual action, selection/tool exit.
- `primary`: frequent family controls.
- `normal`: ordinary groups.
- `low`: proofing, export, specialist galleries.
- `custom`: user or mod groups unless promoted by the user.

Within one priority, later groups overflow first. The overflow menu preserves the same family and group order.

### Persistence

Persistence uses stable IDs, not array indices:

```text
layout_version
contexts{}
families{}
toolbars{}
monitor_anchors{}
orphaned_contributions{}
```

- Per-context settings choose family order, group visibility, density, and pinned tab.
- Per-family settings choose control order at allowed extension points, control variant, and user-hidden controls.
- Floating toolbar geometry uses physical screen coordinates plus monitor identity and scale.
- If a monitor disappears, the host restores the toolbar inside the nearest visible work area.
- If a mod is removed, its contribution becomes an orphaned placeholder in edit mode and does not collapse neighbouring core placement.
- If the mod returns with compatible IDs, customisation is restored.

Reset is available per control, family, context, toolbar, and entire Command Manager.

## Mod API

### Manifest is metadata

The JSON manifest declares contribution metadata. Executable capability checks and command handlers live in the compiled native or WASM payload, consistent with ADR 0002 and ADR 0006.

Illustrative `info.json` contribution:

```json
{
  "name": "markdown-baseline",
  "version": "1.2.0",
  "unibrowser_api": "^0.2",
  "capabilities": [
    "commands.execute",
    "commands.capability.provide"
  ],
  "contributes": {
    "commands": [
      {
        "id": "markdown-baseline.subscript",
        "title": "Subscript",
        "icon": "subscript"
      }
    ],
    "command_families": [
      {
        "id": "unibrowser.family.text-formatting",
        "extend": true,
        "controls": [
          {
            "id": "markdown-baseline.subscript-control",
            "type": "toggle",
            "command": "markdown-baseline.subscript",
            "insert": {
              "after": "unibrowser.control.text.strikethrough"
            }
          }
        ]
      }
    ],
    "toolbars": [
      {
        "id": "markdown-baseline.quick",
        "label": "Markdown baseline",
        "groups": ["markdown-baseline.baseline"],
        "floatable": true,
        "detachable": true
      }
    ],
    "capability_providers": [
      {
        "id": "markdown-baseline.provider",
        "families": ["unibrowser.family.text-formatting"],
        "contexts": ["text/markdown"],
        "runtime": "selection"
      }
    ]
  }
}
```

The long-term manifest schema adds an optional `contributes` object to `ModManifest`. The host validates it before loading payload callbacks.

### Runtime callback contract

The payload registers versioned callbacks:

```text
resolve_family(context_revision, family_id, context) -> FamilyCapability
resolve_controls(context_revision, control_ids, context) -> ControlCapability[]
read_control_state(context_revision, control_ids, selection) -> ControlState[]
execute(command_id, arguments, context_revision) -> CommandResult
```

Callbacks return data only. The host renders controls. A mod does not receive DOM access.

### Runtime API (v1)

Host constant: `MOD_RUNTIME_API_V1` (`1`). Native mods register a `ModRuntimeRegistry` entry with:

- `ModRuntimeDescriptor` (`mod_name`, `provider_id`, `api_version`, `ModPermissions` bitfield)
- `snapshot(control_id) -> ControlRuntimeSnapshot` (read-only capability, selection, value, mixed, emulation note)
- optional `execute(command_id) -> host action token` when `EDIT_DOCUMENT` is granted

Permissions align with the table above: `PROVIDE_CAPABILITY`, `READ_SELECTION`, `EDIT_DOCUMENT`, `COMMAND_MANAGER`, `TOOLBAR`, `DETACH_TOOLBAR`. Capability snapshots cannot block Save or cover content; document edits must return host action tokens (for example `markdown-subscript-emulate`) that the shell applies through `sync_text_document` and the per-path undo stack.

The proving mod `cmd-proving` registers `cmd-proving::native`, contributes `cmd-proving.control.chip` after core Subscript, and ships detached toolbar `cmd-proving.detached-markdown`.

### Review 2026-10-07

Review target: `6c49cf9`.

- **Read-only capability path:** the v1 callback receives only a control ID and returns data. It has no document, DOM, window, or Save handle. Native T1 code is trusted process code, so Rust types cannot prevent unrelated global side effects; the host now bounds its response to 50 ms, validates API version and control identity, ignores selection/value without `READ_SELECTION`, and degrades callback failure to Unsupported. Save controls bypass runtime overlays.
- **Execution permission and edits:** `EDIT_DOCUMENT` is checked before callback execution. The callback returns a typed `HostEditRequest`, not executable UI code or an arbitrary action string. Markdown Subscript is applied to the host text backend through one `UndoStacks` transaction, then the frontend reloads the document. The direct Monaco mutation from `6c49cf9` was removed.
- **Content and Save safety:** capability callbacks cannot request surfaces. Command Manager and toolbar contributions are filtered by manifest grants. Detached windows are host-created and screen-clamped. Capability or contribution diagnostics are advisory and never gate Save.
- **Timeout and error model:** v1 has typed `RuntimeCallbackError`, a 50 ms capability deadline, a 250 ms execution deadline, and WASM fuel metering. A timed-out native T1 thread cannot be safely killed in-process; it is abandoned and the provider degrades. A future out-of-process native host can enforce cancellation.
- **Version shape:** callback snapshots and edit requests carry stable, serialisable names under `MOD_RUNTIME_API_V1`. Unknown API versions and mismatched control IDs are rejected. V2 should replace the permission bitfield with a forward-compatible named set and add cancellation/context revision to execution requests.
- **Native/WASM parity:** the former “mirror” test was hollow because it copied a native snapshot through a helper. It was removed. `real_wasm_snapshot_matches_native_proving_runtime` now builds a real `wasm32-unknown-unknown` guest, loads it through wasmi host imports, captures the guest-produced JSON snapshot, and compares it with the native proving provider.
- **Keyboard and ARIA:** `6c49cf9` checked ordinary ArrowDown before Alt+Down, making Alt+Down unreachable. Unsupported native-disabled buttons were also absent from roving focus and could not announce reasons. Unsupported controls now use `aria-disabled` while remaining focusable; activation announces the reason without invoking the command. Emulated controls retain the subtle `.cm-emulation-mark` and described approximation. Automated checks cover emulated, unsupported, and mixed states.

Review evidence:
[`cmd-proving-detached-light.png`](../screenshots/cmd/cmd-proving-detached-light.png),
[`cmd-proving-detached-dark.png`](../screenshots/cmd/cmd-proving-detached-dark.png).

Region check (each capture is the detached toolbar webview only; no main-window
shell, editor, Command Manager row, or onboarding card is mounted):

1. **Host chrome:** a single compact row on the themed panel background. No tab
   strip, location field, Save control, pin/follow row, or content host.
2. **Dock control (left):** host `Dock in main window` returns the bar to the
   main Command Manager.
3. **Toolbar frame (centre):** drag handle (`⋮⋮`), bar title `Markdown proving`,
   and the mod command button with extension icon and `Apply subscript (proving)`
   label (detached bar uses command title; the in-window CM control reads
   `Proving chip` with the mod badge).
4. **Theme:** light capture uses Quiet Utility light panel/surfaces; dark capture
   uses the dark panel token set with the same layout.

### Contributions

Mods may:

- add commands;
- add controls to declared family extension points;
- contribute new groups or families;
- contribute Command Manager tabs for genuinely distinct contexts;
- suggest toolbars;
- make toolbars floatable or detachable to another monitor;
- provide direct handlers or emulations;
- provide value and mixed-selection state.

Mods may not:

- replace Save, Save As, autosave, or recovery;
- disable or cover the editor;
- open an overlay above content except through a host-owned permitted surface;
- move a toolbar outside recoverable screen bounds;
- hide core controls by capability response;
- capture global shortcuts outside the host keybinding map;
- use colour or an icon as the only state communication.

### Conflict resolution

Conflicts are detected at manifest validation:

- Duplicate command ID with incompatible metadata: contribution rejected.
- Compatible metadata for an existing command: handler/provider registration allowed; core label and semantics remain canonical.
- Duplicate control, group, family, or toolbar ID: lower-precedence contribution rejected.
- Multiple providers: deterministic provider precedence and diagnostics as described above.
- Multiple insertion requests at one extension point: order by explicit `before`/`after`, dependency order, priority, then mod ID.
- Cycles: involved contributions move to the end of the extension point and receive a resolver note.
- Keybinding conflicts: command remains available; shortcut is unbound until the user chooses.

### Ordering and priority

Manifest contributions use integer priority from `-100` to `100`, default `0`. Priority influences ties and overflow. It never grants security precedence or permission.

Core reserves IDs under `unibrowser.*`. Mods use their manifest name as namespace. A mod may extend a core family but may not publish new `unibrowser.*` IDs.

### Versioning

- `unibrowser_api` gates binary/API compatibility.
- Contribution schema has its own `command_ui_schema` semver.
- Unknown optional fields are ignored within a compatible minor version.
- Unknown control types or required fields reject only that contribution, not the entire mod when safe isolation is possible.
- Stable IDs survive label, icon, and implementation changes.
- Breaking family order or control semantics requires a major schema/API change or explicit migration.

### Trust and permissions

Permissions are least-privilege and surfaced before enablement:

| Permission | Grants |
| --- | --- |
| `commands.execute` | Execute registered command logic |
| `commands.capability.provide` | Resolve capability and selection state |
| `document.read.selection` | Read current selection snapshot |
| `document.edit` | Apply edits through the host transaction API |
| `ui.command_manager.contribute` | Add families, groups, controls, or tabs |
| `ui.toolbar.contribute` | Suggest host-rendered toolbars |
| `ui.toolbar.detach` | Allow host to detach a toolbar window |

Native T1 mods still require explicit trust. WASM providers receive only granted host imports. All edits pass through host transactions and undo. Capability resolution is read-only.

The host enforces reserved content insets. Detached toolbars are separate platform windows. In-window floating toolbars are clamped to non-content chrome or explicit user drag positions and cannot intercept the Save control.

## Keyboard, palette, and accessibility

### Unified reachability

Every visible control has:

1. Tab/Shift+Tab reachability by group.
2. Arrow-key movement within a group.
3. Home/End to first/last control.
4. Enter or Space for the main action.
5. Alt+Down for dropdown/split variants.
6. Escape to close menus and return to the invoking control.
7. The same command in the command palette.
8. An optional user-rebindable shortcut in the host keybinding map.

The command palette's standard shortcut remains Ctrl+Shift+P. The S key is not reserved, but the user may bind S to the palette or a toolbar through keybindings.

### Palette representation

Palette rows show:

- command label;
- family and context;
- current value/state where useful;
- shortcut;
- capability marker;
- unsupported or emulated explanation.

Unsupported rows remain focusable and do not execute. Emulated rows execute and disclose the approximation.

### Screen-reader states

Controls expose:

- accessible name and description;
- role appropriate to button, switch, combobox, spinbutton, radio group, or grid;
- `aria-pressed`, `aria-checked`, `aria-valuenow`, and `aria-valuetext` where applicable;
- mixed values as `aria-checked="mixed"` or `aria-valuetext="Mixed"`;
- unsupported as disabled plus described reason;
- emulated as described text, for example "Subscript, approximated with font scaling";
- shortcut through `aria-keyshortcuts` when representable.

Disabled controls remain discoverable in the Command Manager and palette. Focus order is stable because capability changes do not move controls.

## Worked examples

### Text family matrix

| Control | `.txt` | Markdown | Rich document | Code |
| --- | --- | --- | --- | --- |
| Font | Unsupported: plain text has no font metadata | Unsupported or emulated by inline HTML policy | Supported | Unsupported |
| Size | Supported as editor display size, not file formatting | Supported as editor display size; document formatting unsupported | Supported as document formatting | Supported as editor display size |
| Bold | Unsupported | Supported | Supported; mixed selection allowed | Unsupported |
| Italic | Unsupported | Supported | Supported; mixed selection allowed | Unsupported |
| Underline | Unsupported | Emulated with inline HTML when policy allows | Supported | Unsupported |
| Strikethrough | Unsupported, greyed in place | Supported | Supported | Unsupported |
| Subscript | Unsupported | Emulated, tooltip: "Approximated with font scaling" or inline `<sub>` | Supported | Unsupported |
| Superscript | Unsupported | Emulated, same disclosure | Supported | Unsupported |
| Text colour | Unsupported | Emulated with inline HTML when allowed | Supported | Unsupported |

In `.txt`, editor display Size is a separate display control state from document formatting. The label and tooltip must say "Editor text size" so no file-format claim is made.

### Plain text

The Text Formatting family appears in full. Bold through colour are greyed in place. Size remains active as editor display size. Strikethrough says:

> Plain text has no strikethrough formatting.

Save remains active regardless of diagnostics or unsupported formatting.

### Markdown

Bold, Italic, Strikethrough, and code-oriented controls are supported. Subscript is emulated by `markdown-baseline` and carries an approximation marker. A mixed selection across subscript and normal text reports Mixed. Applying Subscript normalises the selected span through one undoable host edit.

### Rich document

The document backend supports the full family. Mixed Bold and mixed font size render as mixed states. The font gallery may use a Large variant while retaining the same control ID and placement.

### Code

The family occupies the same position, but document formatting controls are unsupported. Editor font and size remain supported as view settings. Find and Proofing stay in their normal groups. Format is a code formatter command provided by an LSP/formatter mod, not the rich-text Format control.

### PDF annotation

**Runtime (current build):** The complete target control set is present. Highlight,
page, zoom, and export have providers. Tools without providers remain visible
and unsupported with a specific reason. The compact row prioritises Highlight,
Underline, and Strikethrough; lower-priority annotation, navigation, and
appearance groups remain available in overflow.

Families (full target):

- History: Undo, Redo.
- Selection: Select, Hand.
- Markup: Highlight, Underline, Strikethrough.
- Annotation: Note, Area, Ink, Text box, Stamp.
- Annotation navigation: Previous annotation, Next annotation.
- Appearance: Stroke colour, fill colour, width.
- View: Page, zoom.
- Export: Export PDF.

If the selected annotation has no fill, Fill Colour is unsupported with "This annotation type has no fill". When several annotations have different stroke colours, Stroke Colour is supported + mixed.

### Image

**Runtime (current build):** History, Image View, Transform, Adjust, and Export
are present. Raster Fit, Crop, Rotate, and zoom have viewer providers. Resize
and non-destructive adjustments remain visible and unsupported with a reason.
SVG uses the same family but reports raster-only operations as unsupported.

Families (full target):

- History.
- Select.
- Crop and transform.
- Adjust: exposure, contrast, colour.
- Text Formatting, only when a text layer is selected.
- Export.

With a bitmap selection, the Text Formatting family stays in its canonical tab position but is unsupported. With a text layer, it resolves according to the layer engine.

Mod authoring: see [command-system-authoring.md](../mods/command-system-authoring.md).

### CAD/3D

**Runtime (UI-14 proving build):** path detection recognises `.dxf`, STEP, and
IGES as `WorkContext::Cad3d`. The complete target family is present. Fit view,
Zoom out, and Zoom in have providers. Select, orientation, pan, orbit, Measure,
Section, Appearance, and Export STEP remain visible and unsupported with
provider-specific reasons. The bundled ASCII DXF fixture provides Outline and
Detail layers for shell and capability evidence.

Families (target):

- Select.
- View: orientation gallery, pan, orbit, zoom.
- Measure.
- Section.
- Appearance.
- Export.

A vendor mod may add `acme.cad.analysis` after Measure and provide a detachable Analysis toolbar on another monitor. The host renders and positions it. If the licence service is unavailable, controls remain visible and unsupported with a reason; model saving remains unaffected.

## Data contracts

The engine should expose one resolved model:

```text
ResolvedControl
  control_id
  family_id
  group_id
  control_type
  commands[]
  capability
  selection_state
  value
  reason
  emulation_note
  labels
  icon
  shortcut
  density_variant
  provider_id
  context_revision
```

Every surface consumes this model rather than recomputing enablement independently.

## Principles check

| Principle | Consequence |
| --- | --- |
| Trust the user | User can pin, customise, select providers, hide controls, detach toolbars, and inspect emulation |
| Never block saving | Command providers and diagnostics cannot gate Save, Save As, autosave, or recovery |
| Local-first | Resolution and persistence work without account or network |
| Predictable and consistent | Stable IDs, family order, and unsupported controls remain in place |
| Simple durable mechanism | Versioned data model, deterministic merge, JSON persistence |
| Mod over core | Core mediates; format semantics and emulations live in backends and mods |

## Resolved defaults (Bear may override)

1. **Markdown inline HTML emulation** is on by default so common formatting "just works"; the emulation marker is shown whenever a control is emulated.
2. **Emulation marker** is always visible but subtle (a small mark, never colour alone). Tooltip and accessible description explain the approximation.
3. **Text Formatting** is one family with movable groups (Font/Type, Emphasis, Baseline/Colour). Default geometry is identical across contexts.
4. **Hiding a control** applies to the active context by default, with an explicit "hide in all contexts" option in customisation.
5. **User-selected provider** outranks direct backend support (user taste wins). A visible **reset to default provider** restores backend precedence.
6. **Detached toolbars** remember stable monitor identity (display name or EDID plus last geometry) and fall back to the nearest visible monitor; they are never stranded off-screen.
7. **Code editor font and size** belong to **Editor Appearance** (view setting, not document formatting) but use the same position and geometry as the Text Formatting Type group.
8. **Licensed controls** use a distinct `locked` capability state (not `unsupported`), with neutral wording and no nag or upsell popups.
