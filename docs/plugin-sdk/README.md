# Unibrowser Plugin SDK (draft)

Plugins are not limited to new file extensions: they may register commands, UI into fixed slots (Command Manager, panels, status bar), content-kind tools, providers, and services ([ADR 0028](../adr/0028-platform-content-kinds-and-plugin-scope.md), [positioning.md](../positioning.md)).

Compiled mods ship as native libraries (`cdylib` on Linux/macOS, `dll` on Windows) that export a small C ABI. With the `dynamic-mods` feature, the host loads **enabled** mods from the user mods directory (see below). Linux is the first platform where cdylibs are loaded at runtime. Rust mods can use the same symbols as the reference stub; other languages link against [`ub-mod.h`](ub-mod.h).

Signing policy (stub): [signing.md](signing.md).

This draft sits beside the stable Rust surface in [`unibrowser-mod-api`](../mod-api-stable.md). Manifest and versioning rules: [ADR 0006](../adr/0006-mod-manifest-and-versioning.md).

## User mods directory

| OS | Path |
| --- | --- |
| Linux | `~/.config/unibrowser/mods/<mod-name>/` (or `$XDG_CONFIG_HOME/unibrowser/mods/`) |
| macOS | `~/Library/Application Support/unibrowser/mods/` |
| Windows | `%APPDATA%\\unibrowser\\mods\\` |

Each mod is a folder containing:

- `info.json` — `ModManifest` (`name`, `version`, `title`, `unibrowser_api`, optional `library`)
- `state.json` — host-managed `{ "enabled": true }` (written when you toggle in the mod portal)
- Native library — see **Library file names** below

Enable or disable mods in **Mod portal** (native and VSIX). Only enabled native mods with a resolvable library are loaded when `dynamic-mods` is on.

## Library file names

If `info.json` omits `library`, the host looks for (examples for mod name `my-mod` → stem `my_mod`):

| OS | Candidate files (in mod folder) |
| --- | --- |
| Linux | `libmy_mod.so`, `libmy-mod.so` (stem uses `-` → `_`) |
| macOS | `libmy_mod.dylib`, `libmy-mod.dylib` |
| Windows | `my_mod.dll`, `my_mod_mod.dll` |

Prefer setting `"library": "libmy_plugin.so"` explicitly for cross-platform packs.

## Quick start (Linux)

**User install (recommended):**

```bash
cargo build -p ub-mod-cdylib-stub
MOD_ROOT="${XDG_CONFIG_HOME:-$HOME/.config}/unibrowser/mods/hello-stub"
mkdir -p "$MOD_ROOT"
cp examples/ub-mod-cdylib-stub/info.json.example "$MOD_ROOT/info.json" 2>/dev/null || true
# Or write info.json with name, version, title, unibrowser_api, library: libub_mod_cdylib_stub.so
cp target/debug/libub_mod_cdylib_stub.so "$MOD_ROOT/"
# Enable in Mod portal or echo '{"enabled":true}' > "$MOD_ROOT/state.json"
```

**Dev stub (optional):**

```bash
export UNIBROWSER_LOAD_CDYLIB_MOD=1
export UNIBROWSER_CDYLIB_MOD_PATH=examples/ub-mod-cdylib-stub/target/debug/libub_mod_cdylib_stub.so
```

Command palette should list **Hello from mod** (`mod.hello`) when loaded.

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
