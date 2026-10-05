# ADR 0009: Browser as unified shell (files and web)

- **Status:** Accepted
- **Date:** 2026-10-05
- **Supersedes:** The “name note” in [vision.md](../vision.md) that downplayed the web-browser role.

## Context

Bear’s product intent: **Unibrowser is a real browser in two senses**:

1. **File browser** — navigate folders, open any file or selection of files in tabs, tweak them with the same commands and chrome.
2. **Web browser** — browse the live web in the same shell (Chrome OS mental model: the browser is the computer surface, not a side app).

Downloads and “open file” actions should **open a new tab inside Unibrowser**, not hand off to a third-party default application. Consistency beats OS file associations for day-to-day work.

Phase 0 proved commands, mods, and toolbars in a desktop window without navigation. Phase 1+ must treat **tabs + locations** as first-class, alongside document backends.

## Decision

1. **Primary metaphor:** one **tab strip** for every location: `file://`, `https://`, and internal `unibrowser://` pages (settings, mod manager, about).
2. **No fork** of Chromium/Gecko for v1 (unchanged from ADR 0001), but the **product is a browser**: omnibox/address bar, back/forward where applicable, bookmarks later, download pipeline that **always offers “Open in new tab”** as default (save to disk is explicit).
3. **File browsing** is not a separate app: a **Files** mod (bundled) contributes directory listing UI and “open in tab” for children; double-click / Enter opens via the same mod routing as “Open file”.
4. **Web** is a bundled mod (`base-web` or `base-html-live`) on the system WebView (Tauri), sharing the host command layer with file-backed tabs (user styles, font size on readable content, etc.).
5. **Mod routing:** host resolves `BrowserLocation` → MIME/sniffer → backend mod; unknown types get an honest tab (“no mod yet”) with optional “install mod” / open externally as last resort.

## Alternatives considered

| Approach | Why not (for Unibrowser) |
| --- | --- |
| File manager + separate web browser | Breaks Chrome OS consistency; two UIs to learn |
| Fork Chromium as the whole app | Fork tax; host/mod story harder |
| egui-only shell long term | Poor fit for real web + rich file previews |

## Consequences

- ADR 0001 stands: **Tauri + WebView** is the right shell for the *browser* product; egui skeleton remains a command-framework spike only.
- Engine gains **navigation types** (`BrowserLocation`, tab session) ahead of full UI.
- Roadmap Phase 1 prioritises: tab model, file tree, web tab, download → new tab.
- Security: web tabs need normal browser hygiene (CSP, permission prompts, no arbitrary `file://` from untrusted origins without policy).

## References

- Owner direction (2026-10-05): Chrome OS–style unified browser; downloads open in tabs.
- ADR [0001](0001-shell-and-rendering.md), [0006](0006-mod-manifest-and-versioning.md)
