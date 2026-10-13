# ADR 0036: New project empty by default and templates

- **Status:** Accepted
- **Date:** 2026-10-13
- **Relates to:** [ADR 0015](0015-native-project-format.md), [ADR 0032](0032-unibrow-container-archive-and-folder-modes.md), [ADR 0031](0031-edit-history-as-file-format.md), [ADR 0029](0029-plugin-permissions-and-network-security.md), [ADR 0006](0006-mod-manifest-and-versioning.md), [ADR 0035](0035-one-document-model-wysiwyg.md), [ADR 0037](0037-plugin-and-template-relationships.md)
- **Research:** [new-project-and-templates.md](../research/new-project-and-templates.md)
- **Design:** [new-project-flow.md](../design/new-project-flow.md)

## Context

The October prototype still surfaces **“New project”** as a format-flavoured choice (e.g. video story templates in [workflows.md](../design/workflows.md)) and duplicates entry points on the New tab page. Office-style products train users to pick **Word vs PowerPoint vs PNG** at creation time. That conflicts with Unibrowser’s positioning: **one native project** (`.unibrow`) that can hold heterogeneous content, with **foreign files opened in place** and **delivery formats chosen at export** (ADR 0035 direction).

Bear’s seed decision (2026-10):

- **New** defaults to an **empty Unibrowser project** — in-memory `.unibrow` graph, **nothing written to disk** until the first mutating edit or an explicit Save, following [ADR 0032](0032-unibrow-container-archive-and-folder-modes.md) consent rules.
- **New from template** is optional: built-in starters, **user-created templates**, and **hub-installed** template packs (mod portal content).
- **Export** to docx, pptx, png, mp4, etc. is a **save/export** choice, not a creation wizard step.
- **Opening an existing foreign file** remains as today (sidecar history per [ADR 0031](0031-edit-history-as-file-format.md) §9).

This ADR is the anchor other UX and packaging decisions hang from: **content and intent first, formats second.**

## Decision

### 1. Creation verbs

| User action | Result |
| --- | --- |
| **New** (menu, NTP, shortcut) | New tab: **empty project** session (`content_kind: project.unibrow`), untitled label, **no filesystem path**, **no disk writes** on open |
| **New from template…** | Non-blocking gallery: pick template → same as New, then **apply template payload** in memory (§4) |
| **Open file / Open folder** | Unchanged browse and foreign-native open paths |
| **Save / Save As** | First persist chooses location and archive vs folder ([ADR 0032](0032-unibrow-container-archive-and-folder-modes.md)) |
| **Export…** | Projection to delivery format (ADR 0035); never offered on the New screen as a peer of “blank project” |

There is **no** File → New Word Document, New Presentation, or New Image. Those capabilities live under **Export** and mod-provided projections.

### 2. Empty project semantics

- Host seeds a **minimal valid graph**: empty canvas/timeline roots per bundled `base-unibrow` (or successor), zero embedded assets, title `"Untitled project"`.
- Session storage mode starts as **in-memory edit** until Save: no `.unibrow` file, no `.unibrow-history`, no `manifest.json` on disk ([ADR 0032](0032-unibrow-container-archive-and-folder-modes.md) §4).
- First mutating edit: if the user has already chosen a save target, follow that container’s rules; if not, continue in memory and show status **Unsaved project** with plain copy that **Save** will ask where to store history.
- **New Window** (when multi-window ships) opens another empty project or restores session per [session-restore.md](../session-restore.md); it does not imply a format.

### 3. Template sources (three lanes)

| Lane | Origin | Discovery |
| --- | --- | --- |
| **Built-in** | Shipped with the app (Office-style doc starter, presentation board, simple video timeline, design space, notes board, …) | New from template gallery, filter **Built-in** |
| **User** | **Save as template…** from any saved `.unibrow` or folder project | Gallery filter **My templates**; files under user config (§5) |
| **Hub** | Mod portal **Templates** category; install pins pack like a mod ([ADR 0006](0006-mod-manifest-and-versioning.md)) | Gallery filter **Installed**; search includes hub metadata |

Templates are **starting graphs + optional asset blobs + metadata**. They are not a separate app mode.

### 4. Template package format

**File extension:** `.ubtemplate` (single OPC zip, same compression rules as [ADR 0032](0032-unibrow-container-archive-and-folder-modes.md) §1) **or** directory bundle `.ubtemplate/` with identical interior layout.

**Required parts:**

| Part | Role |
| --- | --- |
| `template.json` | Manifest (§5) |
| `project.json` | Initial graph snapshot to load into session |
| `assets/` | Optional content-addressed blobs referenced by graph |
| `preview/` | Optional `preview.png` (and `preview-dark.png`) for gallery thumbnails |

**Must not include:** executable plugin binaries, Rhai scripts, or VSIX payloads. Those ship as **mods** installed separately.

**Apply algorithm:**

1. Create empty project session (§2).
2. Run **relationship resolution** for the template pack ([ADR 0037](0037-plugin-and-template-relationships.md)); show consent sheet for requires/recommends before continuing.
3. Replace graph from `project.json`; register assets from `assets/`.
4. On first mutating op, write **pin header** for exact versions of enabled plugins ([ADR 0031](0031-edit-history-as-file-format.md) §3).
5. Unsatisfied `requires` after user dismisses consent: **inline banner** with **Install**; optional integrations stay grey per ADR 0037; never block Save.

### 5. Template manifest (`template.json`)

Versioned document (`template_schema_version`, initial `1`). Plugin relationships use [ADR 0037](0037-plugin-and-template-relationships.md) (`requires`, `recommends`, `optional`, `incompatible`).

| Field | Required | Meaning |
| --- | --- | --- |
| `id` | yes | Stable URI-style id (`unibrowser.template.video.simple`) |
| `version` | yes | Semver of **this template pack** (not the app) |
| `title`, `description` | yes | Gallery copy; localisable keys allowed |
| `categories` | yes | e.g. `document`, `presentation`, `video`, `design`, `notes` |
| `tags` | no | Search facets (Visual Studio–style); e.g. `video`, `timeline`, `office` |
| `relationships` | yes | ADR 0037 object; at minimum `requires` may be `{}` for blank template |
| `default_workspace` | no | Hint: `browse` \| `write` \| `create` \| `live` ([ux.md](../design/ux.md)) — applies layout only |
| `min_app_version` | no | Host semver lower bound |
| `hub` | no | `{ "publisher", "content_hash" }` when installed from portal |

**Versioning / pinning:**

- Applying a template records its `id@version` in the project manifest when the project is first saved.
- Hub updates do not mutate user projects; **Update template** in portal installs a new pack version; gallery shows both until user removes old pack.
- Reproducibility: saved projects replay with **history pin header**, not the template file ([ADR 0031](0031-edit-history-as-file-format.md)).

### 6. User-created templates

- Command: **File → Save as template…** (and project context menu when graph is saved at least once or user confirms exporting current in-memory state).
- Writes `.ubtemplate` to user-chosen path **and** registers in `templates/user/index.json` under config dir.
- User templates may include **embedded assets** the project already contains; must not embed mods.
- **Remove** from gallery deletes registry entry only; does not delete user’s file without confirm.

### 7. Hub and mod portal

- Template packs are **first-class portal artefacts** beside mods: signed catalog entries, offline cache, pinned checksum ([mod-system.md](../mod-system.md)).
- Installing a template pack extracts to `templates/installed/<id>/<version>/` (read-only to mods).
- Portal UI: **Templates** tab with Install / Update / Uninstall; no auto-apply on install.
- Template pack `relationships` drives the shared **relationship consent** sheet ([ADR 0037](0037-plugin-and-template-relationships.md)); network fetch still follows [ADR 0029](0029-plugin-permissions-and-network-security.md).

### 8. Privacy and security (ADR 0029)

| Rule | Behaviour |
| --- | --- |
| **Data-only default** | Template apply reads JSON and assets only; no `eval`, no WASM, no native code in the pack |
| **No network on apply** | Hub fetch happens at **install** time with existing portal gates; applying a local template is offline |
| **Plugin code** | Missing plugins prompt install; installed plugins run under normal tier + permission grants |
| **Secrets** | Templates cannot declare `secrets` slots; mods installed separately go through grant store |
| **Untrusted packs** | Unsigned hub entries show **Install anyway** with publisher warning; same trust model as unsigned mods |

Templates **cannot bypass** [ADR 0029](0029-plugin-permissions-and-network-security.md): they do not receive network or filesystem scopes by themselves.

### 9. Built-in starter set (v1)

Ship at least:

| Template id | Intent |
| --- | --- |
| `unibrowser.template.blank` | Empty graph (same as **New**; hidden from gallery or shown as “Blank” tile) |
| `unibrowser.template.document.office` | Single rich-text frame + print-oriented page setup |
| `unibrowser.template.presentation` | Slide board + speaker notes region |
| `unibrowser.template.video.simple` | Timeline + title card placeholder |
| `unibrowser.template.design.space` | Infinite canvas + grids |
| `unibrowser.template.notes` | Lightweight board + markdown note nodes |

Exact graph shapes are owned by bundled mods; this ADR fixes **product verbs and pack shape** only.

### 10. UI surfaces (summary)

Detailed UX: [new-project-flow.md](../design/new-project-flow.md). Implementation: NEW-* tickets in [ui-implementation-plan.md](../design/ui-implementation-plan.md).

- New tab: **New** primary; **New from template** secondary; **Open file** / **Open folder**; recents below.
- File menu: **New**, **New from template…**, **Open…**; template gallery follows Visual Studio **Create a new project** patterns ([research](../research/new-project-and-templates.md)).
- Remove duplicate “New project” rows and format pickers from NTP.

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes — empty default; templates optional; explicit Save path |
| Never blocks save or work | Yes — missing plugins grey features, not Save |
| Local-first | Yes — templates cache locally; apply is offline |
| Predictable and consistent | Yes — one project type at creation; foreign open unchanged |
| Simple durable mechanism | Yes — OPC template pack mirrors `.unibrow` |
| Mod not core | Yes — graph semantics in mods; host owns session + apply + gallery |

## Consequences

**Positive**

- One mental model: you always start a **Unibrowser project**; export is delivery.
- Aligns with ADR 0032 consent (no disk litter from New).
- Hub templates reuse mod portal trust, pinning, and offline cache.
- User templates support schools, studios, and FOSS shareable starters without forking the app.

**Tradeoffs**

- Users trained on Office may look for “New PowerPoint” until Export is discoverable.
- Template + plugin dependency UX adds portal steps for rich starters.
- `.ubtemplate` is another OPC shape to validate and document.
- Built-in starters require ongoing design QA so they do not imply format lock-in.

## References

- [new-project-and-templates.md](../research/new-project-and-templates.md)
- [new-project-flow.md](../design/new-project-flow.md)
- [workflows.md](../design/workflows.md) — update journey 1 and 5
- [ADR 0032](0032-unibrow-container-archive-and-folder-modes.md)
- [ADR 0031](0031-edit-history-as-file-format.md)
- [ADR 0029](0029-plugin-permissions-and-network-security.md)
- [ADR 0037](0037-plugin-and-template-relationships.md)
