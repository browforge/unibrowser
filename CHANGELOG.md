# Changelog

All notable changes during the **36h blitz** on `main` are listed here. Format loosely follows [Keep a Changelog](https://keepachangelog.com/).

## [0.1.0] — blitz (in progress)

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
