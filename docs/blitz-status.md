# 36h blitz status (living Wed report)

**Branch:** `main` · **Version:** 0.1.0 blitz · **Last updated:** during active blitz (edit this file each Wed) · **Last blitz commit:** see `git log -1` on `main`

Use this page for weekly status posts. Capability tables in [README.md](../README.md) stay detailed; this doc is the narrative plus gaps and runbook.

## Shipped (highlights)

| Area | What works |
| --- | --- |
| Shell | Tauri 2 desktop: tabs, omnibox (`file://`, paths, `https://`, `unibrowser://`), multi-window, **split view** (experimental vertical split, [split-view.md](split-view.md)), **session restore** ([session-restore.md](session-restore.md)), downloads (default open in tab) |
| Downloads UI | `unibrowser://downloads` manager: history, retry failed, clear list |
| Security | Trust banners, remote-web chrome hint, click-to-play media; omnibox `file://` paths percent-decoded and traversal-checked ([security.md](security.md)) |
| Perf | Lazy Monaco/pdf.js/mammoth; [perf.md](perf.md) + `scripts/measure-cold-start.sh` |
| Find / zoom | Ctrl+F in-document overlay (Monaco, DOCX, EPUB); page zoom for web/PDF/image ([commands-cross-format.md](commands-cross-format.md)) |
| Status bar | Path, Command Manager context (pinned/auto), unsaved dot, encoding, cursor, **word count / reading time** (text + DOCX), font pt (click reset), zoom (click reset on web/PDF/image), encode % ([commands-undo-redo.md](commands-undo-redo.md)) |
| VSIX / extensions | Opt-in **Node extension host** (Advanced); sample `examples/vsix-sample-extension`; palette runs `main` with `registerCommand` + `showInformationMessage` shim |
| Markdown preview | TOC sidebar (heading jump); **Export HTML** (standalone file, embedded images); Monaco code colour in preview; Mermaid (lazy); relative images via `convertFileSrc`; links via `resolve_preview_link` |
| Undo / PDF | Document undo/redo on text, SVG, and `.unibrow` timeline (grey when stack empty); PDF thumb strip syncs active page; plain-text print preview; export PDF from Markdown/DOCX print HTML; matrix in [commands-undo-redo.md](commands-undo-redo.md) |
| Packaging | [packaging.md](packaging.md) stubs for Windows MSI/NSIS and Linux AppImage/deb |
| Chrome | Tab drag reorder, duplicate tab (Ctrl+Shift+K), **reopen closed tab** (Ctrl+Shift+T, last 10 paths/URLs), close others / close to the right, **mute tab** stub (https + media), middle-click close, pin (Ctrl+click); omnibox Places/recent/URL suggestions; [chrome.md](chrome.md) |
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
| Image edit | PNG/JPEG: rotate, flip, save-as (new file); crop export; **RGB histogram** side panel; grey **Filters** / **Curves** stubs |
| `.unibrow` v2 | `format_version` 2 + validation on open; v1 still loads; [unibrow-format-migration.md](unibrow-format-migration.md) |
| Multi-window | Shared prefs/mods; independent tabs per window; session v2 restores all windows; [multi-window.md](multi-window.md) |
| OS file drop | Drop onto any window → open in that window; multi-file → multiple tabs |
| Print preview | Markdown/DOCX print CSS preview tab; export PDF opens same preview first |
| Keyboard cheat sheet | Palette **Show keyboard shortcuts** → searchable overlay (`get_keyboard_shortcuts`: defaults + `keybindings.json`); `unibrowser://keys` page |
| Reader mode | **Toggle reader mode** on https tabs (injected declutter CSS); grey in Command Manager when not http(s) |
| Bookmark bar lite | Optional bar under chrome (pins + up to 8 quick URLs); Settings toggle; layout uses `#content-host` bounds so webview never sits under the bar |
| Tab capture | Palette **Capture visible tab** → PNG in Downloads + new tab (Linux WebKitGTK visible snapshot on https; raster encode / canvas fallback elsewhere; diagnostics + grey disable reason when native https capture missing) |
| Text compare (stub) | Palette **Compare text files (experimental)** — Monaco diff, side-by-side or unified toggle |
| Clip markers | Named markers on **video** timeline clips in `.unibrow`; jump from clip list |
| Contributing | [docs/CONTRIBUTING.md](CONTRIBUTING.md) — build, checks, proving mod, PR expectations |
| WebView2 downloads | Gap documented + issue template; `webview_download.rs` spike (`DownloadStarting` stub on Windows) — [browser-shell.md](browser-shell.md) |
| VSIX activation | `onCommand:` activation at stub execute; activated extensions on diagnostics |
| Image eyedropper | Stub on PNG/JPEG viewer: sample RGB in status bar |
| Architecture | Runtime mermaid (shell ↔ engine ↔ mods ↔ sidecars) in [architecture.md](architecture.md) |
| Image compare (stub) | Palette **Compare images** side-by-side with split slider + synced zoom; grey off image tabs |
| Workspace folders | Files **Set workspace** → capped recursive tree; open files from tree; [workspace-folders.md](workspace-folders.md) |
| Git status lite | Dirty **●** on listed paths when folder is in a git repo (`git status --porcelain`); Commit/Push/Sync grey in palette |
| Document autosave | Settings interval for text/SVG save + DOCX reader font prefs; **off by default** |
| Notification center | Status bar **Alerts** drawer (last 20); chrome toasts unchanged |
| Safe mode | `UNIBROWSER_SAFE_MODE=1` or Settings → disable user mods; [safe-mode.md](safe-mode.md) |
| Privacy | No analytics SDKs; [privacy.md](privacy.md) + About **No telemetry** line |
| Demo pack | [examples/demo-pack/](../examples/demo-pack/) sample txt/md/svg/png/unibrow |
| Crash logs | Panic hook writes `crash-logs/` under config; About/Diagnostics **Open crash logs folder** (local only) |
| Updates | Settings **Check for updates** opens releases URL only — [updates.md](updates.md) |
| Go to tab | **Ctrl+Shift+A** (or T) / palette **Go to tab** — fuzzy search across all windows |
| Diagnostics memory | Linux RSS from `/proc/self/status` when available |
| Content fullscreen | **F11** hides chrome for the active viewer; **Escape** exits — [fullscreen-content.md](fullscreen-content.md) |
| Media PiP stub | Palette **Picture-in-picture** on video/audio tabs; system PiP when webview allows, else grey reason |
| Spellcheck expansion | Experimental setting also targets DOCX preview + EPUB reading notes |
| Diagnostics | `unibrowser://diagnostics` — mods, ffmpeg, WebView stack, recent error log ring buffer |
| Settings search | Filter Settings sections by typing in the search field |
| Roadmap | [roadmap.md](roadmap.md) refreshed vs blitz shipped vs open (VSIX JS, WebView2 downloads, OCR, full encode) |
| CI | [`scripts/check-workspace.sh`](../scripts/check-workspace.sh), GitHub Actions + [ci.md](ci.md) (Origin contract) |
| Files | Breadcrumbs, favourites, sort/filter, multi-select, drag-drop, zip peek, magic-byte sniff, virtualised large listings |
| Commands | Command Manager (dense mode), palette + chords (`Ctrl+Shift+P` default, user override via `keybindings.json`), workspace layout presets, import/export JSON; VSIX commands on **Extensions** tab (stub run) |
| Viewers | Monaco text, PDF (multi-engine proving banner + lazy raster proxy), DOCX, SVG, CSV, EPUB search, images, video/audio, MIDI stub, `.unibrow` board/timeline + MIDI scrub status |
| Projects | Templates, `project.json` autosave backup, share zip import/export; timeline **H.264 export** (ffmpeg, fast/balanced) with progress toasts |
| Mods | Bundled `base-*`, mod portal (context install panel + Open VSX / `.ubmod` stubs), Rhai scripts, Linux user `mods/` + dynamic cdylib (feature flag); VSIX `activationEvents` stored |
| Toolbars | Floating bars drag-anywhere; screen coords across monitors/DPI ([toolbars.md](toolbars.md)); right-click **Reset this toolbar**; theme + high-contrast on CM and floating bars |
| Cross-format | `unibrowser.font.size.increase` live on text, SVG, DOCX, EPUB, HTTP user-style; per-origin page zoom (+/-) with palette **Reset zoom for all sites** ([commands-cross-format.md](commands-cross-format.md)) |
| Reader mode | Per-origin reader on/off remembered in `web-reader-prefs.json` when you toggle on https tabs |
| Places | Files manager **Places** (recent + pinned); palette fuzzy open for paths ([commands-cross-format.md](commands-cross-format.md) palette) |
| Plugin sample | Build `examples/hello-world.ubmod` locally + [plugin-sdk.md](plugin-sdk.md) (`mod.hello` cdylib) |
| Recovery | Text/SVG/JS crash backups (opt-in, Settings/About); `.unibrow` `project.json` debounced backups + launch prompt; **session restore defers** paths with pending project or editor crash backups ([session-restore.md](session-restore.md)) |
| https loading UX | Spinner over content host until load finish or document title; 30s slow-network message — [chrome.md](chrome.md) |
| i18n chrome | Status bar a11y/zoom, CM tab pin tooltips, tab strip hints, downloads manager strings in en-NZ / en-US |
| Downloads path | `get_downloads_dir` / `dirs` crate: Windows Known Folder, Linux XDG, fallback `~/Downloads` |
| Timeline snap | Optional **Snap clip moves and edges to playhead** on `.unibrow` track (drag block body or edge handles) |
| Timeline undo | Clip move/trim, duck, fade push `project.json` onto session undo stack; **Undo** restores one step |
| PDF thumbs | Multi-page PDFs: horizontal lazy thumbnail strip (intersection observer) above the page canvas |
| Mod portal | Per user native mod / VSIX **Enable** / **Disable** (independent of safe mode); restart note on toggle |
| File associations | Documented: opens paths in-app, no OS default handler claims yet — [file-associations.md](file-associations.md) |
| About feedback | **Report an issue** opens GitHub issue template chooser (`UNIBROWSER_REPORT_ISSUE_URL` override) |
| Downloads | Linux WebKit `on_download` → shell; **https + platform path** fetches then opens file tab; **Content-Disposition** renames generic hints; failed fetch toast + history ([browser-shell.md](browser-shell.md)) |
| Settings reset | Confirmable **Reset onboarding**, **Clear recent files**, **Reset floating toolbars** |
| Command Manager | Scrollable command chips; horizontal tab strip; pin + layout buttons stay visible |
| Tab sleep (stub) | Background https tabs **sleeping** after 10 min idle (UI only) — [tab-sleep.md](tab-sleep.md) |
| Print selection | Palette **Print selection** for Monaco / DOCX fragment via print CSS |
| A11y audit | Done vs open checklist in [ux-a11y.md](ux-a11y.md) |
| ADR 0020 | Linux download fetch → file tab — [adr/0020-linux-download-open-as-tab.md](adr/0020-linux-download-open-as-tab.md) (indexed in [index.md](index.md)) |
| Omnibox HTTP warn | Settings **Warn before opening HTTP sites** (default on); confirm on `http://` navigation |
| Media keys | Focused video/audio: Space play/pause, arrows seek ±5s |
| DOCX lite | Mammoth preview + **Save edits as HTML** sibling `.ub-edit.html` — [docx-roundtrip.md](docx-roundtrip.md) |
| EPUB notes | Reading notes in prefs; **Export reading notes** → `{book}.reading-notes.txt` beside the EPUB |
| A11y prefs | High contrast, reduce motion (system/on/off + `prefers-reduced-motion`), optional **larger hit targets**, **Focus mode** denser chrome; skippable tour + Settings checklist ([onboarding.md](onboarding.md), [ux-a11y.md](ux-a11y.md)); CM tab keys, palette Escape, focus rings |
| WASM proving | When `examples/hello-wasm/hello.wasm` is present, palette **`mod.wasm.hello`** host stub (no wasmtime); [wasm-t2-status.md](wasm-t2-status.md) |
| Palette history | Last **10** executed commands at top of palette (**Recent:**); stored in config only |
| Context help | **F1** / **What's this** opens `unibrowser://docs/{slug}` from repo `docs/` (dev checkout) |
| PDF annotate stub | Highlight rects in `{file}.pdf.ub-annotations.json`; stamp/ink grey — [pdf-annotate.md](pdf-annotate.md) |
| In-app docs | `unibrowser://docs/*` with TOC from [index.md](index.md), back link, in-page search |
| SVG edit lite | Inline preview; fill/stroke panel for path/text; grey path boolean/simplify |
| Timeline encode intents | H.264 export honours **fade** (two video clips), **clip fade in/out**, and **duck_under_video** on audio (approx); timeline UI for video fades; [timeline-encode-intents.md](timeline-encode-intents.md) |
| Deep find | **Ctrl+Shift+F** filename + text scan; opens text at line; limits in [deep-find.md](deep-find.md) |
| **base-js** | `.js`/`.jsx`/`.mjs`/`.cjs`/`.ts`/`.tsx` open in Monaco (`javascript` / `typescript` language ids); `.rhai` stays plain text |
| Scripting tip | Shown only when JS/TS still open in **base-text**; hidden when **base-js** handles the file |
| Bottom panel | **Output** (Rhai) + grey **Terminal** tab (planned) — [terminal-panel.md](terminal-panel.md) |
| Web proxy prefs | System / direct / manual host+port persisted; not applied to webviews yet — [network-proxy.md](network-proxy.md) |
| Branding | Window title **Unibrowser — {tab}**; icons documented in [branding.md](branding.md) |
| Workspace quick open | **Ctrl+P** fuzzy file find under pinned workspace root (separate from omnibox / deep find **Ctrl+Shift+F**) |
| MIDI audio render stub | Viewer + palette command grey unless **ffmpeg** and **fluidsynth** on PATH — [midi-audio-render.md](midi-audio-render.md) |
| Demo artifacts | `examples/demo-pack/` stays small; `hello-world.ubmod` built locally (`scripts/build-hello-ubmod.sh`), not committed |
| i18n | Chrome strings `en-NZ` / `en-US`; Language toggle in Settings |
| Scripting | Rhai demo from palette → output panel; `.rhai` plain text; JS/TS via **base-js** Monaco |
| DXF | Proving mod wireframe + disabled CAD tools ([proving-mod-dxf.md](proving-mod-dxf.md)) |
| `.unibrow` assets | Orphan files under `assets/` listed in project UI; remove with confirm when unreferenced |
| `.unibrow` labels | Inline rename for board node and timeline clip labels (double-click) |
| Web find | Ctrl+F on https tabs via content webview find; grey off when platform stub says unavailable |
| Files multi-open | Checkbox / Shift / Ctrl selection; **Open selected** and Enter batch-open via `open_selected_paths` |
| Docs | [docs/index.md](index.md), [getting-started.md](getting-started.md), [report-visuals.md](report-visuals.md), [report-shots/README.md](report-shots/README.md), ADRs 0001–0020 |
| Quality | `cargo test --workspace`, path/transfer fuzz tests, `./scripts/smoke-desktop-qa.sh` |

## How to run

```bash
cargo test --workspace
cd crates/unibrowser-desktop && npm ci && npm run build
npm run tauri dev
```

CLI version: `cargo run -p unibrowser-cli -- --version`

Report screenshots: [report-visuals.md](report-visuals.md) and `./scripts/capture-report-screenshots.sh`.

## Metrics (Wed report skeleton)

Re-run `./scripts/blitz-metrics.sh` before publishing (counts below from 2026-10-05T22:15Z).

| Metric | Value (snapshot) |
| --- | --- |
| Workspace Rust sources (`crates/` + `mods/`) | 129 `.rs` files · ~25.2k lines |
| Desktop TypeScript (`crates/unibrowser-desktop/src`) | 63 `.ts` files · ~13.9k lines |
| Bundled format mods (`mods/`) | 12 packages (`base-*` + `toy-markdown`) |
| Architecture decision records | 20 in `docs/adr/` |
| Workspace crates (root `Cargo.toml` members) | 25 members incl. desktop, engine, mod-host, media, project, VSIX, cdylib stub |

### Workspace crates (core)

| Crate | Role |
| --- | --- |
| `unibrowser-desktop` | Tauri shell + Vite UI |
| `unibrowser-engine` | Tabs, commands, Command Manager |
| `unibrowser-mod-host` | Mod load + registry |
| `unibrowser-mod-api` | Stable mod ABI types |
| `unibrowser-project` | `.unibrow` manifest + zip |
| `unibrowser-media` | ffmpeg sidecar probe/encode |
| `unibrowser-vsix` / `unibrowser-extension-host` | VSIX metadata + stub host |
| `unibrowser-script-host` | Rhai scripting |
| `unibrowser-cli` / `unibrowser-app` | CLI + thin app entry |
| `unibrowser-build-info` | Version/git embed |

### Proving mods (placeholder UX)

| Mod | Proves | Grey / stub |
| --- | --- | --- |
| `base-dxf` | CAD wireframe viewer | Measure, Export STEP ([proving-mod-dxf.md](proving-mod-dxf.md)) |
| `base-audio` | Waveform + HTML audio | DAW split/stem/automation commands |
| `base-video` | HTML video + scrub thumbs | PiP when webview lacks API |
| `base-files` | Git dirty badges | Commit / push / sync SCM |
| `base-png` | Image tools | Filters stub; compare/capture hooks |
| `toy-markdown` | Markdown preview path | — |

### Open gaps (ranked for post-blitz)

Recent blitz ships (tab reopen/mute, PDF text copy, web zoom reset-all, reader prefs, omnibox `file://` hardening) are **done** — not listed below.

| Rank | Gap | Why it matters |
| --- | --- | --- |
| 1 | VSIX JS extension host (full) | Node sidecar spike ships (`main` + minimal shim); most `vscode` API still missing |
| 2 | WebView2 download wiring | Windows default browser workaround |
| 3 | OCR sidecar | Announced stub, no Tesseract pool |
| 4 | Full timeline / DAW encode | ffmpeg H.264 + approximate duck/fade; no full DAW mix bus |
| 5 | Native mod load on Win/macOS | Linux `mods/` + cdylib today |
| 6 | EPUB/DOCX fidelity | Asset URLs; Mammoth layout |
| 7 | Web proxy applied to webviews | Prefs persist only |
| 8 | i18n depth | Chrome only (`en-NZ` / `en-US`) |
| 9 | Cloud / collab | Share zip only |
| 10 | Binary / heavy CAD DXF | Proving parser + wireframe only; friendly empty state, not full geometry |

## Gaps (honest)

| Item | Notes |
| --- | --- |
| VSIX JS extension host | Opt-in Node sidecar + sample extension ([extension-host.md](extension-host.md)); not a full VS Code host |
| OCR | UI placeholder; Tesseract sidecar not built ([ocr-sidecar.md](ocr-sidecar.md)) |
| Spellcheck | Monaco still has no built-in dictionary; experimental browser spellcheck covers text mirror + DOCX + EPUB notes |
| Timeline encode | H.264 when ffmpeg present; duck/fade approximations only; no stem/mix bus export |
| Windows/macOS dynamic mods | Linux user `mods/` today |
| i18n | Viewer copy not fully translated; only chrome catalogues today |
| Cloud / collab | Share zip only; no live multi-user |

## Known bugs / rough edges

Severity: **P1** user-blocking or data risk · **P2** major UX gap · **P3** polish / edge case.

| Sev | Issue | Notes |
| --- | --- | --- |
| P1 | WebView2 downloads (Windows) | `DownloadStarting` not wired; Linux WebKit path improved (fetch to hinted path, no Finished duplicate). Use Advanced → system browser on Windows ([browser-shell.md](browser-shell.md)). |
| P2 | EPUB assets | Spine HTML only; embedded images/fonts often missing until asset URL resolution. |
| P2 | DOCX fidelity | Mammoth layout limits; no `.docx` write-back (HTML sibling only — [docx-roundtrip.md](docx-roundtrip.md)). |
| P2 | VSIX JS host | Node spike only; grey when pref off or `node` missing ([extension-host.md](extension-host.md)). |
| P2 | Web proxy prefs | Persisted but **not applied** to embedded https webviews ([network-proxy.md](network-proxy.md)). |
| P3 | PDF text copy | Needs pdf.js **text layer**; scanned/raster-only pages have no selectable text (Copy / Print selection grey). |
| P3 | Large PDF open | Profile with `UNIBROWSER_PROFILE_OPEN=1` if open feels slow. |
| P3 | Spellcheck experiment | No underlines inside Monaco; mirror textarea / DOCX / EPUB notes only. |
| P3 | OCR | Button disabled until Tesseract sidecar ships ([ocr-sidecar.md](ocr-sidecar.md)). |
| P3 | Integrated terminal | Bottom **Terminal** tab disabled; use system shell ([terminal-panel.md](terminal-panel.md)). |

## Demo script (~10 min)

0. **Sample pack** — Follow the keyed script in [examples/demo-pack/README.md](../examples/demo-pack/README.md) (palette **Ctrl+Shift+P**, help **F1**, go-to-tab **Ctrl+Shift+A**).
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
