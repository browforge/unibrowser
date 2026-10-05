# Unibrowser Plugin SDK (draft)

Compiled mods ship as native libraries (`cdylib` on Linux/macOS, `dll` on Windows) that export a small C ABI. The host loads them when enabled (Linux today: `UNIBROWSER_LOAD_CDYLIB_MOD=1`). Rust mods can use the same symbols as the reference stub; other languages link against [`ub-mod.h`](ub-mod.h).

This draft sits beside the stable Rust surface in [`unibrowser-mod-api`](../mod-api-stable.md). Manifest and versioning rules: [ADR 0006](../adr/0006-mod-manifest-and-versioning.md).

## Quick start (Linux)

```bash
cargo build -p ub-mod-cdylib-stub
export UNIBROWSER_LOAD_CDYLIB_MOD=1
export UNIBROWSER_CDYLIB_MOD_PATH=examples/ub-mod-cdylib-stub/target/debug/libub_mod_cdylib_stub.so
# run unibrowser-desktop; command palette should list "Hello from mod" (id `mod.hello`)
```

## Required exports (ABI v1)

| Symbol | Role |
| --- | --- |
| `ub_mod_loader_abi_version()` | Must return `1` |
| `ub_mod_register(host)` | Register commands via host vtable |
| `ub_mod_api_version()` | NUL-terminated `unibrowser-mod-api` version string |
| `ub_mod_info_json()` | NUL-terminated JSON: `name`, `version`, `unibrowser_api` |

Optional lifecycle hooks (`ub_mod_init`, `ub_mod_shutdown`) are reserved for a future ABI revision.

## Host vtable (`UbModHostV1`)

See [`ub-mod.h`](ub-mod.h). The host passes a context pointer (`host_ctx`) that backs an internal command registry. `register_command(ctx, id, title)` adds palette metadata only; the host may attach Rust handlers for well-known demo ids during the blitz (`mod.hello`).

## WIT sketch

Component-model direction for sandboxed mods: [`ub-mod.wit`](ub-mod.wit). The current loader uses the C vtable directly; WIT is not wired yet.

## Example crate

[`examples/ub-mod-cdylib-stub`](../../examples/ub-mod-cdylib-stub) builds `libub_mod_cdylib_stub.so` and registers `mod.hello`.

## Packaging checklist

1. Pin `unibrowser_api` in `info.json` to the host `MOD_API_VERSION`.
2. Ship `library` next to `info.json` (future: discover under user mod dir).
3. Bump `ub_mod_loader_abi_version` only when the host vtable changes.
