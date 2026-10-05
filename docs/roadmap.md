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

- **Tauri 2** shell: omnibox, tab strip, back/forward where meaningful ([ADR 0001](adr/0001-shell-and-rendering.md), [ADR 0009](adr/0009-browser-as-unified-shell.md)).
- **`BrowserLocation`** per tab; download handler defaults to **open in new tab**.
- **Bundled `base-files` mod:** folder navigation; open files in tabs (no separate file-manager app).
- **Bundled `base-web` mod:** `https://` tabs on system WebView; shared command layer with file tabs.
- Open/save, dirty indicators, toolbar layout persistence.
- **Format mods:** plain text, Markdown, PNG/JPEG, SVG, HTML on disk, PDF view, DOCX basic preview, DXF view.

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
