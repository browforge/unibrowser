# Unibrowser vision

**Unibrowser** is a real **browser** in two senses at once:

1. **File browser** — walk folders and open anything you find (single files or batches) in tabs, view and edit with one UI.
2. **Web browser** — use the live web in the same shell, the way Chrome OS treated the browser as the main surface.

You should not need a different app per format or per “is this local or online?”. A download or an “open” action **opens a new tab in Unibrowser** by default, not a third-party viewer. Tweaking a PDF, a spreadsheet export, a page, and a folder of assets should feel like the same product.

On top of that shell, the **same commands** work everywhere they make sense (for example **Increase font size** on text in HTML, SVG, DOCX, EPUB, PNG text layers, or a live page via user styles). Each format mod implements the command; the Command Manager frame does not jump around — only the active tab’s tools and enablement change.

## Dual audience (one product, two promises)

Unibrowser is built for **hackers and grandmas** at the same time — not two editions, one shell with **progressive disclosure**.

### 1. FOSS crowd — free, moddable, stay out of the way

- **Free as in beer, free as in speech** — permissive core (Apache-2.0); community mods and transparent licensing.
- **Factorio-style mods** — engine + loader + API; formats and features extend without forking the host.
- **Do not interfere with user space** — your files stay where you put them; config is plain JSON in your profile; no dark patterns, no required cloud, no telemetry-by-default. You own your data, your mods, and your layouts.
- **Engine stays out of the way** — the host provides navigation, commands, and chrome; it does not fight the user for attention or lock basic workflows behind accounts.

Power tools (palette, keybindings, Command Manager customisation, pin, mod manifests) exist for those who want them; nothing forces everyone through that surface.

### 2. Chrome OS / grandma crowd — it just works

- **So easy anyone can use it** — tabs and an omnibox feel like a browser; open a folder, photo, letter, or website without learning another app.
- **No format homework** — users should not need to know raster vs vector, PNG vs WebP, or MIME types. They open **a photo** or **a document**; Unibrowser picks a sensible backend.
- **Conversion stays under the hood** — lazy conversion and native-until-encode ([ADR 0011](adr/0011-lazy-conversion-native-until-encode.md)): the product handles messy reality; the primary UI does not lecture about codecs.
- **Mods optional for basics** — bundled `base-*` mods ship in the box; installing more mods is for people who want more, not a prerequisite to open common files.

### How both fit together

| Everyday default | Power under the hood |
|------------------|---------------------|
| Open file → tab just works | Same tab is a `BrowserLocation` + mod backend |
| Simple toolbar / auto context | Pin tab, reorder commands, `toolbars.json` |
| Save where you expect | Per-backend save; dirty flags; no opaque vault |
| “Increase font size” when it applies | Same command id across mods; grey-out when not |

Decision record: [ADR 0013](adr/0013-dual-audience-foss-and-everyday-users.md).

## Problems we are solving

- **UI inconsistency**: Every application reinvents toolbars, shortcuts, and command discovery.
- **Format silos**: PNG, PDF, and spreadsheets in three apps means three undo models and three export flows.
- **“Wrong format” rejection**: Video editors refuse PDF or MIDI on a timeline; Unibrowser should **mix unusual formats** and convert only when preview or export requires it ([ADR 0011](adr/0011-lazy-conversion-native-until-encode.md)).
- **File vs web split**: OS defaults and “open with” train users to leave the flow; Unibrowser keeps you in one tabbed environment.
- **Extension fatigue**: Monolithic suites are hard to maintain; tiny single-purpose apps do not share behaviour.

## Design north stars

### Chrome OS–style unified shell

- **Tabs are universal**: `file:///…`, `https://…`, and internal pages share one strip and one command layer.
- **Downloads → new tab** (default): saving to disk is explicit; opening for work is in-app.
- **File browsing is in-product**: directories are a mod surface, not Explorer/Finder with a separate Unibrowser launch.

### Context switching (primary UX goal)

Unibrowser should **excel at moving between contexts** without feeling like a different app:

- **Stable chrome** — tabs, omnibox, Command Manager frame, palette, status (Affinity unified studio, DaVinci Resolve pages, SOLIDWORKS shell).
- **Auto-relevant tools** — Command Manager tab follows document context; user may **pin** a tab ([ADR 0012](adr/0012-context-switching-ux-benchmarks.md)).
- **Grey-out-not-hide** — SOLIDWORKS / Blender `poll()` analogue: commands stay visible, disabled when they do not apply.
- **Customisable toolbars** — SOLIDWORKS Command Manager + GIMP-style dock/float/reorder; target is to **match or beat** both ([ui-commands.md](ui-commands.md)).

**UI benchmarks (study references):**

| Source | Lesson |
|--------|--------|
| [SOLIDWORKS Command Manager](https://help.solidworks.com/2024/english/solidworks/sldworks/c_commandmanager.htm) | Primary toolbar model; context tabs; grey-out; more customisable than Office ribbon |
| [GIMP docks](https://docs.gimp.org/3.0/en/gimp-concepts-docks.html) | Drag-anywhere toolbars |
| [Affinity](https://affinity.serif.com/en-us/whats-new/) | Familiar chrome across personas / unified app |
| [Blender workspaces](https://docs.blender.org/manual/en/latest/interface/window_system/workspaces.html) | Workspaces + operator search; tools do not vanish |
| [DaVinci Resolve](https://documents.blackmagicdesign.com/UserManuals/DaVinci-Resolve-19-User-Manual.pdf) | Page switching with consistent overall shell |

### Lazy conversion / native-until-encode

Keep **native representations** in the project as long as possible:

- **PDF on a timeline** — vector model in project; rasterise at video encode or use display-resolution proxies while scrubbing.
- **MIDI in a project** — MIDI data in project; synth preview on scrub; render to audio only when baking or exporting.

Convert at **preview** or **export** boundaries, not on “Open file”. Details: [ADR 0011](adr/0011-lazy-conversion-native-until-encode.md).

### Factorio-style mods

- Engine ships fundamentals: **navigation**, tabs, commands, modes, mod loader.
- **Formats and web** are mods (`base-text`, `base-web`, …) on the same API as community mods.
- **Bundled mods** cover grandma-path basics; **optional mods** extend the FOSS path without gatekeeping open.
- Manifests, dependencies, API versioning, and upstream API requests when mods outgrow the host.

### VS Code-style core

- Small Rust engine; bite-sized mods; no Chromium/Firefox **fork** for v1 (embed via WebView/CEF path per ADR 0001).

## Non-goals (initially)

- macOS and mobile targets.
- Every obscure format on day one (plugins and honest “unsupported” tabs).
- Forking Chromium, Gecko, or LibreOffice as the whole codebase.
- GPL/AGPL engines linked into the permissive core.
- Building full video/office editors before the command and conversion contracts are documented (see roadmap).

## Success picture

**Grandma:** She opens her email link, then a folder of family photos, then a PDF — same window, same back button, no “wrong app” dialogs. She never hears the word WebP.

**Hacker:** She edits `toolbars.json`, binds Ctrl+Shift+P, ships a mod that adds a format, and knows Unibrowser did not upload her files anywhere.

> You download a `.docx` from a site; it opens in a new tab. You open a project folder; images and README open in tabs with the same Increase font size control where text is selected. A PNG without a text layer keeps that control visible but greyed out. Later: a PDF on a video timeline stays vector in the project until you export.

When context switching feels SOLIDWORKS-grade inside a browser-shaped shell, we expand mods along **Text → Office → Code → Image → Audio** and mixed-format projects.

## Name

**Unibrowser** — the name matches the product: one browser for files and the web. Prior uses and trademarks still need a formal check (AU/NZ/US) before a public launch.
