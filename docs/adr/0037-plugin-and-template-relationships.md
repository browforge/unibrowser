# ADR 0037: Plugin and template relationships

- **Status:** Accepted
- **Date:** 2026-10-13
- **Relates to:** [ADR 0006](0006-mod-manifest-and-versioning.md), [ADR 0025](0025-mod-apis-and-dependencies.md), [ADR 0031](0031-edit-history-as-file-format.md), [ADR 0036](0036-new-project-empty-by-default-and-templates.md), [ADR 0029](0029-plugin-permissions-and-network-security.md)
- **Research:** [new-project-and-templates.md](../research/new-project-and-templates.md)

## Context

Mods already declare Factorio-style `dependencies` in `info.json` ([ADR 0006](0006-mod-manifest-and-versioning.md)). Template packs ([ADR 0036](0036-new-project-empty-by-default-and-templates.md)) need the same honesty about **which plugins must be present**, which **improve** the experience, which **integrate opportunistically**, and which **must not run together**.

Industry models:

| System | Mechanism | Lesson |
| --- | --- | --- |
| **Factorio** | `dependencies` in `info.json` with prefixes: required, `?` optional, `(?)` hidden optional, `!` incompatible, `~` no load-order effect ([Mod structure](https://wiki.factorio.com/Mod_structure#info.json)) | Explicit incompatible; optional without forcing load order |
| **npm** | `dependencies`, `peerDependencies`, `optionalDependencies`; ecosystem **recommended** / suggested deps in tooling ([package.json](https://docs.npmjs.com/cli/v10/configuring-npm/package-json#dependencies)) | Ranges + peers; optional install must not break consumers |
| **Visual Studio** | Project templates declare required workloads/SDKs; **Create a new project** surfaces missing prerequisites before configure ([Create a project](https://learn.microsoft.com/en-us/visualstudio/ide/create-new-project)) | User sees dependency gaps in the template picker, not after save |

Unibrowser needs one **relationship vocabulary** for **plugin manifests** (`info.json`) and **template manifests** (`template.json`), plus shared **resolution**, **install consent UI**, **lockfile pins** ([ADR 0031](0031-edit-history-as-file-format.md)), and **actionable conflict copy**.

## Decision

### 1. Relationship kinds (four + host)

All relationship entries reference a **plugin id** (`name` from `info.json`) and a **semver range** (npm semver syntax: `^`, `~`, `>=`, exact pin allowed).

| Kind | JSON key | Factorio analogue | npm analogue | Meaning |
| --- | --- | --- | --- | --- |
| **Requires** | `requires` | (no prefix) | `dependencies` | Hard dependency. Host **refuses to enable** the declaring pack/mod or **refuses to apply** the template until every `requires` entry is satisfied by an **enabled** plugin version in range. |
| **Recommends** | `recommends` | (ecosystem convention) | suggested / `peerDependencies`-like | **Suggested** companion on the consent sheet: **checkbox on by default**; user may uncheck before **Continue**. Example: a DOCX **export** mod recommends a **document editor** mod. |
| **Optional** | `optional` | `?` optional | `optionalDependencies` | Listed on the same sheet with **checkbox off by default**; user may enable. If absent or unchecked, the declaring mod/template still works (integration hooks off). |
| **Incompatible** | `incompatible` | `!` | (no direct field; overrides) | If the other plugin is **enabled**, the declaring plugin **cannot be enabled** and the template **cannot be applied**. UI shows **reason** string from manifest (required). |

**Not modelled here (future):** Factorio `~` (dependency without load-order edge). Unibrowser load order remains topological on `requires` only ([ADR 0006](0006-mod-manifest-and-versioning.md)).

**Host virtual `base`:** satisfies no relationship target by itself; `requires.base` remains valid for API floor.

### 2. Manifest shape

#### 2a. Plugin `info.json`

Add (or migrate from legacy `dependencies` / `optional_dependencies`):

```json
{
  "name": "export-docx",
  "version": "1.2.0",
  "relationships": {
    "requires": {
      "base-text": ">=0.3.0 <0.4.0"
    },
    "recommends": {
      "rich-document": "^2.0.0"
    },
    "optional": {
      "grammar-harper": "^1.0.0"
    },
    "incompatible": {
      "legacy-docx-flat": {
        "range": "*",
        "reason": "Uses a different document graph; enable only one DOCX path."
      }
    }
  }
}
```

- Legacy `dependencies` maps to `relationships.requires` on load (deprecation warning in portal).
- `requires`, `recommends`, and `optional` values are a **range string** or `{ "range", "reason" }`; `reason` is shown on the consent sheet (required for `incompatible`; strongly encouraged for `requires`).
- `incompatible` values use the same object form; `reason` is mandatory and shown in UI.

#### 2b. Template `template.json`

Same `relationships` object ([ADR 0036](0036-new-project-empty-by-default-and-templates.md) amends §5). **Templates do not enable plugins**; they **declare** what the starting graph needs. Apply-time resolution uses the same engine as mod enable.

Remove separate `requires.plugins` / `optional_plugins` / `companion_mods` arrays in favour of `relationships` (migration: old fields → `requires` / `optional` for one release).

### 3. Resolution algorithm

Run on: **enable plugin**, **install plugin**, **apply template**, **open project** (replay pins).

1. **Collect** declared relationships from the target pack plus all currently **enabled** plugins (transitive `requires` only).
2. **Incompatible pass:** for each enabled plugin `A`, if any enabled `B` matches `A.relationships.incompatible` or mutual incompatibility, **block** with message (§6).
3. **Requires pass:** for target `T`, every `requires` entry must match some **installed** package version in range. Installed but **disabled** counts as missing for enable/apply.
4. **Recommend / optional:** never block; compute **offers** for consent UI (§4).
5. **Load order:** topological sort on `requires` edges among **enabled** mods ([ADR 0006](0006-mod-manifest-and-versioning.md)); cycles → block with cycle list.
6. **Pin header** ([ADR 0031](0031-edit-history-as-file-format.md) §3): on first mutating op after enable/apply, append pins for **exact versions** of every plugin that participated in resolution (enabled set), not merely `requires`.

Opening a saved project: replay uses **pin header**, not template file. Relationship declarations are validated only when user **changes** enabled set or applies a new template.

### 4. Install and apply consent UX

Single **relationship consent** surface (non-modal sheet; blocks the triggering action until dismissed):

| Section | Controls | Default |
| --- | --- | --- |
| **Required** | Locked **on** (no toggle); plugin id, version range, installed candidate; manifest **`reason`** or host fallback | All must be installed/enabled; **Continue** disabled until satisfied or user runs **Install missing** |
| **Recommended** | Checkbox per entry + manifest **`reason`** (or fallback) | **Checked** by default; user may uncheck any entry |
| **Optional** | Checkbox per entry + **`reason`** (or fallback); same sheet, below recommends | **Unchecked** by default; user may check any entry |
| **Incompatible** | Read-only red list if user already has conflicting mods enabled | **Continue** disabled until user disables conflicting mod or cancels |

**Install from portal:** show consent after **Install** click, before download when network needed ([ADR 0029](0029-plugin-permissions-and-network-security.md)).

**Apply template:** show consent on **Create** after configure ([new-project-flow.md](../design/new-project-flow.md)). Required missing → **Install** / **Cancel**; recommends start checked; optional start unchecked.

**Never:** skip the consent sheet; enable incompatible pair; turn on optional entries without an explicit user check.

Save, autosave, and foreign-file open remain enabled when relationships are unsatisfied; only the **dependent feature** greys out ([ADR 0023](0023-saving-is-sacred.md)).

### 5. Lockfile and template interaction ([ADR 0031](0031-edit-history-as-file-format.md))

| Event | Lock / pin behaviour |
| --- | --- |
| User enables mods after consent | Pin header records **exact** `plugin_version` per enabled id |
| User applies template | Template `id@version` stored in `manifest.json` on first save ([ADR 0036](0036-new-project-empty-by-default-and-templates.md)); pins reflect plugins **enabled after consent** (checked requires, checked recommends, checked optional) |
| User unchecks a **recommend** or **optional** on the sheet | Those ids are not installed/enabled; not pinned on first op |
| Hub updates plugin | Does not rewrite pins; **Update available** in portal; replay still uses pinned version until user runs **Upgrade project plugins** (future command) |
| Missing pin at replay | Block replay for that plugin’s ops with **Install version x.y.z** (existing ADR 0031 rule) |

Template `relationships.requires` is a **declaration**, not a substitute for pin header: saved work always pins what actually ran.

### 6. Conflict and status messages

Copy is human-first; include **id**, **required range**, **found version** or **disabled**, and **one action**.

| Code | When | User message (pattern) |
| --- | --- | --- |
| `REL_REQUIRES_MISSING` | enable/apply | “**{title}** needs **{id}** ({range}). Install it from the portal or disable **{title}**.” |
| `REL_REQUIRES_DISABLED` | enable/apply | “**{id}** is installed but turned off. Enable it to use **{title}**.” |
| `REL_REQUIRES_RANGE` | enable/apply | “**{id}** is {found}, but **{title}** needs {range}.” |
| `REL_INCOMPATIBLE` | enable/apply | “**{a}** cannot run with **{b}**. {reason}” |
| `REL_CYCLE` | enable | “Circular dependency: {a} → {b} → {a}. Contact the mod authors.” |
| `REL_TEMPLATE_APPLY` | apply | “This template needs plugins you do not have yet.” + consent sheet |
| `REL_RECOMMEND_OFF` | runtime | (No toast.) Missing recommend: feature shows **Install {id} for {feature}** inline. |

Portal list rows show chips: **Needs mod**, **Conflict**, **Recommend available**.

### 7. Governance

- Schema version `relationship_schema: 1` on both manifest types.
- Validator in `unibrowser-mod-host` rejects unknown relationship keys and incompatible entries without `reason`.
- Hub index may mirror relationship summaries for search filters (Visual Studio–style tags).

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes — recommends default on but user can decline; clear conflicts |
| Never blocks save or work | Yes — blocks enable/apply, not Save |
| Local-first | Yes — resolution uses installed cache |
| Predictable and consistent | Yes — same four kinds for mods and templates |
| Simple durable mechanism | Yes — semver ranges + pin header |
| Mod not core | Yes — host resolves; mods declare |

## Consequences

**Positive**

- One mental model for portal, template gallery, and enable toggles.
- Export/editor pairing via `recommends` without hard coupling.
- Incompatible sets prevent double-DOCX-style disasters.

**Tradeoffs**

- Migration from legacy `dependencies` fields.
- Consent sheet adds a step to template apply (matches Visual Studio prerequisite honesty).
- Transitive requires across many mods need careful cycle diagnostics.

## References

- [ADR 0036](0036-new-project-empty-by-default-and-templates.md)
- [new-project-flow.md](../design/new-project-flow.md)
- [Factorio info.json](https://wiki.factorio.com/Mod_structure#info.json)
- [npm package.json dependencies](https://docs.npmjs.com/cli/v10/configuring-npm/package-json#dependencies)
- [Visual Studio: Create a new project](https://learn.microsoft.com/en-us/visualstudio/ide/create-new-project)
- [Visual Studio project templates](https://learn.microsoft.com/en-us/visualstudio/ide/how-to-create-project-templates)
- [.vstemplate schema](https://learn.microsoft.com/en-us/visualstudio/extensibility/project-template-schema-reference)
- [Custom templates for dotnet new](https://learn.microsoft.com/en-us/dotnet/core/tools/custom-templates)
