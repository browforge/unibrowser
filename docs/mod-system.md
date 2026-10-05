# Mod system overview

Unibrowser mods are **compiled loadables** (DLL / `.so` / `.dylib` and/or **WASM**), not JSON programs. A small **manifest** (`info.json`) holds metadata, dependencies, API version, and **entry points** only ([ADR 0002](adr/0002-mod-system-and-sandboxing.md), [ADR 0006](adr/0006-mod-manifest-and-versioning.md)).

**VS Code `.vsix`** is a separate **compat import** lane ([ADR 0014](adr/0014-vsix-compatibility-layer.md)) — do not conflate with the native mod ABI.

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
| **Microsoft Marketplace** | Separate store; **do not** treat as default catalog (redistribution / ToS risk). Users may sideload their own `.vsix`. |
| **Mod Portal** | Built-in **Extensions-style** UI: browse, install, **recommended for context**, **“install support for this file”** when nothing can open it. |

See [ADR 0014](adr/0014-vsix-compatibility-layer.md).

## VSIX compat lane

- Import path for VS Code-style extensions — contributions and/or extension host.
- May **compile or adapt** into Monaco assets or registry entries; does **not** replace `library` / `wasm_module` native mods.
- **Sideload** `.vsix` always allowed; **Open VSX** optional network catalog; native **`.ubmod`** registry later.

## Mod Portal (everyone + power users)

- **Grandma:** unsupported file → friendly prompt → install a mod that adds the capability (no jargon).
- **FOSS:** same portal plus Open VSX browse and **Install from file** for `.vsix` / `.ubmod`.
- **Context recommendations:** suggest mods when Command Manager context or active document type matches (metadata TBD).

## Choosing native vs VSIX

| Need | Prefer |
|------|--------|
| New file format / timeline / backend | **Native** compiled mod (T0–T2) |
| Syntax highlighting | VSIX import (V2) or WASM/native grammar mod |
| Cross-app command with real enablement | Native handler in compiled mod |
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
