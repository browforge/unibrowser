# Mod system overview

Unibrowser is a **platform**: plugins (mods) can add formats, features, views, panels, toolbars, commands, scripts, providers, and services — not only “open this extension”. Product framing: [positioning.md](positioning.md). Platform contracts: [ADR 0028](adr/0028-platform-content-kinds-and-plugin-scope.md).

Mods are **compiled loadables** (DLL / `.so` / `.dylib` and/or **WASM**), not JSON programs. A small **manifest** (`info.json`) holds metadata, dependencies, API version, and **entry points** only ([ADR 0002](adr/0002-mod-system-and-sandboxing.md), [ADR 0006](adr/0006-mod-manifest-and-versioning.md)).

**VS Code `.vsix`** is a separate **compat import** lane ([ADR 0014](adr/0014-vsix-compatibility-layer.md)) — do not conflate with the native mod ABI.

**Engines vs mods:** the product ships **host engines** (text, document, raster, vector, …); each lane has a **proving `base-*` mod**. **One mod may use several engines** (e.g. PDF → vector + raster + document). See [architecture.md](architecture.md).

**Content kinds:** tools declare which **content kinds** they target (image, rich text, audio clip, …). Format mods expose **content-kind trees** (including embedded nodes). Capability is resolved per kind and per **host format** — see [command-system.md](design/command-system.md).

## Native mods (first-class)

### Package shape (target)

```
my-mod/
  info.json              # metadata, deps, unibrowser_api, library / wasm_module
  libub_mod_my_mod.so    # Linux payload (platform-specific names on Win/macOS)
  # optional: mod.wasm for T2-only distribution
```

| Audience | What they get |
|----------|----------------|
| **Everyone (grandma)** | Prebuilt **bundled `base-*`** binaries shipped with the app (FOSS, Apache-2.0) |
| **Power / FOSS** | Optional community mods: compile or install a package; inspect manifest + trust signing |
| **Vendors (future)** | **Closed-source or paid** T1 native mods: binary + manifest; core stays open |

### Dev skeleton (today)

In-tree `mods/base-*` still use **Rust source + static registration** for fast CI. That is a **T0 ergonomics exception**, not the community mod format.

### Execution tiers

| Tier | Payload | Typical use |
|------|---------|-------------|
| T0 | Bundled native | `base-text`, `base-web`, … |
| T1 | Signed native DLL/so | CAD, commercial codecs, paid mods |
| T2 | WASM | Default target for **untrusted community** mods |
| T3 | Sidecar process | GPL/AGPL tools (ffmpeg, LOK) |

### Security

- **Signing** for T1 native (policy TBD).
- **WASM allowlist** for T2 host imports.
- **User space:** mods live in a user-visible directory; uninstall deletes manifest + binaries; no required cloud ([ADR 0013](adr/0013-dual-audience-foss-and-everyday-users.md)).

## VSIX vs Open VSX vs portal

| Concept | Meaning |
|---------|---------|
| **`.vsix`** | **File format** (zip + `package.json`). Install via sideload or download. |
| **Open VSX** | **Open marketplace** that publishes `.vsix` packages — preferred online source ([open-vsx.org](https://open-vsx.org)). Not the same as the file format. |
| **Microsoft Marketplace** | **Not** an embedded catalog (ToS / redistribution). Users may sideload their own `.vsix`. |
| **Mod Portal** | Built-in **Extensions-style** UI: browse default catalogs, install, **recommended for context**, **“can’t open this file → install these mods”**. |

See [ADR 0014](adr/0014-vsix-compatibility-layer.md).

### Catalog policy (decided — FOSS path)

- **Default catalogs in the portal:** [Open VSX](https://open-vsx.org) + **future native `.ubmod` registry** (format mods; added when the index ships).
- **Offline stub catalog (blitz):** `list_mod_catalog` filters bundled seed rows (Open VSX ids + `.ubmod` ids) locally in the mod portal; live search still calls Open VSX when network is opted in. **Install from file** is always available.
- **No Microsoft Visual Studio Marketplace** integration as a built-in store.
- **Sideload** `.vsix` and `.ubmod` always allowed (Install from file).
- Portal behaviour above is **product scope**, not optional “power only”.

## VSIX compat lane

- Import path for VS Code-style extensions — contributions and/or extension host.
- May **compile or adapt** into Monaco assets or registry entries; does **not** replace `library` / `wasm_module` native mods.
- **Sideload** `.vsix` / `.ubmod` always allowed; **Open VSX** is the default online `.vsix` catalog (network may be opt-in at runtime).

## Mod Portal (everyone + power users)

- **Grandma:** unsupported file → **“can’t open this file”** → installable mods that add the capability (plain copy).
- **FOSS:** same portal — Open VSX + native registry browse when live, plus **Install from file**.
- **Context recommendations:** built-in; suggest mods from Command Manager / document context (handler metadata still TBD — [ADR 0014](adr/0014-vsix-compatibility-layer.md) open questions).

## Choosing native vs VSIX

| Need | Prefer |
|------|--------|
| New file format / timeline / backend | **Native** compiled mod (T0–T2) |
| Tool for a content kind (colour profile on any image node) | Native mod + `content_kinds` + capability provider |
| Dock panel, status item, or service | Native mod with `ui.panel.contribute` / `ui.status.contribute` / `service.register` |
| Syntax highlighting | VSIX import (V2) or WASM/native grammar mod |
| Cross-format command with honest capability | Native handler + content-kind declarations |
| Reuse VS Code extension (metadata only) | VSIX V1 |
| GPL office kernel | T3 sidecar |

## Loading order

1. Virtual `base` mod  
2. Topological sort by manifest dependencies  
3. Load **payload** per tier (`dlopen` / WASM instantiate / sidecar spawn)  
4. Call exported **register** entry (ABI TBD)  
5. (Future) VSIX contributions merged after native registration where configured  

## See also

- [architecture.md](architecture.md)
- [api-gap-upstream.md](api-gap-upstream.md)
- [roadmap.md](roadmap.md)
