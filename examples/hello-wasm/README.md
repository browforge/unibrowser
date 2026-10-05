# Hello WASM proving package

Minimal **T2** proving artefact for the blitz inch. The desktop host does **not** instantiate this module with wasmtime yet; when `hello.wasm` is present in the repo tree, startup registers palette command `mod.wasm.hello` (no-op message).

See [docs/wasm-t2-status.md](../../docs/wasm-t2-status.md) and [ADR 0019](../../docs/adr/0019-wasm-t2-sandbox.md).

`hello.wasm` is an empty valid WebAssembly module (magic + version only).
