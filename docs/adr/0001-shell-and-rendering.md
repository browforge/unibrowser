# ADR 0001: Shell and rendering

- **Status:** Accepted (product direction); skeleton uses alternate shell
- **Date:** 2026-10-05

## Context

Unibrowser needs a desktop shell on **Windows (primary)** and **Linux** that can host both native Rust views and web-based editors (Monaco, ProseMirror, pdf.js). Bear considered porting a browser; the research brief recommends **embedding** over forking Chromium or Gecko.

Alternatives evaluated:

| Option | Pros | Cons |
| --- | --- | --- |
| **Tauri 2 + system WebView** | Small binary; Rust host; Apache/MIT; native bridges | WebView variance on Linux; HTML mod needs WebView anyway |
| **Electron** | Identical Chromium; fastest VS Code-like ecosystem | Large footprint; Node security surface |
| **CEF** | Full Chromium embedding API | Heavy; C++ integration cost |
| **Native Rust UI (egui, iced, Slint, Xilem/Vello)** | Pure Rust skeleton; fast iteration on CommandManager | Rich HTML/DOCX editors are not native widgets |
| **Browser fork (Chromium/Gecko)** | Maximum web compatibility | Permanent rebase tax; ruled out for v1 |
| **Servo / Ladybird** | Rust / independent engine | Embedding APIs immature; not MVP |

## Decision

1. **Product shell (post-skeleton):** **Tauri 2** with OS WebView (WebView2 on Windows, WebKitGTK on Linux). Unibrowser **is** a browser ([ADR 0009](0009-browser-as-unified-shell.md)): file and `https://` tabs share one chrome; format editors and live web run as **mods** inside that shell.
2. **This repository’s skeleton:** **eframe + egui** for the CommandManager, grey-out toolbar, and command palette (default **Ctrl+Shift+P**) so the Rust engine and mod API are proven without Node/WebView setup. Document surfaces in the skeleton are placeholders; real rendering migrates to Tauri webviews per mod.
3. **Do not fork** Chromium or Gecko for Unibrowser v1.

## Consequences

- CI must build Tauri on Windows/Linux in Phase 1; skeleton CI stays pure Rust.
- Linux QA must track WebKitGTK behaviour for HTML mod.
- Optional future: Tauri experimental CEF path if OS WebViews diverge too far.

## References

- Research brief §1 (Tauri recommended MVP shell)
- [Tauri](https://tauri.app/)
