# ADR 0002: Mod system and sandboxing

- **Status:** Accepted (tiered model)
- **Date:** 2026-10-05

## Context

Factorio-style mods must load with dependencies and API version checks. Community mods must not compromise the host. Alternatives:

| Model | Pros | Cons |
| --- | --- | --- |
| **In-process native dylib** | Full API; best performance; simple for `base` mods | No sandbox; trust required |
| **WASM (wasmtime + WIT / Extism)** | Sandboxed; portable | Component Model still maturing; FFI overhead; hard for GPU/video |
| **Out-of-process sidecar** | Strong isolation for GPL/AGPL tools | Latency; IPC complexity |
| **JS/TS in WebView (Tauri)** | Matches editor ecosystem | Sandboxed by web model; not for native CAD kernels |

## Decision

**Tiered mod execution:**

| Tier | Who | Runtime | Trust |
| --- | --- | --- | --- |
| **T0 Bundled base** | Ships with app | In-process native (Rust `cdylib` or static registration) | Same as engine |
| **T1 Community native** | Power users, CAD vendors | Signed native mod + capability manifest | User opt-in; limited host APIs |
| **T2 WASM** | Untrusted / marketplace | **wasmtime** with WASI preview + host WIT imports | Default for third-party in Phase 4 |
| **T3 Sidecar** | GPL/AGPL (Inkscape CLI, LOK, ffmpeg) | Separate process, JSON-RPC | Licence isolation |

Skeleton implements **T0** with manifest loading and optional `cdylib` entry symbol; WASM and sidecars are documented only.

## API versioning

Mods declare `api_version` in `info.json`. Host exposes `UNIBROWSER_MOD_API_VERSION` (semver). Loader rejects incompatible mods with a clear error (major mismatch; minor backward compatible per semver rules).

## VSIX compatibility (separate lane)

Native **`info.json` mods remain first-class** for formats, enablement, and trust tiers above.

**VS Code `.vsix` packages** are an optional **import/compat path**: unpack `package.json` contributions and map them to Unibrowser command registry, keybindings, and Monaco (phased). They do **not** replace manifests or T0–T3 execution rules; JS extension code requires a future sandboxed **extension host** with a **subset** of `vscode.*` APIs.

Grandma-path installs use bundled native mods only; VSIX is power-user opt-in ([ADR 0013](0013-dual-audience-foss-and-everyday-users.md)).

Full mapping, phases, and limits: [ADR 0014](0014-vsix-compatibility-layer.md). Overview: [mod-system.md](../mod-system.md).

## Consequences

- Base mods in repo use the same manifest format as future community mods.
- Security reviews focus on T1 signing and T2 WASM host imports.
- Heavy AGPL editors never link into `unibrowser-engine`.
- VSIX reuse is pursued where contribution points align; unsupported VS Code APIs fail visibly, not silently.
