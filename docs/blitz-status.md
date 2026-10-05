# 36h blitz status (living Wed report)

**Branch:** `main` · **Version:** 0.1.0 blitz · **Last updated:** during active blitz (edit this file each Wed).

Use this page for weekly status posts. Capability tables in [README.md](../README.md) stay detailed; this doc is the narrative plus gaps and runbook.

## Shipped (highlights)

| Area | What works |
| --- | --- |
| Shell | Tauri 2 desktop: tabs, omnibox (`file://`, paths, `https://`, `unibrowser://`), multi-window, **split view** (experimental vertical split, [split-view.md](split-view.md)), **session restore** ([session-restore.md](session-restore.md)), downloads (default open in tab) |
| Downloads UI | `unibrowser://downloads` manager: history, retry failed, clear list |
| Security | Trust banners, remote-web chrome hint, click-to-play media ([security.md](security.md)) |
| Perf | Lazy Monaco/pdf.js/mammoth; [perf.md](perf.md) + `scripts/measure-cold-start.sh` |
| Find / zoom | Ctrl+F in-document overlay (Monaco, DOCX, EPUB); page zoom for web/PDF/image ([commands-cross-format.md](commands-cross-format.md)) |
| Status bar | Path, Command Manager context (pinned/auto), unsaved dot, encoding, cursor, font pt, zoom, encode % ([commands-undo-redo.md](commands-undo-redo.md)) |
| Undo / PDF | Document undo/redo on text + SVG; export PDF from Markdown/DOCX print HTML (headless Chromium / wkhtmltopdf on Linux); matrix in [commands-undo-redo.md](commands-undo-redo.md) |
| Packaging | [packaging.md](packaging.md) stubs for Windows MSI/NSIS and Linux AppImage/deb |
| Chrome | Tab drag reorder, middle-click close, pin (Ctrl+click); omnibox Places/recent/URL suggestions; [chrome.md](chrome.md) |
| Audio proving | `wav`/`mp3`/`flac`/`ogg` viewer: HTML audio + waveform (Web Audio or ffmpeg PCM); grey DAW commands on `base-audio` |
| CM pin UX | Separate **Pin commands tab** control; tab clicks are preview-only until pin; tip in Settings + first pin toast |
| Clipboard / timeline | Copy image from PNG viewer; **Paste still** on `.unibrow` timeline (asset + clip stub) |
| Mod catalog | Offline filter in mod portal (`list_mod_catalog`); Open VSX network search optional; sideload always |
| Licensing | [NOTICE.md](../NOTICE.md) + [docs/licenses/](licenses/) third-party attributions; `./scripts/generate-notice.sh` for Rust deps |
| Error UX | Chrome toasts with **Copy details** on failures; friendly copy for opens/encodes (no raw panic strings) |
| EPUB | Chapter dropdown + prev/next; last position per book in prefs; font size via shared increase/decrease commands |
| Grandma backup | Settings export/import JSON: preferences + `keybindings.json` + `toolbars.json` |
| About | `unibrowser://about`: version, git hash, mod/command counts, licence + NOTICE pointers |
| DevTools | **F12** / palette **Toggle developer tools** on https tabs; [devtools.md](devtools.md) |
| Image edit | PNG/JPEG: rotate, flip, save-as (new file); crop export; grey **Filters** stub |
| `.unibrow` v2 | `format_version` 2 + validation on open; v1 still loads; [unibrow-format-migration.md](unibrow-format-migration.md) |
| CI | [`scripts/check-workspace.sh`](../scripts/check-workspace.sh), GitHub Actions + [ci.md](ci.md) (Origin contract) |
| Files | Breadcrumbs, favourites, sort/filter, multi-select, drag-drop, zip peek, magic-byte sniff, virtualised large listings |
| Commands | Command Manager (dense mode), palette + chords (`Ctrl+Shift+P` default, user override via `keybindings.json`), workspace layout presets, import/export JSON; VSIX commands on **Extensions** tab (stub run) |
| Viewers | Monaco text, PDF (multi-engine proving banner + lazy raster proxy), DOCX, SVG, CSV, EPUB search, images, video/audio, MIDI stub, `.unibrow` board/timeline + MIDI scrub status |
| Projects | Templates, `project.json` autosave backup, share zip import/export; timeline **H.264 export** (ffmpeg, fast/balanced) with progress toasts |
| Mods | Bundled `base-*`, mod portal (context install panel + Open VSX / `.ubmod` stubs), Rhai scripts, Linux user `mods/` + dynamic cdylib (feature flag); VSIX `activationEvents` stored |
| Toolbars | Floating bars drag-anywhere; positions in `toolbars.json` as screen coords ([toolbars.md](toolbars.md)); theme + high-contrast on CM and floating bars |
| Cross-format | `unibrowser.font.size.increase` live on text, SVG, DOCX, EPUB, HTTP user-style ([commands-cross-format.md](commands-cross-format.md)) |
| Places | Files manager **Places** (recent + pinned); palette fuzzy open for paths ([commands-cross-format.md](commands-cross-format.md) palette) |
| Plugin sample | `examples/hello-world.ubmod` + [plugin-sdk.md](plugin-sdk.md) (`mod.hello` cdylib) |
| Recovery | Text/SVG crash backups (opt-in); `.unibrow` `project.json` debounced backups + launch prompt when project autosave or crash recovery is on |
| Downloads | Linux WebKit `on_download` → shell; optional **system browser** hand-off ([browser-shell.md](browser-shell.md)) |
| A11y prefs | High contrast, reduce motion, skippable first-run tour + Settings checklist ([onboarding.md](onboarding.md), [ux-a11y.md](ux-a11y.md)); CM tab keys, palette Escape, focus rings |
| i18n | Chrome strings `en-NZ` / `en-US`; Language toggle in Settings |
| Scripting | Rhai demo from palette → output panel; `.rhai` / `.js` script viewer |
| DXF | Proving mod wireframe + disabled CAD tools ([proving-mod-dxf.md](proving-mod-dxf.md)) |
| Docs | [docs/index.md](index.md), [getting-started.md](getting-started.md), [report-visuals.md](report-visuals.md), ADRs 0001–0019 |
| Quality | `cargo test --workspace`, path/transfer fuzz tests, `./scripts/smoke-desktop-qa.sh` |

## How to run

```bash
cargo test --workspace
cd crates/unibrowser-desktop && npm ci && npm run build
npm run tauri dev
```

CLI version: `cargo run -p unibrowser-cli -- --version`

Report screenshots: [report-visuals.md](report-visuals.md) and `./scripts/capture-report-screenshots.sh`.

## Gaps (honest)

| Item | Notes |
| --- | --- |
| VSIX JS extension host | Metadata + stub execute ([extension-host.md](extension-host.md)); no `main` JS |
| OCR | UI placeholder; Tesseract sidecar not built ([ocr-sidecar.md](ocr-sidecar.md)) |
| Spellcheck | Monaco has no dictionary; optional **experimental** browser spellcheck mirror in Advanced settings |
| Timeline encode | H.264 path when ffmpeg present; no audio mix / cross-fades; preview concat still copy-only |
| Windows/macOS dynamic mods | Linux user `mods/` today |
| i18n | Viewer copy not fully translated; only chrome catalogues today |
| Cloud / collab | Share zip only; no live multi-user |

## Known bugs / rough edges

- EPUB preview: spine HTML only; embedded images/fonts often missing until asset URL resolution lands.
- DOCX: layout fidelity limited (Mammoth); tracked changes not editable.
- Large PDFs: use profiling env `UNIBROWSER_PROFILE_OPEN=1` if open feels slow.
- WebView downloads: Linux wired; **WebView2 `DownloadStarting` not wired** — use Advanced → system browser ([browser-shell.md](browser-shell.md), [downloads.md](downloads.md)).
- Spellcheck experiment does not underline inside Monaco; mirror textarea only.
- OCR button stays disabled until sidecar ships.

## Demo script (~10 min)

1. **Home + chrome** — Open a folder tab; drag a tab to reorder; Ctrl+click to pin; omnibox: pick a **Place** or recent path, Enter to open. ([chrome.md](chrome.md))
2. **Command Manager pin** — Open a `.md` file; pin the Command Manager context (pin control); switch to another doc and show commands stay on the pinned tab; unpin to auto-follow.
3. **Cross-format font** — Same **Increase font size** on Markdown preview, DOCX, and an HTTP tab; mention status bar font/zoom. ([commands-cross-format.md](commands-cross-format.md))
4. **`.unibrow` H.264** — New project (Notes template); add a timeline clip; **Export MP4 (H.264)**; watch status bar encode % (ffmpeg required).
5. **Mod portal** — Open a file with no handler; mod portal panel suggests Open VSX / `.ubmod` without blocking the tab.
6. **Onboarding skip** — `unibrowser://settings` → dismiss checklist or skip tour forever; session restore still optional.
7. **Optional** — Open an MP3: waveform + play; show grey DAW toolbar; palette lists disabled DAW commands with reasons.

## Links

- [CHANGELOG.md](../CHANGELOG.md)
- [qa-desktop.md](qa-desktop.md)
- [CONTRIBUTING.md](../CONTRIBUTING.md)
