# New project and templates research

Survey of how creative and knowledge tools start work: empty defaults versus format pickers, template galleries, and where export fits. **Audience:** contributors implementing File → New, the New tab page, and hub templates ([ADR 0036](../adr/0036-new-project-empty-by-default-and-templates.md), [ADR 0037](../adr/0037-plugin-and-template-relationships.md), [new-project-flow.md](../design/new-project-flow.md)).

**Method:** public help pages, extension docs, and vendor galleries (October 2026). Unibrowser’s seed decision (Bear, 2026-10): **New = empty Unibrowser project**; templates are optional starting points; **docx/pptx/png are export/save**, not creation types.

## Seed decision (Unibrowser)

| Layer | Rule |
| --- | --- |
| **New** | Opens an **empty** `.unibrow` session in memory. Nothing is written until the first edit or explicit Save, per [ADR 0032](../adr/0032-unibrow-container-archive-and-folder-modes.md) consent. |
| **New from template** | Applies a **starting graph** (and optional assets) from built-in starters, user-saved templates, or hub-installed template packs. |
| **Open file** | Unchanged: double-click or Open still opens foreign files in place. |
| **Export** | Delivery formats (Office, PDF, PNG, MP4, …) are **Save As / Export** projections ([ADR 0035](../adr/0035-one-document-model-wysiwyg.md)), not New wizard steps. |

Content and intent drive creation; file extension is a downstream choice.

## Primary reference: Visual Studio (Professional)

Unibrowser’s **New from template** gallery and configure step should track **Visual Studio’s Create a new project** experience ([Create a project](https://learn.microsoft.com/en-us/visualstudio/ide/create-new-project)), not the lighter VS Code “new untitled file” model.

### Dialog structure

| VS element | Behaviour | Unibrowser mapping |
| --- | --- | --- |
| **Recent templates** | Pin of last-used templates at top of dialog ([Create a project](https://learn.microsoft.com/en-us/visualstudio/ide/create-new-project)) | **Recent templates** row in gallery (per user, local only) |
| **Search** | Filters template name, description, tags | Same; `/` focuses search in gallery |
| **Language filter** | C#, C++, … dropdown | **Content / domain** filter: Document, Video, Design, Notes, … |
| **Platform filter** | Windows, Android, … | **Workspace** filter: Browse, Write, Create, Live (layout hint, not a mode lock) |
| **Project type filter** | Console, WPF, … | **Template kind**: Blank, Office starter, Timeline, … |
| **Template tags** | Chips on each card (e.g. Android, Machine Learning) | `tags` in `template.json` + hub metadata ([ADR 0036](../adr/0036-new-project-empty-by-default-and-templates.md) §5) |
| **Prerequisites** | Missing SDK/workload called out before Next | `relationships.requires` + consent sheet ([ADR 0037](../adr/0037-plugin-and-template-relationships.md)) |
| **Configure step** | **Project name**, **location**, **solution name** ([Create a project](https://learn.microsoft.com/en-us/visualstudio/ide/create-new-project)) | **Project name**, **save location** (optional preflight for first Save), **Create** / **Cancel** |
| **Extension templates** | VSIX-installed templates appear in same dialog ([Project templates](https://learn.microsoft.com/en-us/visualstudio/ide/how-to-create-project-templates)) | Hub-installed `.ubtemplate` packs in **Installed** filter |

### Template packaging (VS ecosystem)

| Format | Role | Unibrowser analogue |
| --- | --- | --- |
| **`.vstemplate`** | XML manifest + zip payload; `TemplateData` (name, description, tags, `RequiredFrameworkVersion`, …) and `TemplateContent` ([schema](https://learn.microsoft.com/en-us/visualstudio/extensibility/project-template-schema-reference)) | `template.json` + OPC interior |
| **`template.json` (`dotnet new`)** | `shortName`, `tags`, `sourceName`, `preferNameDirectory`, post-actions ([custom templates](https://learn.microsoft.com/en-us/dotnet/core/tools/custom-templates)) | `id`, `tags`, `relationships`, `project.json` seed |
| **VSIX** | Ships templates into VS template registry | Mod portal **Templates** tab |

VS separates **choosing a template** from **choosing delivery format** (build output). Unibrowser separates **choosing a project starter** from **Export** (ADR 0035).

### Secondary note: VS Code

VS Code is useful for **Open Folder**, **profiles**, and extension walkthroughs, but it is **not** the model for Unibrowser’s template gallery:

- **New File** / contributed types ([custom editors](https://code.visualstudio.com/api/extension-guides/custom-editors#new-file-menu))
- **Dev container** template JSON ([dev containers](https://containers.dev/templates))
- **Get Started** walkthroughs ([walkthroughs](https://code.visualstudio.com/api/ux-guidelines/walkthroughs))

Use these for optional tour and folder-open parity only ([new-project-flow.md](../design/new-project-flow.md)).

## Comparison table (other products)

| Product | Default “new” | Template / starter path | Format at creation? | Export vs create | Sources |
| --- | --- | --- | --- | --- | --- |
| **Visual Studio** | **Create a new project** dialog | Recent + search + filters + tags; configure name/location/solution; `.vstemplate` + `dotnet new` + VSIX templates | Project **type**, not export format | Build output / publish separate from template pick | [Create a project](https://learn.microsoft.com/en-us/visualstudio/ide/create-new-project), [Project templates](https://learn.microsoft.com/en-us/visualstudio/ide/how-to-create-project-templates), [dotnet custom templates](https://learn.microsoft.com/en-us/dotnet/core/tools/custom-templates) |
| **JetBrains IDEs** | **New Project** wizard | Language, build system, sample code ([create project](https://www.jetbrains.com/help/idea/create-new-project.html)) | Yes — stack at creation | Build artefact export | [New project wizard](https://www.jetbrains.com/help/idea/new-project-wizard.html) |
| **Microsoft Office** | **Blank** first tile | **File → New** gallery + [templates.office.com](https://templates.office.com/) | Word / PowerPoint / Excel up front | Save As format | [Word templates](https://support.microsoft.com/en-us/word/create-and-use-word-templates) |
| **Figma** | Empty design file | Community templates ([templates](https://help.figma.com/hc/en-us/articles/360041003114-Use-templates)) | Design vs FigJam split | Export PNG/PDF | [Figma community templates](https://www.figma.com/community/templates) |
| **Canva** | Pick use case | Template gallery first | Use case ≈ format | Download | [Canva templates](https://www.canva.com/templates/) |
| **Notion** | Empty page | Template gallery ([templates](https://www.notion.com/help/guides/using-notion-templates)) | Block model | Export later | [Add templates](https://www.notion.com/help/add-and-manage-templates) |
| **Obsidian** | New note | Templates core plugin ([templates](https://help.obsidian.md/Plugins/Templates)) | `.md` in vault | Plugins | [Core plugins](https://help.obsidian.md/Plugins) |
| **DaVinci Resolve** | New Project | Presets in Project Manager | Timeline format at create | Deliver page | [Resolve](https://www.blackmagicdesign.com/products/davinciresolve) |
| **Blender** | Startup file | App templates ([app templates](https://docs.blender.org/manual/en/latest/getting_started/app_templates.html)) | `.blend` | File → Export | [Startup scene](https://docs.blender.org/manual/en/latest/files/blend/open_save.html#save-startup-file) |

## Patterns

### 1. Visual Studio: filter, tag, then configure

The **Create a new project** flow optimises for **finding** the right starter (recents, search, three filters, tags) and only then asks **where it lives** (name + location). Unibrowser keeps **New** as one-click empty project; **New from template** follows this two-step pattern inside the paper gallery ([new-project-flow.md](../design/new-project-flow.md)).

### 2. Office and Canva train “format first”

Office’s New screen ties templates to Word/PowerPoint/Excel. Unibrowser avoids “New PNG” / “New Word” as primary actions; those remain **export profiles**.

### 3. Dependencies are declared, not discovered

VS surfaces missing workloads/SDKs before configure. Factorio and npm formalise dependency kinds ([Factorio info.json](https://wiki.factorio.com/Mod_structure#info.json), [npm dependencies](https://docs.npmjs.com/cli/v10/configuring-npm/package-json#dependencies)). Unibrowser uses [ADR 0037](../adr/0037-plugin-and-template-relationships.md) for plugins **and** templates.

### 4. Pinning and lockfiles

History **pin header** ([ADR 0031](../adr/0031-edit-history-as-file-format.md)) records exact plugin versions after resolution; template choice is recorded separately on first save ([ADR 0036](../adr/0036-new-project-empty-by-default-and-templates.md)).

### 5. Privacy: templates are data, not code—until they ask

Template apply is offline data load; plugins install through portal consent ([ADR 0029](../adr/0029-plugin-permissions-and-network-security.md)).

## Lessons for Unibrowser

1. **Primary action:** **New** → empty in-memory `.unibrow` project tab.
2. **New from template:** Visual Studio–style gallery (recents, search, filters, tags) → relationship consent → **configure** (name, location) → apply.
3. **File menu:** **New**, **New from template…**, **Open file…**, **Open folder…**; no format-specific New entries.
4. **Relationships:** `requires` / `recommends` / `optional` / `incompatible` on mods and templates ([ADR 0037](../adr/0037-plugin-and-template-relationships.md)).
5. **Save / Export:** first Save asks path ([ADR 0032](../adr/0032-unibrow-container-archive-and-folder-modes.md)); Export lists delivery formats ([ADR 0035](../adr/0035-one-document-model-wysiwyg.md)).
6. **Hub:** template packs install like VSIX templates into the same picker.

## Related docs

- [ADR 0036](../adr/0036-new-project-empty-by-default-and-templates.md)
- [ADR 0037](../adr/0037-plugin-and-template-relationships.md)
- [new-project-flow.md](../design/new-project-flow.md)
- [ui-implementation-plan.md](../design/ui-implementation-plan.md) — NEW-*, REL-* tickets
