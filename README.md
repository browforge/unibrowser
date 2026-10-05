# Unibrowser

Open source **browser for files and the web**: one tab strip for folders, local files, and `https://` pages (Chrome OS–style). Downloads default to **open in a new tab**. Same commands and toolbars across formats (SOLIDWORKS-style modes, grey-out-not-hide), powered by Factorio-style mods and a VS Code–sized Rust core.

**Status:** Phase 1 MVP browser shell on **Tauri 2** (`unibrowser-desktop`). See [docs/vision.md](docs/vision.md) and [docs/roadmap.md](docs/roadmap.md).

## What works today (blitz snapshot)

| Area | Working now | Not yet |
| --- | --- | --- |
| **Browser** | Tabs, omnibox (`file://` + `https://`), back/forward/reload, home dir browse; **recent files** list (optional, persisted) | WebView2/GTK download hooks |
| **Editors** | **Lazy-loaded** Monaco text + save, **Ctrl+F find**; Markdown preview on `.md` | Full WYSIWYG markdown |
| **Viewers** | Lazy pdf.js / three.js; PDF; DOCX; images; DXF layers; **CSV/TSV table** (read-only stub); video/audio; Rhai; SVG; EPUB | Full spreadsheet edit; DXF NURBS |
| **Shell** | Omnibox autocomplete; **dirty tab •** + close confirm; **session restore** (opt-in); **auto-save** (opt-in, text/SVG); tab search; drag-drop; **status bar**; **bookmark bar** (home, downloads, project + pins); **Compare** (Monaco diff); **en-NZ** chrome strings via `t()`; split view; toasts; `./scripts/smoke-desktop-qa.sh` | Full i18n; Monaco spellcheck |
| **Downloads** | WebView hook emits URL → `handle_download_url` opens **new tab**; path traversal rejected ([`downloads.rs`](crates/unibrowser-desktop/src-tauri/src/downloads.rs) tests) | Save-to-disk disposition |
| **Security** | `..` rejected on `open_path`, reads, CSV, drops; `.unibrow` assets confined to `assets/` | Sandboxed mod execution |
| **Mods API** | Frozen **0.1** surface ([docs/mod-api-stable.md](docs/mod-api-stable.md)); `examples/ub-mod-cdylib-stub` | Runtime cdylib load |
| **Keys** | Single chords + sketch **Ctrl+K, Ctrl+S** save sequence ([design note](docs/design/key-chord-sequences.md)) | User-defined sequences in JSON |
| **Projects** | `.unibrow` folder or zip, export zip, asset CRUD, drag-drop | Cloud sync |
| **Commands** | Command Manager tabs (keyboard roving), floating bars, palette, keybindings | Full audio/image toolbars |
| **UI** | Light/dark/system theme persisted; optional onboarding tip | — |
| **Mods** | Bundled `base-*` mods; **Mod portal** with Open VSX search/install, VSIX enable/disable/uninstall, V1 commands + keybindings (user `keybindings.json` wins), palette shows **needs extension host** | VSIX JS execution, `.ubmod` store |
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

### Windows installer (MSI / NSIS)

After `npm run tauri build` on Windows, Tauri 2 emits bundles under:

`crates/unibrowser-desktop/src-tauri/target/release/bundle/`

Typical outputs (exact names depend on `tauri.conf.json`):

- **MSI** — `bundle/msi/*.msi` for enterprise-style installs (WebView2 bootstrapper may run on first launch if not already installed).
- **NSIS** — `bundle/nsis/*-setup.exe` when the NSIS target is enabled in Tauri bundle config.

Sign installers before wide distribution. WebView2 runtime must be present on the target machine ([Microsoft WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)).

## Linux packaging (AppImage / deb)

From `crates/unibrowser-desktop`, after `npm run tauri build`:

| Format | Typical path | Notes |
| --- | --- | --- |
| **AppImage** | `src-tauri/target/release/bundle/appimage/*.AppImage` | Single file, no root install; needs FUSE or `--appimage-extract` on some distros. |
| **deb** | `src-tauri/target/release/bundle/deb/*.deb` | `dpkg -i` installs menu entry + binary; pulls WebKitGTK deps via package metadata. |

Enable targets in `src-tauri/tauri.conf.json` under `bundle.targets` if a format is missing locally. GTK/WebKit dev packages from [Tauri prerequisites](https://tauri.app/start/prerequisites/) are required to build on Linux.

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
