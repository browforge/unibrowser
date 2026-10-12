# Getting started (FOSS contributors)

Unibrowser is an Apache-2.0 browser shell for local files and the web. This guide gets you from clone to a green test run and a sensible first PR.

## Prerequisites

- Rust 1.85+ (`rust-toolchain.toml`)
- Node.js 20+
- **Linux (desktop):** GTK 3 and WebKitGTK dev packages ([Tauri prerequisites](https://tauri.app/start/prerequisites/)). On Ubuntu or Debian, run `sudo ./scripts/setup-linux-deps.sh` once per machine (build tools, WebKitGTK, xvfb, ImageMagick, AppImage helpers, and the `wasm32-unknown-unknown` Rust target).

Windows desktop builds are optional for engine-only changes; CI checks `x86_64-pc-windows-msvc` for non-Tauri crates.

## Clone and build

```bash
git clone <your-fork-url>
cd unibrowser
cargo test --workspace
cd crates/unibrowser-desktop
npm ci
npm run build
npm run tauri dev
```

The app opens on your home directory by default. Use the omnibox for `unibrowser://settings`, `https://example.com`, or a file path.

To inspect an embedded https tab, press **F12** or run **Toggle developer tools** from the command palette. See [devtools.md](devtools.md).

## Where code lives

| Area | Path |
| --- | --- |
| Desktop shell (Tauri + UI) | `crates/unibrowser-desktop/` |
| Engine (tabs, commands, palette) | `crates/unibrowser-engine/` |
| Mod host and bundled mods | `crates/unibrowser-mod-host/`, `mods/` |
| Projects (`.unibrow`) | `crates/unibrowser-project/` |
| Design docs and ADRs | `docs/` ([index](index.md)) |

Prefer extending an existing mod or shell viewer before adding parallel pipelines.

## Before you open a PR

```bash
./scripts/smoke-desktop-qa.sh
```

That runs `cargo test --workspace`, `npm run build`, and prints the manual checklist from [qa-desktop.md](qa-desktop.md).

## Good first tasks

- Fix a bug with a unit test next to the logic (`unibrowser-project`, `path_security`, viewers in `src-tauri`).
- Improve en-NZ strings in `crates/unibrowser-desktop/src/i18n/en-NZ.json` (use `t()` in UI).
- Document behaviour in `docs/` when you change user-visible flows.
- Native mod experiments: [plugin-sdk/README.md](plugin-sdk/README.md) and `examples/ub-mod-cdylib-stub`.

## Out of scope for drive-by PRs

- Full VSIX JavaScript extension host ([ADR 0018](adr/0018-extension-host-stub.md)) — stub remains intentional.
- Cloud sync, installer signing, and production timeline encode (see README gaps).

## Licence and conduct

Apache-2.0 ([LICENSE](../LICENSE), [ADR 0007](adr/0007-licensing.md)). Be respectful in issues and reviews; see [CONTRIBUTING.md](../CONTRIBUTING.md).
