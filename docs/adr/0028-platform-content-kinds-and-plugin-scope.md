# ADR 0028: Platform model — content kinds and full-scope plugins

- **Status:** Accepted
- **Date:** 2026-10-11
- **Supersedes in part:** ADR 0002, ADR 0005, ADR 0010, ADR 0025, ADR 0027 (see below)

## Context

Unibrowser is not a 1:1 clone of a monolithic office or creative suite (LibreOffice, GridCraft/WordCraft/ArtCraft-style all-in-one apps). It is a **stable, extensible platform**: a small core plus community-built plugins that supply features, views, formats, and services.

Today, much of the documentation still frames mods primarily as **file-format backends** and **document-type contexts** (Text, Office, Image, PDF). That model forces duplicate tools per app and export round trips when the same semantic object (for example a raster image) appears inside another host (DOCX, Markdown, video frame).

The owner direction:

1. **Plugin scope is unrestricted** within host contracts: features, views, panels, toolbars, Command Manager groups, scripts, commands, format readers/writers, content-kind tools, render or analysis providers, and long-running services — not only “open this extension”.
2. **Content kinds** are the unit tools target (`image`, `text_run`, `rich_text`, `audio_clip`, `video_frame`, `vector_shape`, `table`, `mesh_3d`, …). Format plugins expose what they hold as **content-kind nodes** in a traversable tree. A colour-profile tool declared for `image` applies to a PNG, an image embedded in a DOCX, a video frame, or an inline image in Markdown without leaving the host file.
3. **Capability states** (`supported`, `emulated`, `unsupported`, plus `hidden_by_user` and host-only `locked`) apply **per content kind** and **per host format** (storage backend). A host that cannot persist a colour profile marks the operation emulated (baked in) or unsupported with a reason — not hidden.
4. **Core** owns stable APIs, consistent placement and behaviour, trust and permissions, and **never blocking Save**. **Plugins** own feature semantics, format logic, emulations, and optional services.

## Decision

### 1. Product positioning

- Core ships a **browser shell**, command system, mod host, and bundled `base-*` proving mods — not a complete clone of every competitor SKU.
- Optional plugins democratise features: the community builds what it needs; core keeps interaction consistent.
- Discoverability is primarily through the **Mod Portal** (recommended mods, install-from-file, context hints), not by hiding gaps behind a single vendor catalogue.

### 2. Content kinds as the tool target

- Every editing or inspection **tool** (command + control family) declares one or more **content kinds** it operates on.
- Every open document exposes a **content tree**: nodes typed by content kind, with stable node ids, parent links, and optional embedded resources (blobs, sidecars).
- **Selection** resolves to one or more content-kind nodes (possibly mixed kinds). Capability and value state are computed against that selection and the active **host format** (the mod/backend that owns persistence for the root file).
- Core maintains a **versioned content-kind registry** (canonical ids, human labels, optional parent/child relationships). Plugins may register **extension kinds** only under declared namespaces; core kinds use the `unibrowser.content.*` prefix.

Illustration (colour tool):

- Tool `acme.colour.profile` declares `content_kinds: ["unibrowser.content.image"]`.
- User selects an embedded image in a DOCX → same control placement and behaviour as for a standalone PNG; capability comes from DOCX backend policy (supported vs emulated ICC bake vs unsupported reason).

### 3. Host format vs content kind

| Concept | Meaning |
| --- | --- |
| **Host format** | Root persistence backend (e.g. `docx`, `png`, `markdown`, `unibrow`) |
| **Content kind** | Semantic type of a node in the content tree |
| **Capability** | For each control, state per **(selection content kind, host format)** |

Format plugins implement **projections** of their internal model into content-kind nodes ([ADR 0025](0025-mod-apis-and-dependencies.md) projections evolve from “per document type” to “per content tree”). Tools never depend on file extension alone.

### 4. Core responsibilities

| Core | Plugins |
| --- | --- |
| Shell, tabs, downloads, session | Feature semantics and domain logic |
| Command registry, resolution, placement | Commands, handlers, emulations |
| Content-kind registry and traversal API | Format read/write, content tree population |
| Capability resolver orchestration | Per-kind capability providers |
| Permission prompts, signing policy, sandbox tiers | Optional services (sidecars, WASM, native) |
| Fixed UI slots (Command Manager, docks, status bar) | Contributions into those slots only |
| Save / Save As / autosave / recovery (always available) | Must not gate, cover, or replace Save |
| Mod Portal, dependency resolver, API registry | `provides` / `dependencies` APIs |

### 5. Plugin contribution scope

Plugins may contribute (via manifest metadata + compiled payload), subject to permissions:

- Commands, scripts (Rhai / registered handlers), and keybinding suggestions
- Command families, groups, controls, and Command Manager tabs
- Floating and detachable toolbars
- **Panels and views** in host dock regions (left, right, bottom)
- **Status-bar items** in fixed left/right slots ([status-bar.md](../design/status-bar.md))
- File-format readers/writers and content-kind adapters
- **Providers** (render, analysis, export pipeline hooks) registered against content kinds
- **Services** (long-lived, host-mediated; optional sidecar)

Plugins may **not**: inject arbitrary HTML/CSS over content; hide core controls via capability responses; disable Save; or capture shortcuts outside the host keybinding map ([ADR 0027](0027-command-system-capabilities-and-extensibility.md)).

### 6. Versioned plugin API and dependencies

- **`unibrowser_api`** (semver): binary ABI and host callback contracts ([ADR 0006](0006-mod-manifest-and-versioning.md)). Major mismatch → refuse load with a clear error.
- **`command_ui_schema`** (semver): declarative `contributes` shape for UI metadata ([ADR 0027](0027-command-system-capabilities-and-extensibility.md)).
- **`content_kind_schema`** (semver, new): registry entries, node metadata, and traversal callbacks.
- **`provides` / `dependencies` / `api_dependencies`** ([ADR 0025](0025-mod-apis-and-dependencies.md)): Factorio-style composition; optional deps degrade with resolver notes, never block Save or shell startup.
- Stable **content-kind ids** and **command/control ids** survive label and implementation changes; breaking semantics require major API or schema bumps and migration notes.

Execution tiers remain as in [ADR 0002](0002-mod-system-and-sandboxing.md) (T0 bundled, T1 signed native, T2 WASM, T3 sidecar). Full-scope plugins do not change tier trust rules.

### 7. Permission scopes (extended)

Least-privilege permissions are declared in the manifest and shown before enablement. Core additions for platform scope:

| Permission | Grants |
| --- | --- |
| `content.read.tree` | Traverse content-kind nodes for active document |
| `content.read.selection` | Read selection mapped to content-kind nodes |
| `content.edit` | Apply edits through host transactions (per node) |
| `content.kind.register` | Register extension content kinds under plugin namespace |
| `ui.panel.contribute` | Contribute docked panel in allowed regions |
| `ui.view.contribute` | Register a host-hosted view surface (no arbitrary DOM) |
| `service.register` | Host-managed background service entry |
| `provider.render` / `provider.analysis` | Register provider for declared content kinds |

Existing command/UI permissions from ADR 0027 remain (`commands.execute`, `commands.capability.provide`, `document.read.selection`, `document.edit`, `ui.command_manager.contribute`, `ui.toolbar.contribute`, `ui.toolbar.detach`, status contributions via `ui.status.contribute`).

Capability callbacks stay **read-only**; edits use host transactions and undo. Save paths ignore capability overlays.

### 8. Supersedes and amends (by ADR)

| ADR | What changes |
| --- | --- |
| **0002** | **Superseded in part:** mods described mainly as format payloads. **Still valid:** manifest + compiled payload, tiers, VSIX separate lane. **New:** mods are general platform plugins; format support is one contribution type. |
| **0005** | **Superseded in part:** mode/tab enablement keyed only on backend mode. **Still valid:** grey-not-hide, fixed positions, JSON layouts. **New:** mode follows **work context**; control capability follows **content kind + host format**. |
| **0010** | **Superseded in part:** `CommandContext` rollout as file-type tabs (Text/Office/Image). **Still valid:** registry, slots, persistence, palette. **New:** tabs place **families**; enablement uses content-kind resolver, not extension alone. |
| **0025** | **Amended:** `text_projection` / `raster_projection` become views over **content-kind trees**; API deps unchanged. Projections are required for search, OCR, and accessibility — not optional niceties. |
| **0027** | **Amended:** `Context.document_type` is insufficient alone; resolution inputs include **selection content kinds**, **host format id**, and **embedded node path**. Family placement and Save exclusion unchanged. |

[ADR 0014](0014-vsix-compatibility-layer.md) and [ADR 0018](0018-extension-host-stub.md) remain the VSIX lane; VSIX contributions map into the same fixed slots and content-kind declarations where applicable.

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes — optional plugins, visible permissions, user provider choice, portal install |
| Never blocks save or work | Yes — Save explicit outside capability negotiation; missing optional APIs degrade |
| Local-first | Yes — registry, traversal, and portal work offline; network opt-in for catalogues |
| Predictable and consistent | Yes — same tool id and placement for a content kind across host formats |
| Simple durable mechanism | Yes — versioned ids, semver APIs, JSON persistence, deterministic resolver |
| Mod not core | Yes — features and formats in plugins; core mediates trees, capabilities, and slots |

## Consequences

**Positive**

- One colour (or metadata) tool works across embedded and standalone assets without export gymnastics.
- Community can ship narrow mods (one kind, one format) without forking a suite.
- Clear split between “where controls live” (core) and “what they do” (plugins).
- Portal and content-kind hints improve discoverability (“install image tools”) vs opaque file associations alone.

**Tradeoffs**

- **API stability burden:** content-kind registry and traversal are long-lived contracts; breaking changes hurt many mods.
- **Quality variance:** more plugins ⇒ uneven polish; core must keep placement and capability honesty consistent.
- **Discoverability:** users rely on portal curation and context recommendations; weak catalogues feel sparse compared to monolithic suites.
- **Implementation cost:** embedded traversal, mixed-kind selection, and per-host capability matrices are more work than per-viewer enablement flags.
- **Testing surface:** contract tests must cover cross-host scenarios (same kind, different persistence).

## References

- [principles.md](../principles.md)
- [positioning.md](../positioning.md)
- [Command system](../design/command-system.md)
- [Mod system overview](../mod-system.md)
- [UI implementation plan](../design/ui-implementation-plan.md) — CMD-12 through CMD-15
