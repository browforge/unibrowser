# Command system authoring for mods

This guide complements [command-system.md](../design/command-system.md) for mod authors contributing commands, controls, families, toolbars, panels, status items, and capability providers. Platform scope: [ADR 0028](../adr/0028-platform-content-kinds-and-plugin-scope.md).

## Content kinds

Declare `content_kinds` on commands, capability providers, and kind-specific panels. Use core ids (`unibrowser.content.image`, `unibrowser.content.rich_text`, …) or register extension kinds under your mod namespace.

Format mods should expose **embedded nodes** (image inside DOCX, inline asset in Markdown) so kind-targeted tools work without export round trips. Capability is **per content kind and per host format** (supported, emulated, unsupported with reason).

## Provider resolution

1. The host builds a **resolution context** from the active document (host format, mode, selection mapped to **content-kind nodes**, work context).
2. For each control, the host computes a **base capability** from selection kinds and host format (not from file extension alone).
3. **Capability providers** (native or WASM) may override capability, selection, value, and emulation notes for controls in their declared families and kinds.
4. **User provider overrides** win over direct backend support until the user resets to the default provider.
5. Provider callbacks are read-only for capability snapshots and bounded by time; they cannot block Save or cover content.

Register a provider in your mod manifest:

```json
"capability_providers": [{
  "id": "my-mod::native",
  "families": ["unibrowser.family.image-adjust"],
  "content_kinds": ["unibrowser.content.image"],
  "host_formats": ["png", "docx"],
  "runtime": "native",
  "priority": 10
}]
```

## Capability states

| State | Meaning for authors |
| --- | --- |
| `supported` | Host or provider implements the command for this context. |
| `emulated` | Behaviour is approximated; set a clear `note` and expect the subtle emulation marker in the UI. |
| `unsupported` | Control stays visible and greyed; set a short `reason` string for tooltip and screen readers. |
| `locked` | Licence or policy gate; neutral wording only, no upsell modals. |
| `hidden_by_user` | User hid the control in this context; respect layout persistence. |

Save, Save As, autosave, and recovery paths always resolve as **supported** regardless of formatting diagnostics or provider failures.

## Emulation

- Use emulation when your backend approximates a core control (for example Markdown subscript via inline HTML).
- The host shows a **visible-but-subtle** emulation mark on the control; tooltip and `aria-description` carry your note.
- Emulated controls remain actionable when policy allows; edits go through host transactions and undo.

## Permissions

Declare only what you need in `capabilities`:

| Permission | Use |
| --- | --- |
| `commands.capability.provide` | Register a capability provider |
| `document.read.selection` | Read selection for snapshots |
| `document.edit` | Return host edit requests from execute callbacks |
| `ui.command_manager.contribute` | Insert controls at declared points |
| `ui.toolbar.contribute` | Add floating toolbars |
| `ui.toolbar.detach` | Allow detached toolbar windows |
| `ui.panel.contribute` | Docked panel in an allowed region |
| `ui.status.contribute` | Status-bar item in a fixed slot |
| `content.read.tree` / `content.read.selection` | Traverse or read selection as kind nodes |
| `content.edit` | Host transactions on kind nodes |
| `provider.render` / `provider.analysis` | Kind-scoped provider hooks |

Missing permission means the host ignores the contribution and may record a diagnostic.

## Diagnostics catalogue

Non-modal diagnostics appear on the Command Manager context control (`title` / `aria-description`). They are sorted deterministically:

| Code | Cause |
| --- | --- |
| Duplicate control or family ID | Two mods claimed the same stable ID |
| Order cycle | `insert.after` / `before` graph has a cycle |
| Keybinding conflict | A chord displaced an existing binding (old binding listed) |

Fix manifests or IDs; diagnostics never block saving.

## Trust rules

- Mods cannot remove or disable host Save controls.
- Mods cannot inject arbitrary HTML into the shell; only declared controls and toolbars render.
- Detached toolbars are host windows; geometry is clamped to visible monitors.
- Native mods are trusted process code; WASM mods run under fuel and timeout limits with the same API surface.
- VSIX and native mods are listed separately; users enable or disable per mod.

## Further reading

- [command-system.md](../design/command-system.md) — worked examples and data contracts
- [ADR 0027](../adr/0027-command-system-capabilities-and-extensibility.md)
- [ADR 0028](../adr/0028-platform-content-kinds-and-plugin-scope.md)
- [api-gap-upstream.md](../api-gap-upstream.md) — request new host APIs upstream
