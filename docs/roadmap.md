# Phased roadmap

Phases are capability-based, not calendar estimates. Each phase should leave the host shippable and testable.

**Strategic order:** nail **context switching** and the **command/manager model** first; grow **one engine lane at a time** with a **proving `base-*` mod**; add **lazy conversion** without building huge editors prematurely.

**Engine principle:** core engines live in the product; each lane ships one bundled mod on a concrete format. Prefer **embedding trusted FOSS** (Monaco, pdf.js, ffmpeg sidecar, …) over rewrites — [architecture.md](architecture.md).

## Engine lanes (core + proving mod)

| Lane | Engine fundamental (host) | Proving mod | Format | FOSS direction (verify before ship) |
|------|---------------------------|-------------|--------|-------------------------------------|
| Text | Editor bridge, save/sync | `base-text` | `.txt` | Monaco (**done** in shell) |
| Browser | Tabs, `BrowserLocation`, WebView | `base-web`, `base-files` | HTML / HTTPS / dirs | Tauri WRY; optional CEF |
| Raster | Bitmap surface, capabilities | `base-png` | PNG | libvips / image crate; canvas in webview |
| Vector | SVG scene graph, path tools | `base-svg` | SVG | SVG.js / svgo; webview DOM |
| Document | Office context, structured doc | `base-docx` | DOCX | mammoth, docx-preview; LOK T3 later |
| Code | LSP hooks, same registry | VSIX + `base-text` / grammars | source files | Monaco + Open VSX grammars |
| Scripting | Sandbox / extension host | `base-js` | JS | QuickJS/Rhai/Extism (policy TBD) |
| Video | Timeline + encode contract | *TBD* (`base-mp4`?) | container *TBD* | **ffmpeg** LGPL **T3** (decided backbone) |
| Audio | Preview vs export clips | *TBD* (`base-wav` / `base-midi`?) | WAV / MIDI candidates | **ffmpeg** **T3**; MIDI-native per ADR 0011 |
| CAD | 2D/3D view host | `base-dxf` | DXF | dxf-parser + three.js; GPL helpers T3 |

**Multi-engine mods:** `base-pdf` (vector + raster + document + pdf.js); video/timeline (video + audio + raster; document/vector via import). See [architecture.md](architecture.md).

Satellite mods: `base-markdown` (text); EPUB (document + raster); not separate engine lanes.

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

Rollout matches Command Manager tabs ([ui-commands.md](ui-commands.md)). Each row = **engine fundamental** + **proving mod** milestone.

| Lane | Engine + proving mod deliverable |
| --- | --- |
| **Text** | Text engine stable; `base-text` + Monaco; optional `base-markdown` |
| **Office** | Document engine v1; **`base-docx`** view/edit (mammoth / docx-preview); optional LOK **T3** |
| **Code** | Code context on text engine; LSP hooks; VSIX grammars (V2 track) |
| **Raster / Vector** | **Raster** engine + **`base-png`**; **Vector** engine + **`base-svg`**; GIMP-style floating tool slots |
| **Scripting** | Scripting engine skeleton; **`base-js`** stub (sandbox policy) |

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

- **Video** + **Audio** engines on **ffmpeg** (LGPL **T3 sidecar**); proving mods/containers still TBD (e.g. MP4/WebM, WAV/MIDI).
- Timeline page inside stable chrome; video mod composes **video + audio + raster** engines ([architecture.md](architecture.md)).
- **`base-pdf`** multi-engine: vector-native clips, raster proxies for scrub, document text; pdf.js view layer.
- MIDI clips native in project; synth preview; PCM bake on export via ffmpeg where needed.

## Phase 4 — Document and CAD depth

- **CAD engine** + **`base-dxf`** (dxf-parser + three.js view; GPL R/W optional T3).
- EPUB; advanced web (user styles, permissions) on **browser engine**.

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
| V4 | **Mod Portal** + Open VSX API (opt-in); sideload `.vsix` |
| V5 | Context recommendations; **cannot open file → suggested mods** |

**Catalog (decided — FOSS path):** built-in Mod Portal defaults to **[Open VSX](https://open-vsx.org)** + **native `.ubmod` registry** when available; **no** embedded Microsoft Visual Studio Marketplace; **sideload** `.vsix` / `.ubmod` always ([ADR 0014](adr/0014-vsix-compatibility-layer.md)).

## Mod Portal (product — in scope)

| Capability | Audience |
|------------|----------|
| Browse / search / install (Open VSX + `.ubmod` index) | Everyone (simple copy); power users see licences, compatibility |
| Recommended for context | Command Manager / document type |
| Can’t open this file → suggest mods | Grandma path for new formats |
| Install from file | FOSS sideload (`.vsix`, `.ubmod`) |

## Bundled mod rollout (proving mods first)

Aligned to **engine lanes** above and Command Manager tabs:

1. `base-text` — **Text** engine / `.txt`
2. `base-web` + `base-files` — **Browser** engine
3. `base-png` — **Raster** engine / PNG
4. `base-svg` — **Vector** engine / SVG
5. `base-markdown` — text satellite
6. `base-pdf` — **multi-engine** (vector + raster + document); timeline-native later
7. `base-docx` — **Document** engine / DOCX
8. `base-dxf` — **CAD** engine / DXF
9. `base-js` — **Scripting** engine (stub)
10. audio proving mod — **Audio** engine (ffmpeg-backed; format TBD)
11. video proving mod — **Video** engine (ffmpeg-backed; container TBD)
