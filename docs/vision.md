# Unibrowser vision

**Unibrowser** is a real **browser** in two senses at once:

1. **File browser** — walk folders and open anything you find (single files or batches) in tabs, view and edit with one UI.
2. **Web browser** — use the live web in the same shell, the way Chrome OS treated the browser as the main surface.

You should not need a different app per format or per “is this local or online?”. A download or an “open” action **opens a new tab in Unibrowser** by default, not a third-party viewer. Tweaking a PDF, a spreadsheet export, a page, and a folder of assets should feel like the same product.

On top of that shell, the **same commands** work everywhere they make sense (for example **Increase font size** on text in HTML, SVG, DOCX, EPUB, PNG text layers, or a live page via user styles). Each format mod implements the command; the toolbar does not move.

## Problems we are solving

- **UI inconsistency**: Every application reinvents toolbars, shortcuts, and command discovery.
- **Format silos**: PNG, PDF, and spreadsheets in three apps means three undo models and three export flows.
- **File vs web split**: OS defaults and “open with” train users to leave the flow; Unibrowser keeps you in one tabbed environment.
- **Extension fatigue**: Monolithic suites are hard to maintain; tiny single-purpose apps do not share behaviour.

## Design north stars

### Chrome OS–style unified shell

- **Tabs are universal**: `file:///…`, `https://…`, and internal pages share one strip and one command layer.
- **Downloads → new tab** (default): saving to disk is explicit; opening for work is in-app.
- **File browsing is in-product**: directories are a mod surface, not Explorer/Finder with a separate Unibrowser launch.

### SOLIDWORKS CommandManager

- **Context modes** map to the active document and selection; toolbar tabs auto-switch.
- Commands that do not apply are **greyed out, never hidden**.
- **Command palette** on standard chords (default **Ctrl+Shift+P** / **Cmd+Shift+P**); **user-configurable keybindings** like VS Code; toolbars customisable and persisted.

### Factorio-style mods

- Engine ships fundamentals: **navigation**, tabs, commands, modes, mod loader.
- **Formats and web** are mods (`base-text`, `base-web`, …) on the same API as community mods.
- Manifests, dependencies, API versioning, and upstream API requests when mods outgrow the host.

### VS Code-style core

- Small Rust engine; bite-sized mods; no Chromium/Firefox **fork** for v1 (embed via WebView/CEF path per ADR 0001).

## Non-goals (initially)

- macOS and mobile targets.
- Every obscure format on day one (plugins and honest “unsupported” tabs).
- Forking Chromium, Gecko, or LibreOffice as the whole codebase.
- GPL/AGPL engines linked into the permissive core.

## Success picture

> You download a `.docx` from a site; it opens in a new tab. You open a project folder; images and README open in tabs with the same Increase font size control where text is selected. A PNG without a text layer keeps that control visible but greyed out.

When that feels SOLIDWORKS-grade in a browser-shaped shell, we expand mods.

## Name

**Unibrowser** — the name matches the product: one browser for files and the web. Prior uses and trademarks still need a formal check (AU/NZ/US) before a public launch.
