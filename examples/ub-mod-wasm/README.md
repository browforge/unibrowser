# ubmod-example (WASM template)

Future home: **browbridge/ubmod-example**. Out-of-tree WASM mod using `unibrowser-mod-sdk`.

- Registers palette command `mod.wasm.demo`.
- Registers file extension `ubwasm` (stub text viewer via host).

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown
cp ../../target/wasm32-unknown-unknown/debug/ub_mod_wasm.wasm .
```

Copy this folder into your user mods directory with `info.json` and `ub_mod_wasm.wasm`, or rely on `cargo test -p unibrowser-mod-host` which builds and loads the artefact.
