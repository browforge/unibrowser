# Unibrowser

Open source **browser for files and the web**: one tab strip for folders, local files, and `https://` pages (Chrome OS–style). Downloads open in a new tab by default; the same commands and toolbars work across formats (SOLIDWORKS-style modes, grey-out-not-hide), powered by Factorio-style mods and a VS Code–sized Rust core.

**Status:** Phase 0 skeleton (Rust engine + egui demo). See [docs/vision.md](docs/vision.md) and [docs/architecture.md](docs/architecture.md).

## Licence

Apache-2.0. See [LICENSE](LICENSE) and [docs/adr/0007-licensing.md](docs/adr/0007-licensing.md).

## Requirements

- Rust 1.75+ (stable)
- **Linux:** GTK3 / X11 or Wayland dev packages for `eframe` (see CI workflow). At runtime you may also need `libxkbcommon-x11-0` on X11 (Ubuntu: `sudo apt install libxkbcommon-x11-0`).
- **Windows:** MSVC toolchain for native builds; cross-check uses `cargo-xwin`

## Build and run (Linux)

```bash
cargo build -p unibrowser-app
cargo run -p unibrowser-app
```

- **S** opens the command palette.
- Switch tabs between sample text, PNG, and Markdown documents.
- **Increase font size** is the same toolbar command; PNG without a text layer stays **greyed**, not hidden.

## Tests

```bash
cargo test --workspace
```

## Windows

### Native (recommended on Windows)

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build -p unibrowser-app --release
cargo run -p unibrowser-app
```

### Cross-check from Linux (CI)

```bash
rustup target add x86_64-pc-windows-msvc
cargo check --workspace --target x86_64-pc-windows-msvc
```

This validates that the Rust workspace type-checks for Windows without linking a GUI binary on Linux. For a full Windows GUI build, use MSVC on a Windows host.

Optional: `cargo-xwin` can link Windows binaries from Linux if the MSVC CRT download succeeds (`cargo install cargo-xwin --locked`).

## Repository layout

| Path | Purpose |
| --- | --- |
| `crates/unibrowser-mod-api` | Versioned mod API |
| `crates/unibrowser-engine` | Commands, modes, toolbar, palette |
| `crates/unibrowser-mod-host` | `info.json` loader |
| `crates/unibrowser-app` | Desktop demo |
| `mods/*` | Bundled base mods (`info.json` + Rust) |
| `docs/` | Vision, architecture, ADRs, roadmap |

## Roadmap

[docs/roadmap.md](docs/roadmap.md) — Phase 1 moves document surfaces to **Tauri 2 + WebView** per [ADR 0001](docs/adr/0001-shell-and-rendering.md).
