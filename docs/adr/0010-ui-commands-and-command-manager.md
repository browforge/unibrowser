# ADR 0010: Reusable UI commands and Command Manager

- **Status:** Accepted
- **Date:** 2026-10-05

## Context

Font size was an example of a **cross-format command** (text, office, code, images, audio, web). The product needs:

1. One **command registry** (execute + enablement + palette).
2. **Reusable UI commands** — the same command id rendered on Command Manager tabs, floating toolbars, palette, and (later) context menus.
3. A **Command Manager** (SOLIDWORKS-style) with context tabs and **user-draggable** toolbars persisted to disk.
4. A **rollout order** for contexts: **Text → Office → Code → Image → Audio**.

## Decision

### Layers

| Layer | Responsibility |
|-------|----------------|
| `CommandRegistry` | Stable ids, handlers, enablement, palette search |
| `UiCommandSlot` | Reference to a command id in a layout |
| `CommandManagerState` | Tabs, floating bars, selected tab, auto context |
| Shell UI | Renders slots, grey-out when disabled, drag-reorder, persist |

### Command contexts

`CommandContext` (`unibrowser-mod-api::context`) drives default tab selection:

- **Text** — plain text, markdown files (non-code extensions)
- **Office** — doc/xls/ppt family (handlers planned)
- **Code** — source extensions (`.rs`, `.ts`, …)
- **Image** — raster/vector backends
- **Audio** — planned
- **Browse** / **Web** — shell navigation

Auto tab switch follows active document; user can select another tab; layout saved in `toolbars.json`.

### Surfaces

`CommandSurface`: `command_manager_tab`, `floating_toolbar`, `command_palette`, `context_menu` (palette wired today; context menu later).

### Persistence

`~/.config/unibrowser/toolbars.json` (see [ui-commands.md](../ui-commands.md)) mirrors ADR 0005 JSON shape, extended with `floating_bars` and `phase` (`active` | `planned`).

### Rollout

| Context | Phase 1 | Next |
|---------|---------|------|
| Text | Save, font size, Monaco | — |
| Office | Tab visible, `planned` | DOCX mod |
| Code | Reuse text commands; separate tab | LSP, format |
| Image | Font on text layers | Raster tools |
| Audio | Tab visible, `planned` | ffmpeg sidecar |

Mods register **commands + handlers**; they do not own toolbar widgets. Manifest `toolbar_suggestions` may be added later (ADR 0006).

## Consequences

- Tauri shell uses `get_command_manager` instead of ad hoc toolbar lists.
- egui harness can migrate to the same `CommandManagerFile` default.
- Office/audio tabs may be empty but remain visible (planned styling) so the manager layout stays stable.
