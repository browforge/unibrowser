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

## Phase 1 — MVP file host

- Real open/save, tabs, dirty indicators.
- **Mods:** plain text, Markdown (view/edit), PNG/JPEG view+metadata, SVG view+text edit, HTML file mod (embedded WebView via Tauri), PDF view (pdf.js in WebView), DOCX basic preview, DXF view.
- Toolbar layout persistence; preferences.
- Tauri 2 shell (per ADR 0001) replacing or wrapping egui chrome for document surfaces.

## Phase 2 — Document and CAD depth

- DOCX edit (permissive JS stack or LOK sidecar, AGPL-labelled where required).
- DXF edit via permissive parser or GPL helper process.
- EPUB reader mod; optional LibreOfficeKit sidecar for office fidelity.

## Phase 3 — Media and live web

- Video/audio with ffmpeg (LGPL build) sidecar; subtitle/text overlays.
- Live page mod: guest WebView + user stylesheet commands (Stylus-like).

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
