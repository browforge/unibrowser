# Changelog

All notable changes during the **36h blitz** on `main` are listed here. Format loosely follows [Keep a Changelog](https://keepachangelog.com/).

## [0.1.0] — blitz (in progress)

### Added (latest)

- Unified **Increase font size** demo: `unibrowser.font.size.increase` on Monaco/text, SVG preview, DOCX/EPUB HTML, HTTP user-style; matrix in [`docs/commands-cross-format.md`](docs/commands-cross-format.md).
- **Places**: recent files + pinned folders in Files manager; palette entries (`unibrowser.places.open`) with `open_path`.
- Theme persistence (light / dark / system) applies to Command Manager chrome and floating toolbars; high-contrast borders on CM + bars.
- Hello-world **`.ubmod`** sample (`examples/hello-world.ubmod`, `scripts/build-hello-ubmod.sh`); [`docs/plugin-sdk.md`](docs/plugin-sdk.md).
- Onboarding: skippable first-run tour (`onboarding_tour_skipped_forever`), non-blocking card; [`docs/onboarding.md`](docs/onboarding.md).
- DXF **proving mod** (`base-dxf`): geometry summary, wireframe/SVG proxy, greyed CAD commands; [`docs/proving-mod-dxf.md`](docs/proving-mod-dxf.md).
- Palette command **Run demo Rhai script** → script output panel + optional font-size bump on text tabs.
- i18n seed: `en-NZ` + `en-US` catalogues; **Language** in Settings (`ui_locale`).
- Floating toolbars: screen-coordinate persistence in `toolbars.json` (multi-monitor friendly); [`docs/toolbars.md`](docs/toolbars.md) + ADR 0013 cross-link.
- Mod portal context panel when a file has no handler (Open VSX + `.ubmod` stub suggestions; non-blocking beside the tab).
- `keybindings.json` palette override end-to-end (`reload_keybindings` on focus); default **`Ctrl+Shift+P`** documented in README and [`docs/keybindings.md`](docs/keybindings.md).
- `.unibrow` project recovery: debounced `project.json` backups under config; launch restore prompt.
- Accessibility: Command Manager tab keyboard map, palette Escape, focus rings, toolbar `aria-label`s ([`docs/ux-a11y.md`](docs/ux-a11y.md)).
- PDF proving banner (document / raster / vector) + `ensure_pdf_raster_proxy`; timeline MIDI scrub status bar + encode note.
- VSIX `activationEvents` metadata; stub execute path; **Extensions** Command Manager tab ([`docs/extension-host.md`](docs/extension-host.md)).
- Timeline **Export MP4 (H.264)** (`fast` / `balanced`) via ffmpeg with `unibrowser-encode-progress` events; sidecar `EncodeH264` op.
- [`docs/browser-shell.md`](docs/browser-shell.md); Settings toggle **Open web downloads in system browser** (WebView2 interim).
- Living Wed report [`docs/blitz-status.md`](docs/blitz-status.md); printer-friendly HTML export for Markdown/DOCX previews.
- OCR UI stub + [`docs/ocr-sidecar.md`](docs/ocr-sidecar.md); experimental browser spellcheck in Advanced settings.
- Docs hub [`docs/index.md`](docs/index.md), FOSS [`docs/getting-started.md`](docs/getting-started.md), and [`docs/report-visuals.md`](docs/report-visuals.md) + `scripts/capture-report-screenshots.sh`.
- EPUB in-book search (spine text scan + chapter jump in viewer).
- File-type badges in the files panel; compact Command Manager option in Settings.
- High-contrast mode and reduce-motion preference (system / on / off).
- Non-blocking onboarding checklist in Settings with hide forever.
- Security fuzz tests for `path_security` and `transfer_paths`; transfer name validation and dest escape guard.
- ADR 0019 WASM T2 sandbox notes; `unibrowser-mod-host/wasm-mods` wasmtime feature stub.
- Large directory listing perf test (`UB_LIST_STRESS=1` for 10k); virtualized files table when >500 entries.
- Tab close purges undo stacks; `window_session_cleanup` regression test.

### Added (previous)

- User mods dir (`~/.config/unibrowser/mods` on Linux): discover `info.json`, load enabled cdylibs when `dynamic-mods` is on (not env-only).
- Mod portal: enable/disable **native mods** and **VSIX** in separate lists.
- T1 signing stub: [docs/plugin-sdk/signing.md](docs/plugin-sdk/signing.md); `UNIBROWSER_REQUIRE_SIGNED_MODS=1` refuses unsigned loads.
- Plugin SDK: Windows/macOS library naming and user mods paths in README.

### Added

- **Plugin SDK draft**: `docs/plugin-sdk/` (README, `ub-mod.h`, `ub-mod.wit`); example `ub-mod-cdylib-stub` registers `mod.hello`.
- **Dynamic mod load (Linux)**: feature `dynamic-mods` on `unibrowser-mod-host` / desktop; set `UNIBROWSER_LOAD_CDYLIB_MOD=1` and optional `UNIBROWSER_CDYLIB_MOD_PATH`.
- **CLI versioning**: `unibrowser` binary (`cargo run -p unibrowser-cli -- --version`) via `unibrowser-build-info` (embeds short git hash when available).
- **About page**: shows version and git hash through `get_build_info`.
- Desktop browser shell (Tauri 2): tabs, omnibox, multi-window, downloads, split view, command manager, palette, keybindings.
- Files panel: breadcrumbs, favourites, multi-select, drag-drop, zip peek, magic-byte sniffing, filename search.
- Viewers: Monaco text, PDF, DOCX, SVG, CSV, EPUB, image/audio/video, MIDI scrub, `.unibrow` board and timeline.
- Timeline: ruler, scrub thumbnails, MP4 preview export, fade transition stub, audio peaks.
- Projects: templates (blank / notes / video), `project.json` autosave, share zip import/export.
- Mod portal, Open VSX install, Rhai scripting, VSIX keybinding contributions.
- `cargo test --workspace` coverage including timeline fuzz and zip roundtrips.

### Changed

- README “what works” table tracks blitz capabilities.

### Fixed

- Removed unused import in `unibrowser-media` (`frame.rs`) that failed `cargo test` warnings-as-errors in strict CI.
