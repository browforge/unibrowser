# ADR 0025: Mod APIs and dependencies

- **Status:** Accepted (engine design + proving tests)
- **Date:** 2026-10-06

## Context

Mods should compose like Factorio interfaces or npm packages: an audio mod exposes **waveform / mix** surfaces that a video mod consumes, without hard-coding crate names. Manifests need versioned **`provides`**, semver **`dependencies`**, optional deps, and **incompatibilities**. The host must resolve load order, report conflicts clearly, and **never block** the shell when an optional API is missing. Every document type must expose **text** and **raster** projections for search, OCR, accessibility, and AI (ADR 0026).

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes — optional APIs degrade; user sees resolver notes, not modal blockers |
| Never blocks save or work | Yes — missing optional mod/API does not gate save or editing |
| Local-first | Yes — no cloud in resolver; APIs are in-process |
| Predictable across formats | Yes — projections are uniform per document backend |
| Simple durable mechanism | Yes — manifest + semver + host registry |
| Mod not core | Yes — APIs are mod-provided; host only mediates |

## Decision

1. **Manifest** (`info.json`): `provides` (API id → version), `dependencies`, `optional_dependencies`, `incompatibilities`, `api_dependencies` (API id → semver range). Factorio-style: `"base >= 1.2"`, `"? audio-core ^2"` (optional), `"! old-mod"` (incompat).
2. **Resolver** (`unibrowser-mod-host::deps`): load order, version checks, incompatibility errors, **optional** deps as non-fatal notes, **API** dependency binding to a providing mod.
3. **Typed API registry** (`ModApiRegistry`): host-mediated calls (WASM import / cdylib vtable / VSIX shim later). Proving: `audio.levels@1` in tests.
4. **Projections** (`unibrowser_mod_api::projections`): each backend offers `text_projection()` and optional `raster_projection()`; basis for AI, OCR, search, a11y.
5. **Proving**: mod A provides `audio.levels@1`; mod B consumes it; missing API → graceful degrade with `ModResolveNote`.

## Consequences

- Bundled mods gain empty `provides` by default; new fields are optional in JSON.
- Desktop loads unchanged until mods declare APIs.

## References

- [ADR 0006](0006-mod-manifest-and-versioning.md), [ADR 0026](0026-ai-assistant-as-mod.md)
- `crates/unibrowser-mod-api`, `crates/unibrowser-mod-host/src/api_registry.rs`
