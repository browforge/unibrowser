# Architecture overview

Unibrowser is a **unified browser shell** (files + web in one tab strip; see [ADR 0009](adr/0009-browser-as-unified-shell.md)). The **host engine** owns navigation, commands, modes, UI chrome, and mod loading; **mods** supply document backends, web surfaces, and the bundled file browser. The skeleton uses **egui/eframe** only to prove commands and toolbars; the product shell is **Tauri + WebView** ([ADR 0001](adr/0001-shell-and-rendering.md)).

**Product emphasis:** world-class **context switching** (stable chrome, auto Command Manager tab, grey-out, pin/customise), **mixed-format projects** with **lazy conversion** ([ADR 0011](adr/0011-lazy-conversion-native-until-encode.md), [ADR 0012](adr/0012-context-switching-ux-benchmarks.md)), and **dual audience** design — everyday defaults, FOSS depth without user-space interference ([ADR 0013](adr/0013-dual-audience-foss-and-everyday-users.md), [vision.md](vision.md)).

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
| **Mod manifest** | `info.json`: id, version, dependencies, `api_version`, contributions. |

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
- [0006 Mod manifest and versioning](adr/0006-mod-manifest-and-versioning.md)
- [0007 Licensing](adr/0007-licensing.md)
- [0008 API gaps and upstream requests](adr/0008-api-gap-upstream-requests.md)
- [0009 Browser as unified shell](adr/0009-browser-as-unified-shell.md)

## Roadmap

See [roadmap.md](roadmap.md).
