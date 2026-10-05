# ADR 0002: Mod system and sandboxing

- **Status:** Accepted (tiered model; amended 2026-10-05 — compiled mod payload)
- **Date:** 2026-10-05

## Context

Factorio-style mods must load with dependencies and API version checks. Community mods must not compromise the host. **JSON manifest alone is not a mod format** — it cannot carry performant format logic, protect IP, or express a stable ABI. Mods are **compiled loadables** (native library and/or WASM) plus a **small manifest** for metadata only.

Alternatives for **payload** execution:

| Model | Pros | Cons |
| --- | --- | --- |
| **In-process native dylib** (`cdylib` / DLL / `.so`) | Full API; best performance; closed-source/paid mods possible | No sandbox; trust + signing required |
| **WASM (wasmtime + WIT / Extism)** | Sandboxed; portable; safer default for community | FFI overhead; GPU/video limits |
| **Out-of-process sidecar** | Strong isolation for GPL/AGPL tools | Latency; IPC complexity |
| **JS/TS in WebView (Tauri)** | VSIX / extension host path | Not the native mod ABI; separate lane ([ADR 0014](0014-vsix-compatibility-layer.md)) |

## Decision

### Manifest vs payload

| Part | Contents | Not allowed |
|------|-----------|-------------|
| **Manifest** (`info.json` or successor) | `name`, version, title, licence, `unibrowser_api`, dependencies, **entry point** (`library` / `wasm_module`), capabilities, signing metadata | Executable logic, scripts as the mod body |
| **Payload** | **Compiled** artifact: Windows `.dll`, Linux `.so`, macOS `.dylib`, and/or **`.wasm`** (tier-dependent) | Shipping only JSON and expecting the host to “interpret” behaviour |

**Skeleton exception (T0 dev):** bundled `base-*` mods in the repo may **statically register** via a built-in table for fast iteration. That is **not** the long-term community format — release and third-party mods ship **manifest + binary**.

### Tiered mod execution

| Tier | Who | Payload | Trust |
| --- | --- | --- | --- |
| **T0 Bundled base** | Ships with app | In-process native (static link or bundled `cdylib`) | Same as engine; **Apache-2.0 / FOSS** |
| **T1 Community native** | Vendors, paid/closed-source mods | Signed **native** library + capability manifest | Explicit user opt-in; publisher signing; limited host API surface |
| **T2 WASM** | Untrusted / marketplace default | **`.wasm`** + WIT imports | Sandboxed; preferred default for **community** mods when performance allows |
| **T3 Sidecar** | GPL/AGPL (Inkscape CLI, LOK, ffmpeg) | Separate process, JSON-RPC | Licence isolation; not a mod DLL |

### Proprietary and paid mods (future)

- **Core engine** and **bundled `base-*`** remain **FOSS** (Apache-2.0) and auditable.
- **T1 native** mods may be **closed-source or paid**; the host loads a binary + manifest, not source. Licence and publisher shown at install; user opts in.
- Paid distribution (store, keys) is **out of scope** for the engine; the architecture only **allows** binary mods and signing, not DRM in user files.

### Security and user space ([ADR 0013](0013-dual-audience-foss-and-everyday-users.md))

- **Signing / trust:** T1 requires signature verification (policy TBD); T2 WASM limits host imports.
- **No interference:** mods install to a user-controlled mods directory; uninstall removes manifest + binaries; no hidden data exfiltration channels in the base design.
- **Fail closed:** incompatible `unibrowser_api` major → refuse load with clear error.

Skeleton implements **T0** manifest discovery + **optional** `library` dynamic load + dev static registration; WASM and sidecars documented only.

## API versioning

Mods declare `unibrowser_api` in the manifest. Host exposes `UNIBROWSER_MOD_API_VERSION` (semver). Loader rejects incompatible mods (major mismatch; minor backward compatible per semver). **ABI stability** is defined at the `cdylib` export boundary (C ABI or stable Rust `extern "C"` shim) — see open questions.

## VSIX compatibility (separate lane)

**VSIX is not a native mod.** It is a **compat import** ([ADR 0014](0014-vsix-compatibility-layer.md)): unpack `package.json`, map contributions, optionally run JS in an **extension host**. VSIX may **feed** Monaco assets or command metadata; it may **not** be conflated with the **native mod ABI** (manifest + DLL/WASM). Adapted assets may be cached beside user mods; the VSIX lane does not replace `library` / `wasm_module` entry points.

Grandma-path: bundled **compiled** `base-*` only; no VSIX, no third-party DLL install prompts.

## Consequences

- [ADR 0006](0006-mod-manifest-and-versioning.md) describes metadata; community mods must ship a **binary** (or WASM) entry.
- Security reviews focus on T1 signing, T2 WIT allowlists, and sidecar IPC.
- Heavy AGPL editors stay in **T3**, never linked into `unibrowser-engine`.
- Documentation and tooling target **SDK → compile → `.ubmod` package** (manifest + artefact), not “drop a JSON mod”.

## Open questions

1. **ABI stability** — C vtable vs `abi_stable` vs versioned `extern "C"` registration function only?
2. **Default for community** — T2 WASM first, T1 native only after signing, or both from day one?
3. **`cdylib` per OS** — one package with all three binaries vs platform-specific downloads?
4. **Rust mods** — dynamic `cdylib` from Rust SDK only, or stable C ABI for other languages?
5. **Paid mods** — signature + licence field only, or integration with a store later?
