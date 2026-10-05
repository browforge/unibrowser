# Phased roadmap

Phases are capability-based, not calendar estimates. Each phase should leave the host shippable and testable.

## Phase 0 — Prove the abstraction (this repository)

**Goal:** Same command, different backends; grey-out-not-hide; S-key palette; mod manifests.

| Deliverable | Status |
| --- | --- |
| Rust engine: command registry, enablement, modes, toolbar | Skeleton |
| Mod loader + `info.json` + API version check | Skeleton |
| Bundled `base-text`, `base-png`, `toy-markdown` mods | Skeleton |
| egui demo window | Skeleton |
| Architecture docs + ADRs | Done |

**Exit criteria:** `cargo test` green; demo increases font size in text mod, greyed on PNG without text selection.

## Phase 1 — MVP browser (files + web)

| Deliverable | Status |
| --- | --- |
| Tauri 2 shell (`unibrowser-desktop`): tabs, omnibox, back/forward/reload | **Working (Linux)** |
| Child content WebView for `https://` (`base-web`, `tauri` `unstable` multiwebview) | **Working** |
| `base-files` directory listing and open in tab | **Working** |
| `BrowserLocation` per tab; download → new tab (command + `on_download` emit) | **Working / stub** ([downloads.md](downloads.md)) |
| Command registry, S-key palette, font-size grey-out on file docs | **Working** |
| egui spike (`unibrowser-app`) | Kept as harness |
| Open/save to disk, dirty persistence, toolbar JSON on disk | Deferred |
| SVG, PDF, DOCX, DXF format mods | Deferred |

## Phase 2 — Document and CAD depth

- DOCX edit (permissive JS stack or LOK sidecar, AGPL-labelled where required).
- DXF edit via permissive parser or GPL helper process.
- EPUB reader mod; optional LibreOfficeKit sidecar for office fidelity.

## Phase 3 — Media and advanced web

- Video/audio with ffmpeg (LGPL build) sidecar; subtitle/text overlays.
- Hardened web mod: permissions, user stylesheet commands (Stylus-like), isolated guest profiles.

## Phase 4 — Ecosystem

- Signed mod distribution; WASM tier for untrusted mods.
- Contribution docs; optional “studio” presets (Affinity-style shared toolbar layouts).

## Phase 5 — Hardening

- Crash reporting, sandbox audit, REUSE/SPDX licence CI, trademark-cleared product name.

## MVP format mod order (recommended)

1. `base-text` — plain text / code
2. `base-markdown` — lightweight markup
3. `base-raster` — PNG, JPEG
4. `base-svg` — vector + text elements
5. `base-html` — file + optional live page (WebView mod)
6. `base-pdf` — view-only
7. `base-docx` — basic edit
8. `base-dxf` — view-first
