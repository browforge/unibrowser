# New project and templates research

Survey of how creative and knowledge tools start work: empty defaults versus format pickers, template galleries, and where export fits. **Audience:** contributors implementing File → New, the New tab page, and hub templates ([ADR 0036](../adr/0036-new-project-empty-by-default-and-templates.md), [new-project-flow.md](../design/new-project-flow.md)).

**Method:** public help pages, extension docs, and vendor galleries (October 2026). Unibrowser’s seed decision (Bear, 2026-10): **New = empty Unibrowser project**; templates are optional starting points; **docx/pptx/png are export/save**, not creation types.

## Seed decision (Unibrowser)

| Layer | Rule |
| --- | --- |
| **New** | Opens an **empty** `.unibrow` session in memory. Nothing is written until the first edit or explicit Save, per [ADR 0032](../adr/0032-unibrow-container-archive-and-folder-modes.md) consent. |
| **New from template** | Applies a **starting graph** (and optional assets) from built-in starters, user-saved templates, or hub-installed template packs. |
| **Open file** | Unchanged: double-click or Open still opens foreign files in place. |
| **Export** | Delivery formats (Office, PDF, PNG, MP4, …) are **Save As / Export** projections ([ADR 0035](../adr/0035-one-document-model-wysiwyg.md)), not New wizard steps. |

Content and intent drive creation; file extension is a downstream choice.

## Comparison table

| Product | Default “new” | Template / starter path | Format at creation? | Export vs create | Sources |
| --- | --- | --- | --- | --- | --- |
| **VS Code** | **New File** (untitled) and **New Window** (empty window); optional **profile** | **File → New File…** picker lists contributed file types ([new file menu](https://code.visualstudio.com/api/extension-guides/custom-editors#new-file-menu)); **workspace** = folder + optional `.code-workspace` ([multi-root](https://code.visualstudio.com/docs/editor/workspaces)); **dev containers** from templates ([dev container templates](https://containers.dev/templates)); **Yeoman** / `yo code` scaffolds extensions ([your first extension](https://code.visualstudio.com/api/get-started/your-first-extension)); **Walkthroughs** in Get Started ([walkthroughs](https://code.visualstudio.com/api/ux-guidelines/walkthroughs)); extension **template galleries** in marketplace | Picker can create `.md`, `.ts`, etc., but core metaphor is **untitled buffer** or **open folder** | Save chooses path/extension; no “new Word doc” in core | [Custom editors](https://code.visualstudio.com/api/extension-guides/custom-editors), [Workspaces](https://code.visualstudio.com/docs/editor/workspaces), [Profiles](https://code.visualstudio.com/docs/editor/profiles) |
| **JetBrains IDEs** | **New Project** wizard default varies by IDE; often empty module | **New Project** dialog: language, build system, sample code ([create project](https://www.jetbrains.com/help/idea/create-new-project.html)); **project templates** from plugins; **Import** for existing trees | Yes — project **type** (Maven, Gradle, …) at creation | Artifact export is build/deploy, not the same wizard | [New project](https://www.jetbrains.com/help/idea/new-project-wizard.html) |
| **Microsoft Office** | **Blank** document/presentation/workbook is first tile | **File → New** opens **template gallery** (Microsoft 365 + personal); search and categories ([create from template](https://support.microsoft.com/en-us/office/create-a-document-from-a-template-3f2b2e96-6b3c-4c56-9d9a-0c0e8c8b0e0a)); online gallery at [templates.office.com](https://templates.office.com/) | **Strong:** user picks Word vs PowerPoint vs Excel up front | Save As changes format; “New” is format-specific app | [Word templates](https://support.microsoft.com/en-us/word/create-and-use-word-templates), [PowerPoint templates](https://support.microsoft.com/en-us/powerpoint/create-and-save-a-powerpoint-template) |
| **Figma** | **New design file** (empty canvas) | **Community** and team **templates**; duplicate to start ([templates](https://help.figma.com/hc/en-us/articles/360041003114-Use-templates)); **FigJam** separate empty board | File type (Design vs FigJam) is a product split, not export | Export PNG/PDF/SVG is separate from New | [Figma templates](https://www.figma.com/community/templates) |
| **Canva** | **Create a design** → pick **use case** (poster, video, …) | Large **template gallery** first; blank sizes under custom dimension | **Use case ≈ format** at start | Download/share picks file type | [Canva templates](https://www.canva.com/templates/) |
| **Notion** | **Empty page** in workspace | **Template gallery** (Notion + community); **Templates** button in sidebar ([templates](https://www.notion.com/help/guides/using-notion-templates)) | Block doc model; no “new .docx” | Export Markdown/PDF/HTML later | [Add templates](https://www.notion.com/help/add-and-manage-templates) |
| **Obsidian** | **New note** in vault (empty or daily) | **Templates** core plugin: insert boilerplate into a note ([templates](https://help.obsidian.md/Plugins/Templates)); community **vault** templates | `.md` implied by vault | Export via plugins | [Core plugins](https://help.obsidian.md/Plugins) |
| **DaVinci Resolve** | **New Project** → name + location | **Project presets** (timeline format, colour science); **Project Manager** ([project manager](https://documents.blackmagicdesign.com/UserManuals/DaVinci-Resolve-18-User-Manual.pdf)) | Timeline resolution at project create | Deliver page for export codecs | [Resolve manual](https://www.blackmagicdesign.com/products/davinciresolve) |
| **Blender** | **General** new file from **startup file** | **App templates** (VFX, video editing, …) swap defaults ([app templates](https://docs.blender.org/manual/en/latest/getting_started/app_templates.html)); user can **Save Startup File** | `.blend` only at create | Export glTF/FBX/OBJ via File → Export | [Startup scene](https://docs.blender.org/manual/en/latest/files/blend/open_save.html#save-startup-file) |
| **Adobe Creative Cloud** | App-specific blank canvas | **Libraries** and **Stock** templates; UXP plugins may add starters | PSD/AI/AE format = app choice | Export/Save As delivery | [Creative Cloud templates](https://helpx.adobe.com/creative-cloud/help/using-template-libraries.html) |

## Patterns

### 1. Developer tools favour empty + folder

VS Code’s centre of gravity is **untitled editor** or **folder workspace**, with contributed “New File…” types as shortcuts—not a modal “pick your final file format.” JetBrains is the opposite for app projects: the wizard encodes build tooling early. Unibrowser aligns with **VS Code + Resolve**: one **project container** (`.unibrow`), optional template, folder or archive when the user saves.

### 2. Office and Canva train “format first”

Office’s New screen is a **marketing + template** surface tied to Word/PowerPoint/Excel. That helps sell SKUs but fights a **unified multimodal** shell. Unibrowser deliberately avoids “New PNG” / “New Word” as primary actions; those remain **export profiles** once content exists.

### 3. Template galleries are browse + search, not wizards

Figma, Notion, and Office use **grid + search + categories**, often inline rather than a blocking multi-step wizard. Obsidian inverts: empty note first, template **inserts** on command. Unibrowser should offer **gallery + search** for “New from template” and keep **New** one click to empty project.

### 4. Templates carry dependencies

VS Code devcontainer templates declare **features** and **extensions** ([devcontainer.json](https://containers.dev/implementors/json_reference/)). Unibrowser template manifests should declare **required plugins** with install prompts ([ADR 0031](../adr/0031-edit-history-as-file-format.md) pin header, [ADR 0029](../adr/0029-plugin-permissions-and-network-security.md)).

### 5. Pinning and lockfiles

pnpm/npm **lockfiles** pin exact dependency versions for reproducibility. Unibrowser already pins plugin versions in `.unibrow-history` ([ADR 0031](../adr/0031-edit-history-as-file-format.md)). Template packs from the hub should pin like **mods in the lockfile** ([ADR 0031](../adr/0031-edit-history-as-file-format.md) §3, hub parity with [ADR 0006](../adr/0006-mod-manifest-and-versioning.md)).

### 6. Privacy: templates are data, not code—until they ask

Chrome extension templates that run code need permissions; Figma community files are data-only. Unibrowser templates must not execute plugin logic on apply without the same **capability + consent** path as installing a mod ([ADR 0029](../adr/0029-plugin-permissions-and-network-security.md)).

### 7. Walkthroughs vs empty canvas

VS Code **Get Started** walkthroughs teach the shell without choosing a file type ([walkthroughs UX](https://code.visualstudio.com/api/ux-guidelines/walkthroughs)). Unibrowser’s optional **60-second tour** ([workflows.md](../design/workflows.md)) should remain separate from New/template.

## Lessons for Unibrowser

1. **Primary action:** **New** → empty in-memory `.unibrow` project tab; tab title “Untitled project” until Save.
2. **Secondary:** **New from template** opens gallery (built-in, user, hub); never blocks New.
3. **File menu:** mirror VS Code — **New**, **New from template…**, **Open File…**, **Open Folder…**; no “New Word Document.”
4. **Save / Export:** first Save asks path and archive vs folder ([ADR 0032](../adr/0032-unibrow-container-archive-and-folder-modes.md)); Export menu lists delivery formats ([ADR 0035](../adr/0035-one-document-model-wysiwyg.md)).
5. **Hub:** template packs are **installable content** beside mods; pinned versions in project history when applied.
6. **User templates:** **Save as template…** on any saved project; stored under user config, shareable as `.ubtemplate` (see ADR 0036).

## Related docs

- [ADR 0036: New project empty by default and templates](../adr/0036-new-project-empty-by-default-and-templates.md)
- [new-project-flow.md](../design/new-project-flow.md)
- [ADR 0032: `.unibrow` container](../adr/0032-unibrow-container-archive-and-folder-modes.md)
- [workflows.md](../design/workflows.md) — journey 5 (update to match ADR 0036)
- [ui-implementation-plan.md](../design/ui-implementation-plan.md) — NEW-* tickets
