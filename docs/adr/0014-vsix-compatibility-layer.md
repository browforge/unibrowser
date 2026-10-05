# ADR 0014: VS Code `.vsix` compatibility layer (goal)

- **Status:** Accepted (direction only; not implemented)
- **Date:** 2026-10-05

## Context

Bear’s direction: **where possible, reuse preexisting mods**, including **VS Code `.vsix` packages**, so power users can leverage a large ecosystem without Unibrowser reimplementing every language pack and command extension.

Constraints:

- Unibrowser is **not** a VS Code fork. Most extensions call **`vscode.*` APIs** that assume the VS Code workbench, extension host, debug adapter protocol, and proprietary UI surfaces.
- **Factorio-native mods** (`info.json`, Rust handlers, document backends) remain the **first-class** model for formats, enablement, and deep integration ([ADR 0006](0006-mod-manifest-and-versioning.md)).
- **Dual audience** ([ADR 0013](0013-dual-audience-foss-and-everyday-users.md)): grandma uses bundled `base-*` mods; **`.vsix` is invisible** in the default UI (advanced / power-user install only).

## Decision

### Two mod lanes (not either/or)

| Lane | Format | Role |
|------|--------|------|
| **Native** | `info.json` + Rust (T0–T3 per [ADR 0002](0002-mod-system-and-sandboxing.md)) | Formats, backends, commands with real enablement, CAD/media, ship with app |
| **VSIX compat** | `.vsix` (zip of `package.json` + assets) | **Import/adapt** contributions into host surfaces; optional thin JS host later |

VSIX does **not** replace `info.json`. A VSIX may be **translated** into native command registrations and Monaco assets; it does not get direct engine internals.

### Compatibility layer goal

1. **Unpack** `.vsix` (Open Packaging Conventions zip).
2. **Parse** `package.json` `contributes` and `activationEvents`.
3. **Map** each contribution point to Unibrowser machinery (honour, shim, or reject with reason).
4. **Never** silently pretend unsupported APIs work — log and surface in a power-user “extension diagnostics” view (future).

### Contribution points (initial mapping)

| VS Code `contributes` | Unibrowser target | Target handling |
|----------------------|-------------------|-----------------|
| `commands` | `CommandRegistry` + palette / Command Manager | **Honour** (static metadata; handler may be stub until host exists) |
| `keybindings` | `KeybindingMap` merge | **Honour** (chord conflicts → user wins, same as VS Code merge rules) |
| `menus` | Command surfaces / context menus | **Shim** (subset: command palette + manager only at first) |
| `languages` | Monaco `languageId` | **Honour** |
| `grammars` | TextMate → Monaco | **Honour** |
| `snippets` | Monaco snippet contrib | **Honour** |
| `themes` | Monaco theme JSON | **Shim** (map to Monaco themes, not global app theme) |
| `configuration` | Settings JSON (future) | **Shim** (namespace under `unibrowser.vsix.<id>`) |
| `views` / `viewsContainers` | Side bar / panels | **Unsupported** v1 (no VS Code activity bar) |
| `customEditors` / `notebooks` | Document backends | **Unsupported** v1 → use native mods |
| `debuggers` / `task` / `terminal` | — | **Unsupported** |
| `authentication` | — | **Unsupported** (no silent Microsoft login) |
| Extension `main` / `browser` + `vscode` API | Extension host | **Shim** later (see phases) |

**Runtime `vscode.*` calls** (e.g. `window.createStatusBarItem`, `workspace.getConfiguration`, LSP client): require an **extension host** with a **curated API surface** — not full parity.

### Phased delivery

| Phase | Scope | User-visible |
|-------|--------|--------------|
| **V0** | This ADR + mapping table | Docs only |
| **V1** | Unpack `.vsix`; `contributes.commands`, `keybindings`; register metadata; commands without JS → palette-only or “not implemented” enablement | Power: “Install VSIX (experimental)” |
| **V2** | `languages`, `grammars`, `snippets`, `themes` → Monaco path | Better syntax highlighting for text/code tabs |
| **V3** | Minimal **extension host** (Tauri worker or Node sidecar): `activate(context)` with **stub `vscode` module** (commands, subscriptions, `languages.register*`) | Some lightweight extensions run |
| **V4** | Open VSX **optional** catalog (user opt-in); signing/trust policy for T1 | Browse community extensions (advanced) |

Full **VS Code Extension Host** parity is **not** a goal. Goal is **high-value reuse** (grammars, commands, themes) with honest limits.

### Sandboxing and trust

- VSIX install = **explicit user action**; list installed VSIX like mods with licence field from `package.json`.
- Prefer **T2 WASM** or **isolated worker** for JS from VSIX before any in-process native bridge ([ADR 0002](0002-mod-system-and-sandboxing.md)).
- Extensions that bundle native binaries follow **T1 signed native** rules or are rejected.

### Dual audience ([ADR 0013](0013-dual-audience-foss-and-everyday-users.md))

| Audience | VSIX |
|----------|------|
| **Everyday** | Never prompted; bundled native mods only |
| **FOSS / power** | Optional `.vsix` / Open VSX; inspectable install dir; uninstall removes files |

### Relation to lazy conversion ([ADR 0011](0011-lazy-conversion-native-until-encode.md))

VSIX may contribute **language support** and **editor commands**; it does not define **document native models** or export pipelines. Format depth stays in native `info.json` mods.

## Consequences

- Roadmap gains VSIX phases parallel to native format mods.
- API gap process ([ADR 0008](0008-api-gap-upstream-requests.md)) tracks which `vscode` APIs extensions actually need.
- Licence compliance: each VSIX’s `package.json` licence must be shown on install; marketplace ToS are not assumed.

## Open questions

1. **Extension host topology** — in-process Node vs worker vs out-of-process RPC vs LSP-only adapters for language servers?
2. **Open VSX** — default catalog vs bring-your-own `.vsix` only? Curation, malware, and AGPL extension bundles?
3. **Licence** — MIT/Apache extensions vs copyleft vs marketplace-redistributable; ship Open VSX mirror or online-only?
4. **Microsoft `vscode` API trademark / compatibility** — document “Unibrowser VSIX subset” without implying Microsoft endorsement.
5. **Command ID collisions** — `publisher.name` prefixing vs raw VS Code ids for palette search?
6. **Engine vs desktop** — unpack in `unibrowser-mod-host` vs `unibrowser-desktop` only?

## Related

- [mod-system.md](../mod-system.md) — overview of native + VSIX lanes
- [ADR 0002](0002-mod-system-and-sandboxing.md) — execution tiers
- [ADR 0006](0006-mod-manifest-and-versioning.md) — `info.json` remains canonical for native mods
