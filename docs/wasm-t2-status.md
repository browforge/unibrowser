# WASM T2 loader status

| Item | Status |
| --- | --- |
| ADR | [0019](adr/0019-wasm-t2-sandbox.md) accepted |
| WIT sketch | [plugin-sdk/ub-mod.wit](plugin-sdk/ub-mod.wit) |
| `unibrowser-mod-host/wasm.rs` | Stub; `wasm-mods` feature flag (no wasmtime yet) |
| Proving package | [examples/hello-wasm](../examples/hello-wasm/) |
| Proving command | `mod.wasm.hello` registered when `hello.wasm` is found at dev paths |
| Guest execution | **Not wired** — command returns a host message only |

## Next engineering steps

1. Optional `wasmtime` behind `wasm-mods`.
2. Parse `info.json` `wasm_module` and instantiate WIT exports.
3. Fuel/memory limits and mod-portal error surfacing on trap.

Community mods should continue to use native T1 (signed) or wait for T2; do not ship production logic in the proving stub.
