# Unibrowser

Open source **browser for files and the web**: one tab strip for folders, local files, and `https://` pages (Chrome OS–style). Downloads default to **open in a new tab**. Same commands and toolbars across formats (SOLIDWORKS-style modes, grey-out-not-hide), powered by Factorio-style mods and a VS Code–sized Rust core.

**Status:** Phase 1 MVP browser shell on **Tauri 2** (`unibrowser-desktop`). See [docs/vision.md](docs/vision.md) and [docs/roadmap.md](docs/roadmap.md).

## What works today (Wed report draft)

Use this section as the source for weekly status. Narrative + gaps: [docs/blitz-status.md](docs/blitz-status.md). Last updated during the 36h blitz on `main`.

### Browser and windows

| Capability | Detail |
| --- | --- |
| Tabs & omnibox | `file://`, paths, `https://`, internal `unibrowser://` pages; back/forward/reload; dirty tab marker and close confirm |
| Multi-window | **Per-window** tabs, split view, undo stacks, compare state, chrome notices; shared prefs on disk ([docs/multi-window.md](docs/multi-window.md)) |
| Home / NTP | Customisable sections (hero, actions, browse folder, recent); plain-text empty states |
| Recent & session | Recent files list; optional session restore on launch |
| Downloads | WebView / GTK hook → `handle_download_url`; default open in new tab; path traversal rejected |
| Web content | Per-window embedded webview; optional **devtools** (Advanced settings); per-origin font zoom |

### Files and formats

| Capability | Detail |
| --- | --- |
| Folder browse | **Breadcrumbs**, parent up, **folder favourites** (star), sort (name/date/size), hidden files, **in-folder name filter** (debounced), **lazy image thumbnails** (capped), **multi-select** (open, copy paths, delete) |
| File ops | **Copy/move** (side-panel picker), **drag-drop** onto folder tabs / current folder / **`.unibrow` board**, **compress to zip**, **properties**, create/rename/duplicate/delete; internal **Ctrl+X/C/V**; **Ctrl+Shift+F** filename search under a root (depth cap); listing **timeout/cancel** on slow paths; **recent** pin/unpin and clear unpinned |
| Archives | **Peek** `.zip` / `.unibrow` zip; click a file entry to **extract to cache and open** when supported |
| Magic-byte sniff | Wrong extension still opens correct viewer: PDF, PNG, JPEG, GIF, WebP, ZIP/EPUB, WAV, MP3, OGG, FLAC, RIFF/AVI → video, etc. |
| Text & code | Monaco editor, save, Ctrl+F, undo/redo stacks; lazy-loaded bundles |
| Documents | PDF (pdf.js), DOCX, SVG, CSV/TSV table stub, EPUB chapters + in-book search |
| Media | Video with scrub frames; **audio peak waveform** (decode or stub bars); optional LGPL sidecar pool |
| 3D / CAD | `.unibrow` **templates** (blank / notes board / video mix), board colours, timeline markers, scrub + MIDI beeps, **project.json autosave** backup (settings), **share zip** export/import (no `derived/`), full zip export, MP4 preview |
| Images | PNG/JPEG/GIF/WebP viewer; **Files bulk convert** to PNG or JPEG |

### Commands, keys, and UI chrome

| Capability | Detail |
| --- | --- |
| Command Manager | Context tabs, floating toolbars, import/export JSON layouts, workspace layout presets |
| Palette | Fuzzy search (prefix/word/consecutive bonuses); **key chord** beside each command when bound |
| Shortcuts | `keybindings.json` + VSIX contributions; `unibrowser://keys` cheatsheet page |
| Chrome | Bookmark bar, split view, tab search, drag-drop open, status bar, toasts, en-NZ `t()` strings |
| Compare | Side-by-side text diff (Monaco) for two files |
| Settings | Theme, downloads, auto-save, board snap, crash recovery backups, NTP sections, advanced webview tools |
| Themes | Light / dark / system persisted |

### Mods, scripting, security

| Capability | Detail |
| --- | --- |
| Bundled mods | `base-*` format handlers; mod portal with Open VSX search/install, VSIX enable/disable |
| Rhai | Sandboxed scripts, per-script command allowlist |
| API | Frozen mod API 0.1 ([docs/mod-api-stable.md](docs/mod-api-stable.md)) |
| Security | `path_security` on open/read/drops; `.unibrow` assets confined under `assets/` |

### Quality and tooling

| Capability | Detail |
| --- | --- |
| Tests | `cargo test --workspace` (incl. **timeline CRUD fuzz** + zip roundtrip on `.unibrow`) |
| Desktop build | `npm run build` in `crates/unibrowser-desktop` |
| Smoke | `./scripts/smoke-desktop-qa.sh` and [docs/qa-desktop.md](docs/qa-desktop.md) |

### Not yet / gaps

- Save-to-disk download disposition UI polish; full VSIX JS extension host
- Cloud sync; full spreadsheet/DXF NURBS
- Windows/macOS cdylib runtime load (Linux + user `mods/` today; see [docs/plugin-sdk](docs/plugin-sdk/README.md))
- Complete i18n beyond en-NZ; Monaco spellcheck
- Windows installer signing and bundled ffmpeg in installer
- Real timeline encode (render fades, audio mix, PDF page-per-time); batch convert beyond PNG/JPEG
- Live multi-user collaboration (share zip is export/import only); cloud project hosting

## Licence

Apache-2.0. See [LICENSE](LICENSE), [NOTICE.md](NOTICE.md), [docs/licenses/](docs/licenses/) (Monaco, pdf.js, Mammoth, ffmpeg, WebView, and other third-party notices), and [docs/adr/0007-licensing.md](docs/adr/0007-licensing.md).

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

## CLI version

```bash
cargo run -p unibrowser-cli -- --version
```

## Tests

```bash
cargo test --workspace
cargo build -p ub-mod-cdylib-stub   # optional: Linux dynamic mod smoke
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

See [docs/index.md](docs/index.md) (ADRs, getting started) and [docs/architecture.md](docs/architecture.md). Contributing: [CONTRIBUTING.md](CONTRIBUTING.md).
