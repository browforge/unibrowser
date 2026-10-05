# ADR 0014: VS Code `.vsix` compatibility layer (goal)

- **Status:** Accepted (direction only; not implemented)
- **Date:** 2026-10-05

## Context

Bear’s direction: **where possible, reuse existing stores and community mods** (VS Code ecosystem, open registries, and eventually native **`.ubmod`** packages) without Unibrowser reimplementing every language pack and format backend.

### Terminology (do not conflate)

| Term | What it is |
|------|------------|
| **`.vsix`** | **Package file format** — a zip (OPC) containing `package.json`, assets, and optionally JS. Same container whether you sideload from disk or download from a store. |
| **Open VSX** | **Open marketplace / registry** ([open-vsx.org](https://open-vsx.org)) — publishes and indexes `.vsix` (and metadata) under an open model. Preferred **online catalog** for editor-style extensions in Unibrowser. |
| **Microsoft Visual Studio Marketplace** | **Different store** — also distributes extensions, often as `.vsix`, but **terms may forbid redistribution** or use outside VS Code-family products. Unibrowser must **not** assume we can repackage or proxy the Microsoft Marketplace. |
| **Native `.ubmod`** | Unibrowser’s **compiled mod** package (manifest + DLL/WASM) — separate catalog lane ([ADR 0002](0002-mod-system-and-sandboxing.md)). |

**Unibrowser installs `.vsix` files**; **Open VSX is one source** of those files. They are not the same thing.

Constraints:

- Unibrowser is **not** a VS Code fork. Most extensions call **`vscode.*` APIs** that assume the VS Code workbench, extension host, debug adapter protocol, and proprietary UI surfaces.
- **Native mods** (manifest + **compiled** DLL/WASM payload) remain **first-class** for formats, enablement, and deep integration ([ADR 0002](0002-mod-system-and-sandboxing.md), [ADR 0006](0006-mod-manifest-and-versioning.md)). `info.json` is metadata only — not the mod body.
- **Dual audience** ([ADR 0013](0013-dual-audience-foss-and-everyday-users.md)): bundled `base-*` covers common types; when something cannot open, a **built-in Mod Portal** offers installable capability **without jargon** (not “install a VSIX”).

## Decision

### Two mod lanes (not either/or)

| Lane | Format | Role |
|------|--------|------|
| **Native** | Manifest + **compiled** mod (T0–T3 per [ADR 0002](0002-mod-system-and-sandboxing.md)) | Formats, backends, commands with real enablement, CAD/media, ship with app |
| **VSIX compat** | `.vsix` (zip of `package.json` + assets) | **Import/adapt** contributions; optional extension host — **not** the native mod ABI |

VSIX does **not** replace manifest + binary mods. A VSIX may be **translated** into registry entries, Monaco assets, or cached data; deep behaviour still belongs in **compiled** native/WASM mods or T3 sidecars.

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
| **V4** | **Mod Portal** UI + Open VSX API integration (opt-in network); native `.ubmod` index when available | Browse/install like VS Code Extensions view |
| **V5** | Context recommendations + **missing-handler** install prompts | Grandma + power users |

Full **VS Code Extension Host** parity is **not** a goal. Goal is **high-value reuse** (grammars, commands, themes, format backends) with honest limits.

### Built-in Mod Portal (product goal)

A first-party **Extensions / Mods** surface in the shell (not a web-only store):

| Feature | Behaviour |
|---------|-----------|
| **Browse** | Search Open VSX (and later native registry) with compatibility badges (“grammar only”, “needs extension host”, “native mod”) |
| **Recommended for context** | Command Manager / document context suggests mods (e.g. on **Code** tab → formatters; on **Image** → raster tools) |
| **Cannot open this file** | User opens unknown or unsupported type → tab explains plainly → **“Install support”** lists mods that register that extension/MIME/backend — no MIME lecture in the headline |
| **Install** | Download `.vsix` or `.ubmod`, verify licence, install to user mod dir, reload handlers |
| **Sideload** | **Always allowed:** “Install from file…” (`.vsix`, `.ubmod`) — FOSS path without any store |

Portal copy is **capability-based** (“Open Photoshop files”, “Edit Word documents”), not “VSIX” or “Open VSX”.

### Catalog (decided — FOSS path)

Bear confirmed **2026-10-05**: Unibrowser takes the **FOSS catalog path** for built-in store integration.

| Decision | Detail |
|----------|--------|
| **Default online catalogs** | **[Open VSX](https://open-vsx.org)** for VS Code-compatible `.vsix`, plus a **future native `.ubmod` registry`** for compiled format mods (second lane in the same Mod Portal; ship when the index exists — not blocked on VSIX phases). |
| **Microsoft Visual Studio Marketplace** | **No embedded catalog** — redistribution / product ToS are incompatible with proxying or repackaging as a default store. Users may still **sideload** a `.vsix` they obtained elsewhere. |
| **Sideload** | **Always allowed** — “Install from file…” for `.vsix` and `.ubmod`; no account required. |
| **Mod Portal** | **Built-in** shell UI: browse default catalogs, **recommended for context**, and **“can’t open this file → install these mods”** ([sections above](#built-in-mod-portal-product-goal)). |

Implementation may gate **network** access (opt-in) for Open VSX; catalog **policy** is fixed.

### Distribution policy (summary)

| Source | Policy |
|--------|--------|
| **Open VSX** | **Default** embedded catalog for `.vsix`; public API; publisher, licence, compatibility badges. |
| **Native `.ubmod` registry** | **Default** when available; parallel lane, not a replacement for Open VSX. |
| **Microsoft Marketplace** | **Excluded** from embedded catalogs; sideload only. |
| **Sideload** | Always supported. |

Goal: **reuse open stores** where licence and ToS allow; **never** block sideloading.

### Sandboxing and trust

- VSIX install = **explicit user action**; list installed VSIX like mods with licence field from `package.json`.
- Prefer **T2 WASM** or **isolated worker** for JS from VSIX before any in-process native bridge ([ADR 0002](0002-mod-system-and-sandboxing.md)).
- Extensions that bundle native binaries follow **T1 signed native** rules or are rejected.

### Dual audience ([ADR 0013](0013-dual-audience-foss-and-everyday-users.md))

| Audience | Portal / extensions |
|----------|---------------------|
| **Everyday** | Bundled mods first; if a file fails, **Mod Portal** offers **“Install support”** in plain language — same UI hackers use, simpler defaults |
| **FOSS / power** | Open VSX browse, native `.ubmod` registry, **sideload**, inspectable install dirs, uninstall removes files |

Grandma never needs to hear “VSIX” or “Open VSX”; hackers can sideload and pin alternate catalogs.

### Relation to lazy conversion ([ADR 0011](0011-lazy-conversion-native-until-encode.md))

VSIX may contribute **language support** and **editor commands**; it does not define **document native models** or export pipelines. Format depth stays in native `info.json` mods.

## Consequences

- Roadmap gains VSIX phases parallel to native format mods.
- API gap process ([ADR 0008](0008-api-gap-upstream-requests.md)) tracks which `vscode` APIs extensions actually need.
- Licence compliance: each package’s licence must be shown on install; **Microsoft Marketplace ToS are not assumed** for redistribution.
- Mod Portal is a **shell feature** (Tauri UI + mod-host install), not a mandatory cloud account.

## Open questions

1. **Extension host topology** — in-process Node vs worker vs out-of-process RPC vs LSP-only adapters for language servers?
2. **Recommendation metadata** — who maintains **extension/MIME → mod id** map (bundled index vs mod manifest `provides` / `opens` fields vs Open VSX categories)?
3. **“Cannot open” resolution** — match on file extension only, MIME sniffing, or user-visible “treat as…” override?
4. **Licence** — MIT/Apache extensions vs copyleft; offline mirror of Open VSX vs online-only API?
5. **Microsoft `vscode` API trademark / compatibility** — document “Unibrowser VSIX subset” without implying Microsoft endorsement.
6. **Command ID collisions** — `publisher.name` prefixing vs raw VS Code ids for palette search?
7. **Engine vs desktop** — unpack in `unibrowser-mod-host` vs `unibrowser-desktop` only?

**Closed (catalog):** default catalogs = Open VSX + native `.ubmod` registry; no Microsoft Marketplace embed; sideload always; Mod Portal with context recommendations and missing-handler install prompts ([Catalog (decided — FOSS path)](#catalog-decided--foss-path)).

## Related

- [mod-system.md](../mod-system.md) — overview of native + VSIX lanes
- [ADR 0002](0002-mod-system-and-sandboxing.md) — execution tiers
- [ADR 0006](0006-mod-manifest-and-versioning.md) — `info.json` remains canonical for native mods
