# Plugin SDK (hello-world `.ubmod`)

Native mods are **folders** (or a **`.ubmod` zip**) with `info.json`, optional `state.json`, and a platform library. Full ABI reference: [`plugin-sdk/README.md`](plugin-sdk/README.md).

## Hello-world package

The workspace ships:

| Artefact | Purpose |
| --- | --- |
| `examples/ub-mod-cdylib-stub/` | Rust `cdylib` source; registers `mod.hello` |
| `examples/hello-world-ubmod/info.json` | Manifest for the sample pack |
| `examples/hello-world.ubmod` | Zip bundle (build locally; not in git — Linux `.so` inside; rebuild on other OS) |
| `scripts/build-hello-ubmod.sh` | Build stub + zip |

### Build (Linux)

```bash
./scripts/build-hello-ubmod.sh
```

This runs `cargo build -p ub-mod-cdylib-stub` and writes `examples/hello-world.ubmod`.

### Install

**From zip (sideload):** Mod portal → Install from file… → pick `examples/hello-world.ubmod` (unpacks to user mods dir when wired; today copy manually):

```bash
MOD_ROOT="${XDG_CONFIG_HOME:-$HOME/.config}/unibrowser/mods/hello-world"
mkdir -p "$MOD_ROOT"
unzip -o examples/hello-world.ubmod -d "$MOD_ROOT"
echo '{"enabled":true}' > "$MOD_ROOT/state.json"
```

**Dev load without install:**

```bash
export UNIBROWSER_LOAD_CDYLIB_MOD=1
export UNIBROWSER_CDYLIB_MOD_PATH=target/debug/libub_mod_cdylib_stub.so
```

Run the desktop app with `dynamic-mods` enabled (default on Linux dev builds). Palette should list **Hello from mod** (`mod.hello`).

### Authoring your mod

1. Copy `examples/ub-mod-cdylib-stub` or link [`ub-mod.h`](plugin-sdk/ub-mod.h).
2. Export `ub_mod_loader_abi_version`, `ub_mod_register`, `ub_mod_api_version`, `ub_mod_info_json`.
3. In `ub_mod_register`, call `register_command(ctx, id, title)`.
4. Set `"unibrowser_api": "0.1.0"` in `info.json` to match the host.
5. Optional T1 signing: [plugin-sdk/signing.md](plugin-sdk/signing.md).

### Verify

```bash
cargo test -p ub-mod-cdylib-stub
cargo test --workspace
```

Command palette → **Hello from mod** → toast with handler result.
