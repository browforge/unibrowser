# New project flow (UX)

Normative interaction design for **New**, **New from template**, and the New tab page. Visual treatment follows the **Quiet Utility** mockup render style (`docs/design/mockups/quiet-utility/`, `render.sh` output): **dark navy shell** framing a **light paper card** work surface. This is the same language as [visual-language.md](visual-language.md) (dark slate frame `#1B1D1E` / `#141617`, warm paper `#FAF9F6` on `#F1F0EC` well).

**Decision record:** [ADR 0036](../adr/0036-new-project-empty-by-default-and-templates.md). **Research:** [new-project-and-templates.md](../research/new-project-and-templates.md).

## Goals

1. **New** means “start thinking in a Unibrowser project,” not “pick a file type.”
2. Templates are **optional acceleration**, browsed like VS Code / Figma galleries—not a blocking wizard.
3. Disk stays clean until the user edits or saves ([ADR 0032](../adr/0032-unibrow-container-archive-and-folder-modes.md)).
4. Export formats never appear on the start card except under an explicit **Export** entry elsewhere.

## Surfaces

| Surface | Job |
| --- | --- |
| **New tab page (NTP)** | Home for empty shell: start work, reopen recents |
| **File menu** | Same verbs as NTP; power-user paths |
| **Command palette** | `New Project`, `New Project from Template`, `Open File`, `Open Folder` |
| **Template gallery** | Full-width overlay or dedicated start sub-page; search + filters |

Remove: duplicate **New project** buttons in the title row, unexplained **Home / Downloads / Settings** strip on NTP ([workflows.md](workflows.md) friction list).

## New tab page layout (1280×800 reference)

Rendered like `new-tab.html` / `new-tab.png`, with these content changes:

```
┌─ Title bar (40px, dark navy) ─────────────────────────────────────────┐
│ [≡]  [ New tab * ] [ + ]     Search, open a path, or URL…     … □ x │
├─ Command Manager (40px, Start context) ───────────────────────────────┤
│ Start │ Open: File · Folder │ Create: New ▾ · Template │ Recent    │
├─ Rail │ ┌─ Paper card (max-width ~720px, centred in well) ─────────┐ │
│ 40px  │ │  Start something new                                      │ │
│       │ │  Open a file, folder, or begin an empty Unibrowser        │ │
│       │ │  project. Nothing is saved until you edit or choose Save. │ │
│       │ │  [ New ]  [ New from template ]  [ Open file ] [ Folder ] │ │
│       │ │                                                         │ │
│       │ │  Recent (when any)                          Clear history│ │
│       │ │  … rows …                                               │ │
│       │ │  Downloads (when active)                    Open shelf │ │
│       │ └─────────────────────────────────────────────────────────┘ │
├─ Status bar (24px) ───────────────────────────────────────────────────┤
│ Unsaved projects stay in memory · Local-first              Online    │
└───────────────────────────────────────────────────────────────────────┘
```

### Paper card rules

- Card sits on **recessed well** (`#E7E5DF` light / `#141617` dark), not floating with heavy shadow.
- **One** primary button: **New** (filled accent).
- **New from template** is secondary (raised control outline).
- Open actions are tertiary text buttons or quiet buttons—equal weight to each other, below create row.
- No format icons (Word, PPT, PNG) on this card.

### Welcome / tour

- Optional one-line welcome under the lead copy; **Take a 60-second tour** / **Dismiss** persist dismissal ([workflows.md](workflows.md) journey 1).
- Tour never blocks **New**.

## New button and dropdown

**New** (primary):

- **Click:** immediate empty project tab (§ Behaviour).
- **Chevron / long-press / Alt+click:** small menu anchored to button:
  - **Empty project** (default, same as click)
  - **Duplicate current project** (when a project tab is focused; future)
  - **New window** (when multi-window enabled)

Command Manager **Create** group label stays **Create**; control title **New** with chevron, not “Project (Video).”

## Template gallery

Opened by **New from template**, File → **New from template…**, or palette.

### Layout (render-1 family)

- Shell remains dark navy; gallery content is a **full-height paper panel** sliding over the well (not a system modal).
- **Header:** search field (focus on open), filter chips **All · Built-in · My templates · Installed**.
- **Body:** responsive grid of template tiles (min 200px): thumbnail, title, one-line description, category pill.
- **Footer:** link **Get more templates** → Mod portal Templates tab (network consent per [plugin-permissions-ui.md](plugin-permissions-ui.md)).

### Tile interaction

| State | UI |
| --- | --- |
| Default | Hover: 1px border on card; no lift shadow |
| Missing plugin | Amber chip **Needs Video mod**; tile still selectable |
| Selected + Enter | Apply template → close gallery → project tab |
| Escape | Close gallery; no new tab |

### Search

- Filters title, description, categories, hub publisher.
- `/` focuses search when gallery open.
- Empty results: “No templates match. Try Built-in or install packs from the portal.”

### Built-in tiles (v1)

Match [ADR 0036](../adr/0036-new-project-empty-by-default-and-templates.md) §9: Office document, Presentation, Simple video, Design space, Notes, plus **Blank** (same as New).

## Recent list

- Same row pattern as `new-tab.html`: icon, name, path, relative time, **kind** label (Project, PDF, Text, …).
- **Project** rows open `.unibrow`, folder projects, or **unsaved** session restore tokens when session restore lists them.
- **Clear history** clears recents only, not open tabs.
- Section hidden when empty (no placeholder rows).

## Keyboard flow

| Key | Context | Action |
| --- | --- | --- |
| `Ctrl+N` / `Cmd+N` | Global | **New** empty project |
| `Ctrl+Shift+N` | Global | **New from template** (open gallery) |
| `Ctrl+O` | Global | Open file |
| `Ctrl+K Ctrl+O` | Global | Open folder (VS Code parity) |
| `Enter` | NTP focused | Activates focused button or opens selected recent |
| `↑` `↓` | NTP | Move among actions / recents |
| `/` | NTP | Focus location/command field |
| `/` | Gallery | Focus template search |
| `Escape` | Gallery | Close gallery |

Palette strings must match menu labels exactly ([ux.md](ux.md)).

## Behaviour: empty project tab

1. Open new tab with `content_kind` project; title **Untitled project**.
2. Command Manager follows **Create** / project surface (timeline/canvas empty states per [ux.md](ux.md) § Empty).
3. Status bar:
   - Before edit: **In memory** · tooltip explains Save will ask for a location.
   - After first edit, still unsaved: **Unsaved** · **Save** enabled ([ADR 0023](../adr/0023-saving-is-sacred.md)).
4. **No files** created under Downloads, temp, or last-opened folder.

## Behaviour: first Save

1. User chooses **Save** or **Save As**.
2. Dialog defaults:
   - Filename `Untitled project.unibrow`
   - Format: **Unibrowser project (.unibrow)** only (not docx).
3. User picks **archive file** or **folder** destination ([ADR 0032](../adr/0032-unibrow-container-archive-and-folder-modes.md)):
   - Archive → work in archive mode after save.
   - Folder → plain folder mode: only `.unibrow-history` after first edit on disk, unless user unpacks/collects later.
4. On success: tab title updates; recents row added; toast optional only if path non-obvious.

## Behaviour: apply template

1. Run empty project seed (same as **New**).
2. Load `project.json` + assets from template pack ([ADR 0036](../adr/0036-new-project-empty-by-default-and-templates.md) §4).
3. If `default_workspace` set, switch layout once (Browse/Write/Create/Live) without changing open tabs.
4. Missing required mod: non-modal banner at top of project content with **Install from portal** and **Learn more**; Save still enabled.
5. Tab title: template `title` until user renames or saves.

## Behaviour: open foreign file (unchanged)

Not part of New flow, but NTP **Open file** must remain one click away. Opening `report.docx` does **not** create a `.unibrow` unless the user converts via explicit command later.

## Export (out of scope for this screen)

Delivery formats live under **File → Export…**, project **Export** in Command Manager, and palette **Export**. Copy must say **Make a copy for sharing** vs **Save project** ([workflows.md](workflows.md) PDF journey). [ADR 0035](../adr/0035-one-document-model-wysiwyg.md) governs projection list.

## Error and edge cases

| Case | UX |
| --- | --- |
| Template apply fails parse | Inline error on tab; offer **New empty project** |
| Hub template removed but still in gallery | Tile shows **Update or remove**; apply uses last installed files if present |
| Offline + missing built-in asset | Should not happen; show diagnostic with **Report** link |
| Low disk on first Save | Standard save error; in-memory work retained |

## Evidence and mockups

When implementing NEW-* tickets, capture:

- NTP light/dark at 1280 with paper card and single **New** primary.
- Gallery with search + one **Needs mod** tile.
- First Save dialog showing `.unibrow` only.

Place PNGs beside existing `docs/screenshots/shell-density/new-tab-*` or new `docs/screenshots/new-project-flow/`.

## Related docs

- [ux.md](ux.md) — NTP policy, empty states, workspaces
- [workflows.md](workflows.md) — end-to-end journeys (update 1 and 5)
- [ui-implementation-plan.md](ui-implementation-plan.md) — NEW-* tickets
- [visual-language.md](visual-language.md) — surfaces and density
