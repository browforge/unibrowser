# Phased roadmap

Phases are capability-based, not calendar estimates. Each phase should leave the host shippable and testable.

**Strategic order:** nail **context switching** and the **command/manager model** first; add format depth and **lazy conversion** pipelines without building huge editors prematurely.

## Phase 0 — Prove the abstraction (this repository)

**Goal:** Same command, different backends; grey-out-not-hide; command palette with standard keybindings; mod manifests.

| Deliverable | Status |
| --- | --- |
| Rust engine: command registry, enablement, modes, toolbar | Skeleton |
| Mod loader + manifest + API version check; **compiled payload** (T0 static dev; `library` field) | Skeleton |
| Bundled `base-text`, `base-png`, `toy-markdown` mods | Skeleton |
| egui demo window | Skeleton |
| Architecture docs + ADRs | Done |

**Exit criteria:** `cargo test` green; demo increases font size in text mod, greyed on PNG without text selection.

## Phase 1 — MVP browser (files + web)

| Deliverable | Status |
| --- | --- |
| Tauri 2 shell: tabs, omnibox, back/forward/reload | **Working (Linux)** |
| Child content WebView for `https://` | **Working** |
| `base-files` directory listing and open in tab | **Working** |
| Download → new tab | **Working / stub** ([downloads.md](downloads.md)) |
| Palette, keybindings, Command Manager + `toolbars.json` | **Working** |
| Monaco text edit + save (text context) | **Working** |
| Context pin (`context_pin`, Pin tab / Follow context in shell) | **Working** |
| Office / audio manager tabs (`planned`) | **Stub** |
| Vision + ADR 0011/0012 (conversion + UX benchmarks) | **Done** |

**Not in Phase 1:** full office suite, NLE, PDF-on-timeline implementation.

## Phase 2 — Context lanes (Text → Office → Code → Image)

Rollout matches Command Manager tabs ([ui-commands.md](ui-commands.md)):

| Lane | Deliverable |
| --- | --- |
| **Text** | Rich text where needed; keep Monaco for plain/code-like |
| **Office** | DOCX/XLSX view-edit; optional LOK sidecar (AGPL-labelled) |
| **Code** | LSP, format, debug hooks — same command registry |
| **Image** | Raster + SVG; text layers; GIMP-style tool slots on floating bars |

**Toolbar UX (GIMP + SOLIDWORKS bar):**

| Item | Target phase |
| --- | --- |
| Reorder within tab | Phase 1 ✓ |
| Float + position | Phase 1 ✓ |
| Drag between tabs | Phase 2 |
| Drag out to new floating bar | Phase 2 |
| Pin context UI in chrome | Phase 2 |

## Phase 3 — Mixed media + lazy conversion

Implement [ADR 0011](adr/0011-lazy-conversion-native-until-encode.md) for real:

- Timeline/project mod (Resolve-style **page** inside stable chrome).
- PDF clips vector-native; proxy raster for scrub; rasterise at encode.
- MIDI clips; synth preview; PCM bake on export.
- ffmpeg (LGPL) sidecar for audio/video encode — not linked into Apache core.

## Phase 4 — Document and CAD depth

- EPUB; DXF view/edit; advanced web (user styles, permissions).

## Phase 5 — Ecosystem and hardening

- **Compiled mod SDK** — `cdylib` + C ABI (or WIT) registration; `.ubmod` package (manifest + binary).
- **T2 WASM** default for untrusted community; **T1 signed native** for vendors / closed-source / paid mods ([ADR 0002](adr/0002-mod-system-and-sandboxing.md)).
- Signed mods; studio presets (Affinity-style shared layouts).
- Crash reporting; sandbox audit; REUSE/SPDX CI.
- **VSIX compat** (phased, [ADR 0014](adr/0014-vsix-compatibility-layer.md)): optional Open VSX (power UI, opt-in) — separate from native ABI.

## VSIX compatibility (parallel track)

Native `info.json` mods stay canonical ([mod-system.md](mod-system.md)). `.vsix` is import/compat only.

| Phase | Deliverable |
| --- | --- |
| V0 | ADR + contribution mapping |
| V1 | Unpack `.vsix`; `contributes.commands` / keybindings |
| V2 | Languages, grammars, snippets, themes → Monaco |
| V3 | Sandboxed JS + stub `vscode` module |
| V4 | Optional Open VSX browse/install |

## MVP format mod order (unchanged, aligned to manager tabs)

1. `base-text` — plain text
2. `base-markdown` — lightweight markup  
3. `base-raster` / `base-svg` — image context  
4. `base-html` / `base-web` — web context  
5. `base-pdf` — view-first; timeline-native later  
6. `base-docx` — office context  
7. `base-dxf` — view-first  
8. `base-audio` / `base-midi` — audio context (preview vs export)
