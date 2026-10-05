# ADR 0019: WASM T2 sandbox (wasmtime + WIT)

- **Status:** Accepted (design + feature stub; loader not wired)
- **Date:** 2026-10-05
- **Amends:** [ADR 0002](0002-mod-system-and-sandboxing.md) (T2 execution detail)

## Context

T1 native mods trade performance for trust (signing, capability manifest). **T2 WASM** is the planned default for **untrusted community** mods: bytecode with a narrow import surface, no arbitrary host memory access.

## Decision

### Runtime

- **Engine:** [wasmtime](https://wasmtime.dev/) (Rust-first, fits the workspace).
- **Interface:** WIT guest world aligned with [`docs/plugin-sdk/ub-mod.wit`](../plugin-sdk/ub-mod.wit) and the existing C registration sketch.
- **Cargo feature:** `unibrowser-mod-host/wasm-mods` pulls optional `wasmtime`; `crates/unibrowser-mod-host/src/wasm.rs` is a **stub** until the loader calls `wasmtime::Engine`.

### Sandbox rules (T2)

| Resource | Policy |
| --- | --- |
| Filesystem | No host paths by default; future capability token per mod for explicit read/write roots |
| Network | Denied unless manifest capability `network` + user opt-in |
| Environment | No `env` inheritance; only host-injected config keys listed in manifest |
| Native code | WASM only; no `dlopen` from guest |
| CPU / memory | Fuel or epoch interruption + memory limits (wasmtime config) |
| Commands | Register only through WIT `command-registration` import; host validates ids |

### Package layout

Same user mods directory as native ([plugin SDK](../plugin-sdk/README.md)):

- `info.json` with `"wasm_module": "mod.wasm"` (or `library` reserved for native)
- Optional `ub-mod.sig` under T1/T2 signing policy ([signing.md](../plugin-sdk/signing.md))

Host loads **either** native `library` (T1) **or** `wasm_module` (T2), not both in one package.

### Failure modes

- Incompatible `unibrowser_api` → refuse load (same as manifest loader today).
- WASM trap → disable mod for session; surface error in mod portal.
- Missing import → refuse load at instantiate time.

## Consequences

- Community marketplace path defaults to T2 without giving guest code the full `cdylib` vtable.
- T1 remains for signed native performance mods.
- CI does not enable `wasm-mods` by default (optional dependency weight).

## Open questions

1. Extism vs raw wasmtime for plugin ergonomics?
2. Shared engine pool vs per-mod isolate?
3. GPU / video decode inside WASM (likely denied; route to host capability)?
