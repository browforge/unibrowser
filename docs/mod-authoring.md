# Mod authoring

Unibrowser mods extend the shell with **commands**, **file handlers**, and (later) capabilities. Three lanes:

| Lane | Artefact | Trust | When to use |
| --- | --- | --- | --- |
| **Native (T1)** | `info.json` + platform `library` (`.so`/`.dll`) | Signing policy + user opt-in | Performance, deep OS integration (Linux user `mods/` today) |
| **WASM (T2)** | `info.json` + `wasm_module` | wasmi sandbox + manifest | Cross-platform community mods |
| **VSIX** | `.vsix` unpacked under extensions dir | Trust prompt + Node host spike | Editor themes, grammars, light command metadata |

Manifest-only JSON describes identity and deps; **executable logic** lives in the compiled entry (`library` or `wasm_module`), not in `info.json`.

## Native cdylib (Linux proving path)

1. Copy [examples/ub-mod-cdylib-stub](../examples/ub-mod-cdylib-stub/).
2. Implement `ub_mod_register` and host callbacks ([plugin-sdk/ub-mod.h](plugin-sdk/ub-mod.h)).
3. Install under `<config>/mods/<name>/` with `info.json` + `lib….so`.
4. Enable in mod portal; restart if the host reports lingering commands.

See [plugin-sdk/README.md](plugin-sdk/README.md).

## WASM mod

1. Scaffold:

   ```bash
   cargo run -p unibrowser-cli -- mod new wasm my-mod /tmp/my-mod
   # or: ./scripts/ub-mod-new-wasm.sh my-mod /tmp/my-mod
   ```

2. Edit `src/lib.rs` (command ids, `register_file_extension`).
3. Build:

   ```bash
   cd /tmp/my-mod
   cargo build --target wasm32-unknown-unknown
   cp ../target/wasm32-unknown-unknown/debug/my_mod.wasm .
   ```

4. Ship folder: `info.json`, `my_mod.wasm`, optional `ub-mod.sig`.
5. Host loads via `unibrowser-mod-host` + [unibrowser-mod-sdk](../crates/unibrowser-mod-sdk/).

Reference: [examples/ub-mod-wasm](../examples/ub-mod-wasm/), [wasm-t2-status.md](wasm-t2-status.md).

## VSIX

1. Package a VSIX or sideload from mod portal.
2. Contributions map to Command Manager / themes / grammars ([extension-host.md](extension-host.md)).
3. JS `main` runs only when Node host pref is on; not a replacement for native/WASM mods.

## Testing

- `cargo test -p unibrowser-mod-host` (WASM load test).
- `cargo test -p unibrowser-cli` (scaffold smoke).
- Desktop: install mod → palette command → open a registered extension.

## Related

- [mod-system.md](mod-system.md)
- [vision-matrix.md](vision-matrix.md)
