# Unibrowser vision

Unibrowser is an open source desktop host for viewing and editing many file formats through **one consistent user interface**. The same command (for example **Increase font size**) appears in the same place whether you are editing plain text, SVG, HTML, a DOCX paragraph, an EPUB chapter, a DXF annotation, or a live web page (via injected user styles). Each format implements the command differently; the user learns the tool once.

## Problems we are solving

- **UI inconsistency**: Every application reinvents toolbars, shortcuts, and command discovery. Power users carry dozens of mental models.
- **Format silos**: Opening a PNG, a PDF, and a spreadsheet means three apps with three undo models and three export flows.
- **Extension fatigue**: Monolithic suites are hard to maintain; tiny single-purpose apps do not share behaviour.

## Design north stars

### SOLIDWORKS CommandManager

- **Context modes** (sketch, model, assembly, drawing, …) map to Unibrowser **modes** driven by the active document and selection.
- Switching mode **selects the relevant toolbar tab** automatically.
- Commands that do not apply are **greyed out, never hidden**, so layout stays stable and tools do not “move”.
- **S-key** opens a fuzzy command palette (like SOLIDWORKS search or VS Code Ctrl+Shift+P).
- Toolbars and command tabs are **user-customisable** and persisted.

### Factorio-style mods

- The **engine** ships only fundamentals: document session, selection bus, command registry, modes, toolbars, and a mod loader.
- **Formats are mods** (`base` bundled mods for text, PNG, HTML, … use the same API as community mods).
- Mods declare **dependencies and versions** in a manifest (`info.json`).
- When a mod needs capability the engine lacks, authors file an **upstream API request**; the host API is **versioned** with explicit negotiation.

### VS Code-style core

- Small, solid **Rust engine**; bite-sized **mods** for formats and workflows.
- Contribution points (commands, handlers, backends, toolbars) rather than a single mega-codebase.
- No Chromium or Firefox fork for v1; embed web tech where needed via mods.

## Non-goals (initially)

- macOS and mobile targets.
- “Every format on day one” (OpenDoc lesson: ship a compelling host and a sharp vertical first).
- Forking Chromium, Gecko, or LibreOffice as the application shell.
- GPL/AGPL engines linked into the permissive core (those stay optional, labelled mods or sidecars).

## Success picture

> Select text in HTML and in SVG; press the same toolbar button or S-key command; both increase font size. With a PNG open and no text selection, the button stays visible but greyed out.

When that feels SOLIDWORKS-grade, we add formats as mods.

## Name note

“Unibrowser” has prior uses (university search, assistive UI, npm helpers). Before public launch, run trademark checks (AU/NZ/US) and consider a distinctive product name if “browser” confuses users (Unibrowser is not primarily a web browser).
