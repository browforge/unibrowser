# ADR 0030: Core as skeleton — capability providers and swappable defaults

- **Status:** Accepted
- **Date:** 2026-10-12
- **Supersedes in part:** ADR 0002, ADR 0005, ADR 0009, ADR 0010, ADR 0015, ADR 0021, ADR 0022, ADR 0028 (see §8)

## Context

[ADR 0028](0028-platform-content-kinds-and-plugin-scope.md) positions Unibrowser as a platform with a small core and full-scope plugins. In practice, the desktop shell still bundles many **user-visible features** (Monaco text editing, PDF viewing, timeline workshop, proofing defaults) as if they were product surface owned by “Unibrowser” rather than by named plugins.

Bear’s direction: the **core is a skeleton**. It ships almost no end-user editing features. What users experience as “the app” is the composition of **plugins** (including first-party `base-*` mods shipped enabled by default). The core owns stable infrastructure only: browser shell, plugin host, command and UI slot registry, permission enforcement, save pipeline, and the **history store** API that persists edit operations ([ADR 0031](0031-edit-history-as-file-format.md)).

Marketing and README copy must not imply a monolithic office or creative suite. Bundled defaults are **convenience**, not identity: any default provider can be replaced by another plugin that claims the same **capability**.

## Decision

### 1. Core ships (only)

| Responsibility | Notes |
| --- | --- |
| **Shell** | Tabs, windows, omnibox, downloads disposition, session restore hooks ([ADR 0009](0009-browser-as-unified-shell.md)) |
| **Plugin host** | Load, dependency resolve, tiers, signing policy ([ADR 0002](0002-mod-system-and-sandboxing.md), [ADR 0029](0029-plugin-permissions-and-network-security.md)) |
| **Command system** | Registry, resolution, palette, Command Manager **slots** ([ADR 0010](0010-ui-commands-and-command-manager.md), [ADR 0027](0027-command-system-capabilities-and-extensibility.md)) |
| **UI slots** | Fixed docks, status bar regions, toolbar attachment points; no domain editors |
| **Permissions** | Manifest validation, grant store, mediated network/FS/secrets |
| **Save pipeline** | Save / Save As / autosave / atomic write policy ([ADR 0023](0023-saving-is-sacred.md)) |
| **History store** | Append log, snapshots, replay orchestration; no knowledge of PDF vs text semantics ([ADR 0031](0031-edit-history-as-file-format.md)) |
| **Content-kind registry** | Canonical ids and traversal orchestration ([ADR 0028](0028-platform-content-kinds-and-plugin-scope.md)) |

Core does **not** ship: a text editor, PDF renderer, spreadsheet grid, timeline clip editor, spellchecker, grammar engine, or AI assistant. Those are plugins. First-party `base-*` mods are **default-enabled plugins**, not core code paths exposed as “built-in Unibrowser features.”

### 2. Capabilities and providers

A **capability** is a stable, versioned contract id (e.g. `unibrowser.capability.open_text`, `unibrowser.capability.render_pdf`, `unibrowser.capability.project_surface`) describing what the host needs from a plugin to open or edit a class of content. Capabilities are **not** file extensions; extensions and MIME types map to capabilities through host routing rules and plugin `contributes` metadata.

A **capability provider** is a plugin entry that declares:

- `provides`: one or more capability ids (with optional priority and content-kind scope)
- Handler hooks: open, save projection, edit transactions, export hooks as required by the capability schema
- Optional **default** flag: “prefer this provider when multiple are installed and the user has not chosen otherwise”

The host maintains a **provider registry** at runtime:

1. Collect all enabled plugins’ `provides` entries for a capability.
2. Apply user **preference** if set (Settings → Plugins → Defaults, or per-file “Open with…”).
3. Else choose the highest-priority provider among those marked `default: true` (first-party bundles use this).
4. Else choose the sole provider; if multiple with no default, show a non-modal **picker** (remember choice per capability or per path pattern).

**Replacement:** Installing `community-markdown` that provides `unibrowser.capability.open_text` does not fork the shell. The user (or workspace policy) sets it as default; Command Manager placement and command ids for cross-cutting actions (Save, Undo) stay core-owned. The old provider remains available via Open with and the portal.

**Conflicts:** Two defaults for the same capability → load-time diagnostic; neither wins until the user picks. Save and shell startup are never blocked ([ADR 0023](0023-saving-is-sacred.md)).

### 3. First-party bundled plugins

Shipped in the default distribution, enabled on first run unless the user disabled them in safe mode or preferences:

- `base-text`, `base-pdf`, `base-docx`, `base-png`, `base-svg`, … — format and surface providers
- `spell-en-nz`, `grammar-harper` — proofing ([ADR 0022](0022-proofing-as-mods.md))
- `ai-assistant` (optional enable) — assistant ([ADR 0026](0026-ai-assistant-as-mod.md))

User-facing copy names the **plugin** (“Text (base-text)”) in About → Plugins, not “Unibrowser Text Editor.” Screenshots and docs may say “text editing” but architecture docs and ADRs treat the editor as a mod.

### 4. What “Unibrowser” claims

| Unibrowser (core) claims | Plugins claim |
| --- | --- |
| Same tab strip for files and web | How each format renders and edits |
| Commands live in the same slots | Command handlers and domain semantics |
| Save always persists current state | Serialising domain state into save/history ops |
| Permissions and trust | Network, FS, and sidecar behaviour |
| Undo/redo through history store | Op types and replay/migrate |

### 5. `.unibrow` host role

The **project package** ([ADR 0015](0015-native-project-format.md)) is a host format whose graph and assets are interpreted by project-capability plugins. Core owns the **container** (zip/directory layout, save atomicity, history log files) and validates **format_version**; clip types, timeline semantics, and workshop UI are plugin-provided. See [ADR 0031](0031-edit-history-as-file-format.md) for history inside the bundle.

### 6. Principles alignment

Skeleton core maximises “mod not core” ([principles.md](../principles.md)): FOSS contributors replace one provider without patching Tauri; everyday users keep bundled defaults that feel complete out of the box.

### 7. Consequences

**Positive**

- Clear story for portal and sideload: you are installing a **provider**, not patching the monolith.
- Smaller security and API stability surface for core releases.
- Aligns README/positioning with implementation trajectory (extract viewers into explicit mods).

**Tradeoffs**

- More indirection in docs and support (“which mod owns PDF?”).
- Provider resolution UX must stay honest when nothing is installed (grey Open with link to portal).
- Refactor cost: legacy code paths still in `unibrowser-desktop` should migrate behind capability ids over time.

### 8. Supersedes and amends (by ADR)

| ADR | What changes |
| --- | --- |
| **0002** | **Amended:** “T0 bundled” mods are **default providers**, not core features. Tier rules unchanged. |
| **0005** | **Superseded in part:** work-context modes as product areas. **Still valid:** grey-not-hide, layouts. **New:** context follows **active capability provider** + selection kinds. |
| **0009** | **Amended:** shell tabs are core; **which backend** opens a location is capability routing. |
| **0010** | **Amended:** Command Manager tabs organise **families** contributed by plugins; core registers shell-only commands (Save, layout, navigation). |
| **0015** | **Amended:** `project.json` graph semantics move to project plugins; core owns package IO + history files per 0031. |
| **0021** | **Amended:** diagnostics host is core; **all** language/format diagnostics are mods. |
| **0022** | **Unchanged intent; clarified:** bundled spell/grammar are default-enabled mods, not core. |
| **0028** | **Amended:** §4 core table shrinks to skeleton list above; “bundled base-* proving mods” are default providers. Content kinds and traversal remain core-orchestrated. |

[ADR 0029](0029-plugin-permissions-and-network-security.md) and [ADR 0031](0031-edit-history-as-file-format.md) complement this ADR; no conflict.

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes — explicit default provider choice, Open with, uninstall |
| Never blocks save or work | Yes — missing provider greys open/edit; save of last snapshot still available per 0031 |
| Local-first | Yes — registry and prefs on disk |
| Predictable and consistent | Yes — same slots; capability ids stable across providers |
| Simple durable mechanism | Yes — manifest `provides`, JSON prefs, semver APIs |
| Mod not core | Yes — defining decision of this ADR |

## References

- [positioning.md](../positioning.md), [README.md](../../README.md)
- [ADR 0031](0031-edit-history-as-file-format.md) — history store and file format
- [ui-implementation-plan.md](../design/ui-implementation-plan.md) — SKEL-* tickets
- [mod-system.md](../mod-system.md)
