# Phased roadmap

Phases are capability-based, not calendar estimates. Each phase should leave the host shippable and testable.

**36h blitz reality (on `main`):** the desktop shell is far past Phase 0–1 skeletons. This section records what the blitz **shipped** versus what remains **open**.

| Area | Shipped in blitz (high level) | Still open |
| --- | --- | --- |
| Shell | Tauri 2 tabs, omnibox, multi-window + session v2, split view (experimental), downloads manager, status bar, tab chrome | WebView2 **DownloadStarting** (use system browser interim) |
| Commands | Palette, keybindings, Command Manager + float bars, cross-format font/zoom, undo/redo matrix, export PDF | Drag commands between CM tabs (Phase 2 toolbar UX) |
| Formats | Monaco text, pdf.js PDF, Mammoth DOCX, EPUB spine, images, audio waveform, `.unibrow` board/timeline, H.264 export stub | Full NLE encode (mix, fades, audio), EPUB asset URLs |
| Mods | Bundled `base-*`, mod portal + offline catalog, `.ubmod` sample, DXF proving mod | VSIX **JS host** (metadata + stub execute only) |
| Trust / legal | NOTICE, third-party docs, trust banners | REUSE/SPDX CI automation |
| OCR | UI placeholder + docs | Tesseract sidecar not built |
| Collab | Share zip import/export | Live multi-user |

Strategic order unchanged: **context switching** and the **command model** first; grow **one engine lane at a time** with a **proving `base-*` mod**; **lazy conversion** until encode ([ADR 0011](adr/0011-lazy-conversion-native-until-encode.md)).

**Engine principle:** core engines live in the product; each lane ships one bundled mod on a concrete format. Prefer **embedding trusted FOSS** (Monaco, pdf.js, ffmpeg sidecar, …) over rewrites — [architecture.md](architecture.md).

## Engine lanes (core + proving mod)

| Lane | Engine fundamental (host) | Proving mod | Format | FOSS direction (verify before ship) |
|------|---------------------------|-------------|--------|-------------------------------------|
| Text | Editor bridge, save/sync | `base-text` | `.txt` | Monaco (**done** in shell) |
| Browser | Tabs, `BrowserLocation`, WebView | `base-web`, `base-files` | HTML / HTTPS / dirs | Tauri WRY; optional CEF |
| Raster | Bitmap surface, capabilities | `base-png` | PNG / JPEG / … | image crate; canvas in webview |
| Vector | SVG scene graph, path tools | `base-svg` | SVG | webview DOM |
| Document | Office context, structured doc | `base-docx` | DOCX | mammoth; LOK T3 later |
| Code | LSP hooks, same registry | VSIX + `base-text` / grammars | source files | Monaco + Open VSX grammars |
| Scripting | Sandbox / extension host | `base-js`, Rhai | JS / `.rhai` | Rhai demo; Extism TBD |
| Video | Timeline + encode contract | `base-video` | MP4 / WebM | **ffmpeg** LGPL **T3**; H.264 stub shipped |
| Audio | Preview vs export clips | `base-audio` | WAV / MP3 / … | **ffmpeg** **T3**; waveform shipped |
| CAD | 2D/3D view host | `base-dxf` | DXF | dxf-parser + three.js wireframe |

**Multi-engine mods:** `base-pdf` (vector + raster + document + pdf.js); `.unibrow` timeline composes video/audio/raster stubs.

Satellite: `toy-markdown`; EPUB (Rust spine preview, not epub.js bundle).

## Phase 0 — Prove the abstraction

| Deliverable | Status |
| --- | --- |
| Rust engine: command registry, enablement, modes, toolbar | **Done** (desktop + tests) |
| Mod loader + manifest + API version check | **Done** (bundled); dynamic Linux optional |
| Bundled `base-text`, `base-png`, `toy-markdown` mods | **Done** (+ full `base-*` set in tree) |
| egui demo (`unibrowser-app`) | **Done** (dev/demo) |
| Architecture docs + ADRs | **Done** |

## Phase 1 — MVP browser (files + web)

| Deliverable | Status |
| --- | --- |
| Tauri 2 shell: tabs, omnibox, back/forward/reload | **Working** |
| Child content WebView for `https://` | **Working** |
| `base-files` directory listing and open in tab | **Working** |
| Download → new tab / manager | **Working** (Linux WebKit; WebView2 gap) |
| Palette, keybindings, Command Manager + `toolbars.json` | **Working** |
| Monaco text edit + save | **Working** |
| Context pin, multi-window, session restore v2 | **Working** |
| Office / audio manager tabs (`planned`) | **Stub** → partial (DOCX/EPUB/audio proving) |

## Phase 2 — Context lanes (Text → Office → Code → Image)

| Lane | Status (blitz) |
| --- | --- |
| **Text** | Monaco + markdown preview; print preview |
| **Office** | Mammoth DOCX view; export PDF / print CSS |
| **Code** | Monaco; VSIX metadata only |
| **Raster / Vector** | Image toolbar (zoom, crop, rotate, save-as); SVG edit |
| **Scripting** | Rhai palette demo |

**Toolbar UX:**

| Item | Status |
| --- | --- |
| Reorder within tab | ✓ |
| Float + position | ✓ |
| Drag between tabs | Open |
| Drag out to new floating bar | Open |

## Phase 3 — Mixed media + lazy conversion

| Milestone | Status (blitz) |
|-----------|----------------|
| P3.0 `.unibrow` package (`project.json` v2, assets, derived) | **Working** (directory + zip) |
| P3.1 Embed assets; derived cache | **Partial** (zip peek, share zip) |
| P3.2 Timeline heterogeneous clips | **Stub** (markers, H.264 export when ffmpeg present) |

ffmpeg sidecar: encode + waveform; not a full render graph.

## Phase 4 — Document and CAD depth

- **CAD:** `base-dxf` proving mod (wireframe) — **stub shipped**
- EPUB: spine + search + chapter nav — **partial**; embedded assets open
- Advanced web: per-origin font/zoom, devtools F12 — **partial**

## Phase 5 — Ecosystem and hardening

- **Compiled mod SDK** — `.ubmod` hello sample; cdylib Linux flag
- **T2 WASM** / **T1 signed native** — policy in ADR 0002; not productised
- VSIX compat track — see below
- Crash reporting automation — crash recovery backups only

## VSIX compatibility (parallel track)

| Phase | Status (blitz) |
| --- | --- |
| V0 ADR + mapping | Done |
| V1 Unpack; commands / keybindings in metadata | **Working** |
| V2 Languages, grammars, themes → Monaco | Open |
| V3 Sandboxed JS + `vscode` shim | **Not started** (stub execute only) |
| V4 Mod Portal + Open VSX (opt-in network) | **Working** (search/install stubs) |
| V5 Context recommendations | **Partial** (mod portal panel) |

## Mod Portal (product)

| Capability | Status |
|------------|--------|
| Browse / install Open VSX + `.ubmod` | Working (network opt-in) |
| Offline catalog filter | Working |
| Can't open file → suggest mods | Working |
| Install from file | Working |

## Bundled mod rollout

All listed `base-*` mods exist in-tree with proving handlers; rollout is **maintenance and depth**, not greenfield.
