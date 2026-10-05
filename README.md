# Unibrowser

Open source **browser for files and the web**: one tab strip for folders, local files, and `https://` pages (Chrome OS–style). Downloads default to **open in a new tab**. Same commands and toolbars across formats (SOLIDWORKS-style modes, grey-out-not-hide), powered by Factorio-style mods and a VS Code–sized Rust core.

**Status:** Phase 1 MVP browser shell on **Tauri 2** (`unibrowser-desktop`). See [docs/vision.md](docs/vision.md) and [docs/roadmap.md](docs/roadmap.md).

## What works today (blitz snapshot)

| Area | Working now | Not yet |
| --- | --- | --- |
| **Browser** | Tabs, omnibox (`file://` + `https://`), back/forward/reload, home dir browse | WebView2/GTK download hooks |
| **Editors** | Monaco text + save; **Markdown preview** toggle on `.md` | Full WYSIWYG markdown |
| **Viewers** | PDF (page + thumbs), DOCX HTML, DXF (layers, pan/zoom), video/audio, Rhai scripts, **SVG source + preview + save** | PDF raster thumbs; DXF NURBS |
| **Projects** | `.unibrow` create/open, add/rename/remove assets (rename renames file on disk), drag-drop | Cloud sync |
| **Commands** | Command Manager tabs, floating bars (drag commands), palette, keybindings file | Full audio/image toolbars |
| **Mods** | Bundled `base-*` mods; **Mod portal** with Open VSX search (opt-in network), **VSIX install** to config dir, V1 command metadata in palette (no extension host) | VSIX JS execution, `.ubmod` store |
| **Media** | ffprobe; optional **LGPL sidecar** pool (`UNIBROWSER_MEDIA_USE_SIDECAR=1`) | Bundled ffmpeg in installer |
| **Scripting** | Rhai sandbox, per-script allowlist, browse-only palette without doc | Script panel UX polish |

## Licence

Apache-2.0. See [LICENSE](LICENSE) and [docs/adr/0007-licensing.md](docs/adr/0007-licensing.md).

## Requirements

- Rust 1.85+ (`rust-toolchain.toml`)
- **Linux (Tauri):** `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev` ([Tauri prerequisites](https://tauri.app/start/prerequisites/))
- **Node.js 20+** for the desktop UI build
- **Windows:** MSVC, WebView2 runtime, Node.js; build on a Windows host (see below)

## Run the product (Tauri)

```bash
cd crates/unibrowser-desktop
npm install
npm run tauri dev
```

- Browse your home directory on first launch; open folders and files in tabs.
- Enter `https://example.com` in the omnibox.
- **Command Manager** tabs (Text, Office, Code, Image, Audio): drag to reorder commands; layouts in `toolbars.json` ([docs/ui-commands.md](docs/ui-commands.md)).
- **Ctrl+Shift+P** ( **Cmd+Shift+P** on macOS) opens the command palette; override chords in `keybindings.json` under your config dir (see [docs/keybindings.md](docs/keybindings.md)).
- Open a `.txt`, `.md`, or other text file for **Monaco** editing; **Ctrl+S** saves to disk.
- Select text in the editor for **Increase / Decrease font size** (greyed for `.png` without text layer).
- **Demo download** exercises open-in-new-tab disposition.

Release build:

```bash
cd crates/unibrowser-desktop
npm run tauri build
# Binary: ../../target/release/unibrowser-desktop (workspace target)
```

## egui spike (optional)

```bash
cargo run -p unibrowser-app
```

## Tests

```bash
cargo test --workspace
```

## Windows

### Native build and run (recommended)

```powershell
cd crates\unibrowser-desktop
npm install
npm run tauri dev
```

Release: `npm run tauri build`

### Cross-check from Linux (CI)

Core crates (excluding the Tauri bundle step that needs `llvm-rc` / Windows resources):

```bash
rustup target add x86_64-pc-windows-msvc
cargo check --workspace --exclude unibrowser-desktop --target x86_64-pc-windows-msvc
```

The `unibrowser-desktop` package is built and tested on **Linux** in CI; produce a Windows installer on a Windows machine.

## Repository layout

| Path | Purpose |
| --- | --- |
| `crates/unibrowser-desktop` | **Tauri 2 browser shell** (product) |
| `crates/unibrowser-mod-api` | Versioned mod API + `BrowserLocation` |
| `crates/unibrowser-engine` | Commands, tabs, modes, palette |
| `crates/unibrowser-mod-host` | `info.json` loader |
| `crates/unibrowser-app` | egui demo / harness |
| `mods/*` | Bundled mods (`base-files`, `base-web`, `base-text`, …) |
| `docs/` | Vision, architecture, ADRs, [downloads](docs/downloads.md) |
