# ADR 0033: Web target (static browser demo)

- **Status:** Accepted
- **Date:** 2026-10-12

## Context

Unibrowser ships today as a **Tauri 2** desktop app: the shell UI is web tech, but file I/O, process spawning, native mods, WebView-hosted `https://` tabs, and OS integration live in the Rust host. Contributors and users asked for a **public demo** that proves the same shell, Command Manager, and text/markdown editing run **entirely in the browser**, with **no upload** of opened files.

The engine crates (`unibrowser-engine`, format logic in bundled mods, capability registry, command resolution) must remain **free of Tauri types**. Desktop and web are two **platform implementations** behind one host-facing contract (file access, dialogs, windowing, process).

## Decision

### 1. Platform interface

Introduce `unibrowser-platform` traits (filesystem, dialogs, windowing, process) implemented by:

| Surface | Implementation |
| --- | --- |
| Desktop | Tauri commands in `unibrowser-desktop` (unchanged default product) |
| Web | TypeScript adapters (File System Access API, download save, `localStorage` / IndexedDB drafts) plus `unibrowser-web-host` compiled to **wasm32** via wasm-bindgen |

The UI calls a single `invoke(cmd, args)` bridge. On desktop it forwards to `@tauri-apps/api/core`; on web it dispatches to WASM plus browser-specific handlers.

### 2. Web architecture

- **Static hosting only:** Vite `build:web` emits assets for GitHub Pages. No Unibrowser backend; no telemetry upload of file contents.
- **WASM:** `unibrowser-engine` (commands, Command Manager layout, capability resolution, tab model, text backends for bundled mods registered in the web host) in `unibrowser-web-host`. **No Rust threads** in v1 WASM (avoids COOP/COEP on Pages); palette and CM stay on the main thread.
- **Front end:** Same `crates/unibrowser-desktop` sources with Vite aliases for Tauri shims in `web` mode.
- **Plugins:** WASM mods (ADR 0019) may run when bundled; **native DLL / cdylib mods cannot**. VSIX JS host, Rhai, sidecars, and OS WebView features are **unsupported** on web with explicit grey-out reasons.
- **Permissions:** Map to the browser sandbox (user gesture for file picker; optional persistent handles via File System Access API on Chromium; elsewhere open via picker and save via download).

### 3. File access on web

| Capability | Chromium (FS Access API) | Other browsers |
| --- | --- | --- |
| Open file | `showOpenFilePicker` when available; else `<input type="file">` | File picker only |
| Open folder | `showDirectoryPicker` when available | Not supported (greyed) |
| Save | `createWritable()` on existing handle; else download blob | Download |
| Drafts / prefs | `localStorage` + IndexedDB for unsaved buffers when no handle | Same |

Bundled **demo pack** ships as static files under `/demo-pack/` (from `examples/demo-pack`).

### 4. Offline / PWA

Web build includes a **web app manifest** and **service worker** that caches shell assets (not user files). Offline use is best-effort: demo pack and last-opened drafts may be available; `https://` tabs and network mods remain unavailable offline.

### 5. First web release scope (v1)

**In scope:**

- Open bundled demo pack entries and user-picked local text/markdown (and plain text save back).
- Shell chrome, Command Manager, themes, settings pages that do not require desktop host.
- Calm in-product notice: runs in browser; files never leave the device; link to desktop downloads.

**Out of scope (greyed with reason, never broken):**

- Native mods, VSIX install, terminal, LSP, ffmpeg sidecar, multi-window, `https://` web tabs in embedded WebView, PDF/DOCX heavy viewers unless already pure-JS paths, AI keys to cloud without user paste, OS file associations.

### 6. CI / Pages

`.github/workflows/pages.yml` builds WASM + `npm run build:web` on push to `main`, deploys via official `configure-pages` / `upload-pages-artifact` / `deploy-pages`. Repository **Settings → Pages → Source: GitHub Actions** must be enabled by the owner once.

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Files stay on device; no silent upload |
| Never blocks save or work | Save via download or FS API; drafts in browser storage |
| Local-first | No account required for web demo |
| Predictable and consistent | Same CM/command IDs; capabilities explain web gaps |
| Simple durable mechanism | Static WASM + Vite, not a hosted Unibrowser server |
| Mod not core | Web host registers bundled text/markdown mods only in v1 |

## Consequences

- Desktop Tauri build and tests remain the primary release path; web is a **demo / try-it** surface.
- Some Tauri commands gain web stubs or WASM implementations; desktop code paths unchanged.
- Future work: more mods in WASM, optional COOP/COEP + threads if a format needs it, richer FS Access folder workspace.

## References

- ADR [0001](0001-shell-and-rendering.md), [0009](0009-browser-as-unified-shell.md), [0019](0019-wasm-t2-sandbox.md), [0028](0028-platform-content-kinds-and-plugin-scope.md), [0029](0029-plugin-permissions-and-network-security.md)
- `crates/unibrowser-platform`, `crates/unibrowser-web-host`, `crates/unibrowser-desktop` web build
