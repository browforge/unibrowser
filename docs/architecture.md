# Architecture overview

Unibrowser is a **unified browser shell** (files + web in one tab strip; see [ADR 0009](adr/0009-browser-as-unified-shell.md)). The **host engine** owns navigation, commands, modes, UI chrome, and mod loading; **mods** supply document backends, web surfaces, and the bundled file browser. The skeleton uses **egui/eframe** only to prove commands and toolbars; the product shell is **Tauri + WebView** ([ADR 0001](adr/0001-shell-and-rendering.md)).

**Product emphasis:** world-class **context switching** (stable chrome, auto Command Manager tab, grey-out, pin/customise), **mixed-format projects** with **lazy conversion** ([ADR 0011](adr/0011-lazy-conversion-native-until-encode.md), [ADR 0012](adr/0012-context-switching-ux-benchmarks.md)), and **dual audience** design — everyday defaults, FOSS depth without user-space interference ([ADR 0013](adr/0013-dual-audience-foss-and-everyday-users.md), [vision.md](vision.md)).

## Runtime: shell, engines, mods, sidecars

```mermaid
flowchart TB
  subgraph Shell["unibrowser-desktop (Tauri shell)"]
    UI[Web UI TypeScript]
    CMD[Tauri commands / events]
    WV[Per-window content webview]
  end

  subgraph Engine["unibrowser-engine"]
    REG[Command registry + palette]
    CM[Command Manager state]
    DOC[Document session + enablement]
  end

  subgraph Mods["Mods & extensions"]
    BASE[bundled base-* backends]
    VSIX[VSIX metadata + stub host]
    HOST[unibrowser-mod-host loader]
  end

  subgraph Sidecars["T3 sidecars & bridges"]
    FF[ffmpeg / unibrowser-media]
    SCR[Rhai script host]
    OCR[OCR placeholder]
  end

  UI --> CMD
  CMD --> REG
  CMD --> DOC
  REG --> HOST
  HOST --> BASE
  VSIX --> REG
  DOC --> BASE
  CMD --> WV
  CMD --> FF
  CMD --> SCR
  UI -.->|lazy| OCR
```

Heavy or copyleft stacks stay **out of process** where ADR 0007 requires it; the shell calls sidecars over stdio/HTTP stubs rather than linking them into `unibrowser-engine`.

### Timeline encode intents (`.unibrow`)

Timeline edits in the shell (clip move/trim, per-clip **fade in/out**, **duck under video** on audio) are stored in `project.json` and compiled into a `TimelineEncodeIntent` for ffmpeg **H.264** export: trimmed video segments, optional crossfade when exactly two video clips share a fade transition, and external audio mixed under the video bus at reduced gain when ducking is enabled. The UI does not bake these on every drag; export is when intents become ffmpeg filters. Gaps, MIDI, and embedded video audio are limited—see [timeline-encode-intents.md](timeline-encode-intents.md).

## Engines in the product, formats in mods

Bear’s direction: the **shipped product** owns **engine fundamentals** (session lifecycle, command wiring, surface hosting, conversion contracts). Each **engine lane** is **proven end-to-end** by one bundled **`base-*` mod** on a concrete format. **Format mods may compose multiple engines** (see below). Community mods extend the same APIs; they do not replace the host engine.

**Principle:** **import trusted FOSS** into engines and mods where possible — embed or bridge (Monaco for text, pdf.js for PDF view, **ffmpeg** for video/audio). **Do not rewrite** mature stacks unless the API gap is smaller than the maintenance cost ([ADR 0007](adr/0007-licensing.md): **ffmpeg** and other copyleft media stacks use **LGPL T3 sidecars** (or equivalent isolation), not linkage into `unibrowser-engine`).

| Engine lane | Host engine owns (fundamentals) | Proving `base-*` mod | Proving format | FOSS backbone (verify licence at adoption) |
|-------------|--------------------------------|----------------------|----------------|---------------------------------------------|
| **Text** | Buffer sync, save/dirty, text `CommandContext`, editor host bridge | `base-text` | `.txt` (plain text) | **[Monaco](https://github.com/microsoft/monaco-editor)** (MIT) — **in use** |
| **Document** | Office context, structured doc session, export hooks | `base-docx` (planned) | DOCX | **mammoth**, **docx-preview**; full fidelity → **LOK** **T3** (MPL/AGPL) |
| **Raster** | Bitmap surface, pixels, selection → capabilities, raster encode | `base-png` (skeleton) | PNG | **libvips** / **image** (Rust); canvas/WebGL; **Squoosh** WASM codecs |
| **Vector** | Paths, transforms, SVG scene graph, vector export | `base-svg` (planned) | SVG | **SVG.js** / DOM in webview; **svgo**; optional Fabric/Konva for edit |
| **Scripting** | Sandboxed runtimes, extension-host hooks, mod `activate` | `base-js` (planned) | JS / user scripts | **QuickJS** / **Rhai**; **Extism** WASM; VSIX JS → extension host ([ADR 0014](adr/0014-vsix-compatibility-layer.md)) |
| **Browser** | `BrowserLocation`, tabs, WebView lifecycle, navigation commands | `base-web` / `base-html` | HTML / `https://` | **Tauri WRY**; optional **CEF** ([ADR 0001](adr/0001-shell-and-rendering.md)) |
| **Video** | Timeline/decode contract, proxies, encode at export ([ADR 0011](adr/0011-lazy-conversion-native-until-encode.md)) | *proving mod TBD* (e.g. `base-mp4`) | container *TBD* | **[ffmpeg](https://ffmpeg.org/)** (LGPL build, **T3 sidecar**) — **default media stack**; WebCodecs as optional fast path |
| **Audio** | Clip model, preview vs bake, MIDI-native project data | *proving mod TBD* (e.g. `base-wav`, `base-midi`) | WAV / MIDI *candidates* | **ffmpeg** (probe/decode/encode, **T3**); cpal/rodio for low-latency preview where needed |
| **CAD** | 2D/3D scene host, units, view tools | `base-dxf` (planned) | DXF | **dxf-parser** + **three.js**; **libdxfrw** **GPL T3** for heavy R/W |

```mermaid
flowchart LR
  subgraph HostEngines["Shipped engines"]
    E1[Text]
    E2[Document]
    E3[Raster]
    E3v[Vector]
    E4[Scripting]
    E5[Browser]
    E6[Video]
    E7[Audio]
    E8[CAD]
  end

  subgraph ProvingMods["One proving mod per lane"]
    M1[base-text]
    M2[base-docx]
    M3[base-png]
    M3v[base-svg]
    M4[base-js]
    M5[base-web]
    M6[base-video TBD]
    M7[base-audio TBD]
    M8[base-dxf]
  end

  E1 --> M1
  E2 --> M2
  E3 --> M3
  E3v --> M3v
  E4 --> M4
  E5 --> M5
  E6 --> M6
  E7 --> M7
  E8 --> M8
```

### Multi-engine mods

A **single format mod** is one tab backend but may **bind several host engines**. The mod owns format-specific parsing and project state; engines expose **stable services** (surfaces, buffers, timelines, structured text). Composition is explicit in manifest metadata (future `engines` / `uses` field) and in code — no monolithic “god backend”.

| Example mod | Engines used **directly** | Engines used **indirectly** (import / render / conversion) |
|-------------|---------------------------|------------------------------------------------------------|
| **`base-pdf`** | **Vector** (paths, text objects), **Raster** (page tiles, scrub proxies), **Document** (text runs, office-like selection) | Text (searchable extracted text), Video (timeline clip export) |
| **Video / timeline mod** | **Video**, **Audio**, **Raster** (frame buffers, thumbnails, scope/waveform as images) | Document (titles/captions), Vector (motion graphics import), Browser (live source preview) |

```mermaid
flowchart TB
  subgraph ModPDF["base-pdf mod"]
    PDF[PDF native model]
  end

  subgraph ModVid["Video mod"]
    TL[Timeline project]
  end

  PDF --> VE[Vector engine]
  PDF --> RE[Raster engine]
  PDF --> DE[Document engine]

  TL --> VIE[Video engine]
  TL --> AE[Audio engine]
  TL --> RE2[Raster engine]
  TL -.->|import/render| DE2[Document engine]
  TL -.->|import/render| VE2[Vector engine]
```

**Command Manager context** may still show one primary tab (e.g. Image vs Office) while multiple engines participate; enablement reflects which engine owns the active selection ([ADR 0003](adr/0003-command-handler-enablement.md)).

**Code layout (today):** engine logic stays in `unibrowser-engine`, `unibrowser-desktop`, and shared bridges; **no separate engine crate per lane yet**. Optional future splits (e.g. `unibrowser-engine-media` wrapping ffmpeg RPC) when dependency graphs force it.

**Satellite mods (thin wrappers):** `base-markdown` (mostly **Text**); `base-pdf` is a **multi-engine** mod, not a single-engine lane — see [roadmap.md](roadmap.md).

## Native project format (`.unibrow`)

Mixed-format canvas and timeline work is persisted in a **Unibrowser project package**, not as a Vegas/Premiere-style single-domain project file. **Export** (MP4, flattened PDF, PNG) is a **boundary** out of the project; sources stay native inside the bundle where [ADR 0011](adr/0011-lazy-conversion-native-until-encode.md) allows.

| Concern | Approach (sketch) |
|---------|-------------------|
| **Container** | Directory bundle and/or ZIP single file; **SQLite** optional for asset index / undo metadata inside the bundle |
| **`project.json`** | Versioned graph: spatial + temporal nodes, mod-typed clips, engine bindings |
| **`assets/`** | Embedded files (default for dropped-in media) |
| **`refs/`** | Optional external `file://` links with relink/collect |
| **`derived/`** | Regenerable preview proxies; provenance; not source of truth |

```mermaid
flowchart LR
  subgraph Project[".unibrow package"]
    PJ[project.json]
    A[assets/]
    R[refs/]
    D[derived/]
  end

  subgraph Shell
    TAB[Tabs / canvas / timeline UI]
  end

  subgraph Export
    OUT[MP4 / PNG / PDF ...]
  end

  TAB --> PJ
  PJ --> A
  PJ --> R
  PJ --> D
  PJ -->|export graph| OUT
  A -.->|native until encode| D
```

**Mods** serialize their node types; the **host** owns the package layout and asset index ([ADR 0015](adr/0015-native-project-format.md)). Standard extension: **`.unibrow`** (directory bundle or ZIP single file).

## Layer diagram

```mermaid
flowchart TB
  subgraph App["unibrowser-desktop / harness"]
    UI[Stable chrome: tabs / omnibox / Command Manager]
    Win[Window + input]
  end

  subgraph Engine["unibrowser-engine"]
    CMD[Command registry]
    CM[Command Manager state]
    EN[Enablement engine]
    MODE[Mode + CommandContext]
    DOC[Document session]
    SEL[Selection snapshot]
  end

  subgraph Host["unibrowser-mod-host"]
    LOAD[Manifest + dependency resolver]
    REG[Handler registration]
  end

  subgraph Mods["Mods (base + community)"]
    T[base-text]
    P[base-png]
    X[base-web]
  end

  Win --> UI
  UI --> CMD
  UI --> CM
  CM --> CMD
  CMD --> EN
  EN --> SEL
  EN --> DOC
  LOAD --> REG
  REG --> CMD
  T --> REG
  P --> REG
  X --> REG
  DOC --> T
  DOC --> P
  DOC --> X
```

## Context switching vs document surface

```mermaid
flowchart LR
  subgraph StableChrome["Stable chrome"]
    TABS[Tab strip]
    OMN[Omnibox]
    CM[Command Manager frame]
    PAL[Palette]
  end

  subgraph Swaps["Per context"]
    CMT[Manager tab + slots]
    SURF[Document surface]
  end

  DOC[Active document] --> MODE[CommandContext]
  MODE --> CM
  CM --> CMT
  DOC --> SURF
  TABS --> DOC
```

Benchmark mapping: SOLIDWORKS Command Manager (tabs + grey-out), GIMP/Resolve (dock/float toolbars), Affinity/Resolve (stable shell), Blender (`poll` = enablement).

## Data flow: executing a command

```mermaid
sequenceDiagram
  participant User
  participant UI as App UI
  participant Reg as Command registry
  participant En as Enablement
  participant Disp as Dispatcher
  participant Mod as Active backend mod

  User->>UI: Click Increase font size
  UI->>Reg: execute(font.size.increase)
  Reg->>En: enabled?(context)
  En-->>Reg: true
  Reg->>Disp: resolve handler for active backend
  Disp->>Mod: handler.execute(ctx)
  Mod-->>UI: selection / document updated
  UI->>En: refresh grey-out state
```

## Navigation (browser shell)

```mermaid
flowchart LR
  subgraph Input
    O[Omnibox / path bar]
    FB[Files mod listing]
    DL[Download complete]
  end

  subgraph Host
    NAV[Location resolver]
    TABS[Tab session]
  end

  subgraph Tab
    L["BrowserLocation\nfile / https / unibrowser://"]
    BE[Backend mod]
  end

  O --> NAV
  FB --> NAV
  DL -->|default OpenInNewTab| NAV
  NAV --> TABS
  TABS --> L
  L --> BE
```

| Concept | Responsibility |
| --- | --- |
| **`BrowserLocation`** | `File`, `Http`, or `Internal` — every tab has one (`unibrowser-mod-api::navigation`). |
| **Tab session** | Ordered tabs, active tab, dirty flags; location + backend state per tab. |
| **Download disposition** | Default **open in new tab**; save to disk only when the user asks. |
| **Files mod** | Directory UI; open children as new tabs via the same resolver as global “Open file”. |

## Core concepts

| Concept | Responsibility |
| --- | --- |
| **Command** | Stable ID, title, icon key, default keybinding; registered once in the host. |
| **Handler** | Mod-provided implementation + enablement predicate (Blender `poll` analogue). |
| **UiCommandSlot** | Command id placed on a manager tab or floating bar ([ADR 0010](adr/0010-ui-commands-and-command-manager.md)). |
| **Backend** | Opens a document type; owns **native** model; maps selection to `SelectionSnapshot`. |
| **`CommandContext`** | Text / Office / Code / Image / Audio / Browse / Web — drives auto manager tab. |
| **Preview vs export** | Mod defines native state; conversion edges at preview/export ([ADR 0011](adr/0011-lazy-conversion-native-until-encode.md)). |
| **Mod package** | `info.json` metadata + **compiled** payload (DLL/WASM); see [mod-system.md](mod-system.md). |

## Crate layout

| Crate | Role |
| --- | --- |
| `unibrowser-mod-api` | Stable types; `CommandContext`, navigation, selection. |
| `unibrowser-engine` | Registry, Command Manager, enablement, palette, layouts. |
| `unibrowser-mod-host` | Load manifests, resolve deps, register bundled/dynamic mods. |
| `unibrowser-app` | egui harness. |
| `unibrowser-desktop` | Tauri product shell. |
| `mods/*` | Bundled base mods (same API as future community mods). |

## Capability model (narrow, not universal DOM)

The host does not expose SVG DOM or OOXML to commands. Mods implement **capability interfaces** (for example `FontSizeCapable`) and publish what the current selection supports via `SelectionSnapshot` and context keys.

## Persistence

- **toolbars.json** — Command Manager tabs, floating bars, `context_pin` ([ui-commands.md](ui-commands.md)).
- **keybindings.json** — chords ([keybindings.md](keybindings.md)).
- Document content: per-backend save; host tracks dirty flag and path.

## Related ADRs

- [0001 Shell and rendering](adr/0001-shell-and-rendering.md)
- [0002 Mod system and sandboxing](adr/0002-mod-system-and-sandboxing.md)
- [0003 Command, handler, enablement](adr/0003-command-handler-enablement.md)
- [0004 Command palette and keybindings](adr/0004-s-key-command-palette.md)
- [0005 Context modes and toolbars](adr/0005-context-modes-and-toolbars.md)
- [0010 UI commands and Command Manager](adr/0010-ui-commands-and-command-manager.md)
- [0011 Lazy conversion / native until encode](adr/0011-lazy-conversion-native-until-encode.md)
- [0012 Context switching UX benchmarks](adr/0012-context-switching-ux-benchmarks.md)
- [0013 Dual audience (FOSS and everyday users)](adr/0013-dual-audience-foss-and-everyday-users.md)
- [0014 VSIX compatibility layer](adr/0014-vsix-compatibility-layer.md)
- [0015 Native project format](adr/0015-native-project-format.md)
- [0016 FOSS UX benchmarks](adr/0016-foss-ux-benchmarks.md)
- [Mod system overview](mod-system.md) (native + VSIX lanes)
- [0006 Mod manifest and versioning](adr/0006-mod-manifest-and-versioning.md)
- [0007 Licensing](adr/0007-licensing.md)
- [0008 API gaps and upstream requests](adr/0008-api-gap-upstream-requests.md)
- [0009 Browser as unified shell](adr/0009-browser-as-unified-shell.md)

## Roadmap

See [roadmap.md](roadmap.md).
