# WASM T2 loader status

| Item | Status |
| --- | --- |
| ADR | [0019](adr/0019-wasm-t2-sandbox.md) accepted |
| WIT sketch | [plugin-sdk/ub-mod.wit](plugin-sdk/ub-mod.wit) |
| Runtime | **wasmi** behind `unibrowser-mod-host/wasm-mods` (default on) |
| Guest SDK | [crates/unibrowser-mod-sdk](../crates/unibrowser-mod-sdk/) (`ub_host` imports) |
| Example mod | [examples/ub-mod-wasm](../examples/ub-mod-wasm/) — `mod.wasm.demo` + `.ubwasm` stub viewer |
| Proving package | [examples/hello-wasm](../examples/hello-wasm/) empty module + legacy `mod.wasm.hello` stub |
| User install | `<config>/mods/<name>/info.json` with `wasm_module` (see `load_user_wasm_mods`) |

## Not done yet

- Full WIT component model (today: `ub_wasm_register` + host imports).
- Fuel/memory limits and mod-portal trap UX.
- Signing policy enforcement distinct from native T1.

Build the example:

```bash
cargo build -p ub-mod-wasm --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/debug/ub_mod_wasm.wasm examples/ub-mod-wasm/
```
