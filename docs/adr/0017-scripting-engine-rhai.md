# ADR 0017: Scripting engine — Rhai first sketch

- **Status:** Accepted (direction for `base-js` spike)
- **Date:** 2026-10-05

## Context

Unibrowser needs a **scripting lane** (`base-js` proving mod) for user automation and future extension glue. Candidates: **QuickJS**, **Rhai**, **Extism** (WASM).

## Decision

**First in-process sketch: [Rhai](https://github.com/rhai-rs/rhai)** (Apache-2.0 / MIT dual-licensed engine, pure Rust).

| Option | Why / why not (MVP) |
|--------|---------------------|
| **Rhai** | Apache-friendly; no GPL sidecar; embeds in Tauri/Rust host; good for **trusted** bundled scripts; sandbox via limited API surface |
| **QuickJS** | Closer to real JS; heavier FFI; licence OK but more bridge work for v1 spike |
| **Extism** | Best for **untrusted** community mods (T2); defer as default until `base-js` needs marketplace-grade isolation |

`base-js` will expose Rhai with a **curated host API** (commands, document read-only hooks) before any full JavaScript compatibility claim.

## Consequences

- No Rhai dependency in core until `base-js` mod crate lands; this ADR records the pick.
- AGPL/GPL runtimes remain **out of process** ([ADR 0007](0007-licensing.md)).

## Related

- [architecture.md](../architecture.md) — scripting engine row
- [ADR 0002](0002-mod-system-and-sandboxing.md) — T2 WASM for untrusted mods later
