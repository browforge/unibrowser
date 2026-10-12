# ADR 0032: `.unibrow` container — archive and folder modes

- **Status:** Accepted
- **Date:** 2026-10-12
- **Relates to:** [ADR 0015](0015-native-project-format.md), [ADR 0030](0030-core-as-skeleton.md), [ADR 0031](0031-edit-history-as-file-format.md)
- **Amends:** [ADR 0015](0015-native-project-format.md) (container shape and default UX), [ADR 0031](0031-edit-history-as-file-format.md) (history paths; pin header merged into `.unibrow-history`)

## Context

[ADR 0015](0015-native-project-format.md) fixed the **`.unibrow`** extension and sketched a directory bundle and/or ZIP single file with `project.json`, `assets/`, `refs/`, and `derived/`. It left **single file vs bundle default** and OPC details open.

[ADR 0031](0031-edit-history-as-file-format.md) made **edit history** part of the durable format: one file **`.unibrow-history`** with a **pin header** (plugin versions) and an append-only **op log**; snapshots live in the log and in content-addressed `assets/`. [ADR 0030](0030-core-as-skeleton.md) assigns **package IO**, atomic save, and history files to core; clip semantics stay in plugins.

Users need two honest shapes for the same project:

- **One file** they can email, back up, and double-click (like a `.docx`).
- **A normal folder** they can browse, script, and pair with large external media (Vegas-style video projects, power users, git-friendly text).

They also need **no surprises**: opening a project must not litter the disk; the first real edit must say plainly where history will live. Three **storage modes** must always be visible so nobody wonders whether Unibrowser is “really saving” or only previewing.

## Decision

### 1. Archive: `.unibrow` as standard OPC zip

A **`.unibrow` file** is a **ZIP archive using the Open Packaging Conventions (OPC)** layout — the same open-container approach as DOCX, VSIX, and 3MF. Any unzip tool can list and extract it.

| Rule | Detail |
| --- | --- |
| **Layout** | OPC parts and relationships; `[Content_Types].xml` and `_rels/.rels` at the package root per OPC |
| **Manifest** | `manifest.json` at the **package root** (format version, title, graph summary pointers; full graph still in `project.json` per [ADR 0015](0015-native-project-format.md)) |
| **Project graph** | `project.json` — versioned nodes and timelines (plugin-owned semantics) |
| **History** | **`.unibrow-history`** at the **package root** — single file: pin header + append log ([ADR 0031](0031-edit-history-as-file-format.md)) |
| **Assets** | `assets/<content-hash>` — **content-addressed**; each distinct blob stored **once** (dedupe on insert); snapshot bodies typically live here |
| **References** | `refs/` — external paths when media is linked not embedded (§7) |
| **Derived** | `derived/` — regenerable previews ([ADR 0011](0011-lazy-conversion-native-until-encode.md)) |
| **Compression** | **STORE (uncompressed)** for parts whose bytes are already compressed (typical video, audio, JPEG, PNG, gzip); **DEFLATE** for text, JSON, YAML, `.unibrow-history`, and other metadata parts |
| **Transparency** | No proprietary wrapper; support can ask users to unzip and inspect paths |

Zip and **folder mode** (§2) share the **same relative paths** for `manifest.json`, `project.json`, `.unibrow-history`, `assets/`, `refs/`, and `derived/`; only the outer container differs.

### 2. Folder mode: project directory (no `.unibrow/` subtree)

A **folder-mode project** is a normal directory (`MyShow`, `Holiday 2026`) with **visible** project parts at the top level. There is **no** hidden `.unibrow/` directory.

| Path (project folder) | Role |
| --- | --- |
| `manifest.json` | Same manifest as archive root |
| `project.json` | Live project graph |
| **`.unibrow-history`** | Single hidden file: pin header + append log ([ADR 0031](0031-edit-history-as-file-format.md)) |
| `assets/`, `refs/`, `derived/` | Same layout as inside the archive |

User-visible media and exports may sit **beside** these files in the project folder. The graph records handles; core does not hide sibling files.

**Open detection:** if the user opens `MyShow/` and `manifest.json` (with Unibrowser project `format_version`) is present, treat as folder-mode project rooted at `MyShow/`. If they open `MyShow.unibrow` (file), treat as archive mode unless already unpacked.

### 3. Three storage modes (always visible)

Every open native project session has exactly one **storage mode**. The UI must show it at all times (status bar; see [ux.md](../design/ux.md)).

| Mode | User label | Writes |
| --- | --- | --- |
| **1. View only** | **View only** | **Nothing** to disk — read archive or folder; in-memory display only; no `.unibrow-history`, no journal |
| **2. Work in archive** | **Archive** | Edits the **`.unibrow` file in place** without unpacking the whole project to a visible tree (Word-style `.docx` editing) |
| **3. Folder mode** | **Folder** | Reads and writes **`manifest.json`**, **`project.json`**, **`.unibrow-history`**, and `assets/` in a normal project folder; large media may sit next to them |

**View only** is the default when the user opens a `.unibrow` from an attachment, a read-only volume, or explicitly chooses “Open for viewing”. Save, autosave, and history append are disabled until the user picks an editing mode or **Save As** into a writable location.

**Work in archive** is the default for a writable `.unibrow` file when the user expects a single portable file.

**Folder mode** is chosen by **Unpack to folder** or by opening an existing folder project.

**Conversion is explicit only** — never silent:

| Command | Effect |
| --- | --- |
| **Unpack to folder** | User picks destination; writes visible `manifest.json`, `project.json`, `assets/`, and **`.unibrow-history`**; archive path recorded for optional “Pack back” |
| **Pack as .unibrow** | User picks output `.unibrow` path; builds OPC zip from current folder project; does not delete folder unless user confirms cleanup |

Switching modes mid-session may require save and a short confirmation when history or refs would change shape.

### 4. Consent and no surprises

Aligned with [principles.md](../principles.md): trust the user, predictable behaviour, local files they can inspect.

| Rule | Behaviour |
| --- | --- |
| **Open never creates files** | Opening a `.unibrow` or foreign file does **not** create `.unibrow-history`, journals, or extra project files on disk |
| **First edit creates storage** | The first **mutating** edit (or explicit **Start editing** / Save when starting from view only) creates the durable store: append via archive journal (§5), or create **`.unibrow-history`** (and project files as needed) in folder mode, or foreign sidecar per [ADR 0031](0031-edit-history-as-file-format.md) §9 |
| **Plain status** | Before and after first edit, status bar states where history will be saved, e.g. **History will be saved in `/path/to/file.unibrow`** (archive: inside the package) or **History will be saved in `/path/to/MyShow/.unibrow-history`** |
| **Large project hint** | When estimated unpacked size, embedded asset count, or largest single asset exceeds a **configurable threshold** (default TBD; e.g. hundreds of MB or many large clips), show a **non-blocking** hint: folder mode may be faster for this project — **Unpack to folder** link. **Never** auto-switch mode |
| **Threshold setting** | Settings → Projects → **Suggest folder mode when project exceeds** (size and/or asset count); off disables hints |

Save remains sacred ([ADR 0023](0023-saving-is-sacred.md)): view only does not trap edits — **Save As** or mode change always offers a writable path.

### 5. Work-in-archive: journal, atomic save, recovery

Editing a `.unibrow` zip without full unpack on every keystroke uses a **write journal** outside the zip:

1. While editing, new ops and blob writes go to a **temporary journal** beside the file (`<name>.unibrow~journal/`) or in the app cache keyed by file identity (user preference: “Keep recovery journal next to project”).
2. On **Save** / autosave flush, core **folds** journal changes into a new zip (including updated **`.unibrow-history`**) and **atomically replaces** the `.unibrow` file (write `*.part`, fsync, rename), same spirit as [ADR 0031](0031-edit-history-as-file-format.md) §10.
3. **Crash recovery** on next open: replay journal into the archive or offer restore; non-modal **Recovered N edits after crash** (no modal blocking Save).

**Tradeoff (documented honestly):** very large archives still pay a **full repack** on save unless the user works in **folder mode**, where `.unibrow-history` and `assets/` append on disk without rewriting gigabytes inside zip. The size hint (§4) steers without forcing.

### 6. Interior consistency and tooling

- **Validators** (CLI and open path) check OPC structure, required parts, manifest `format_version`, and `.unibrow-history` header/log consistency.
- **Hand repair:** folder mode allows fixing `project.json` in an editor; archive mode allows unzip → fix → re-zip with the same paths.
- **Export without history** and **clean before sharing** from [ADR 0031](0031-edit-history-as-file-format.md) apply to both shapes.

### 7. Embedded vs referenced large media

| Pattern | When |
| --- | --- |
| **Embed** | Default under size threshold; stored once under `assets/<hash>` |
| **Reference** | User chooses **Link** or asset above embed threshold; `refs/` entry with path + last-seen hash |
| **Collect into project** | Explicit command copies referenced bytes into `assets/` and updates graph handles; required before “single file” archive is safe to move |

Missing reference: banner with **Relink** / **Collect**; read-only if file missing and user declines — never delete graph nodes silently.

### 8. Relation to prior ADRs

| ADR | Update |
| --- | --- |
| **0015** | **Closed:** OPC zip is the canonical **single-file** shape; folder mode is the canonical **power-user** shape; same interior paths |
| **0030** | Core owns OPC IO, mode state, journal, atomic replace, deduped `assets/`, and `.unibrow-history` IO |
| **0031** | **Amended:** no `history/` directory or `manifest.lock`; pins are the header of `.unibrow-history` |

Foreign-format sidecars ([ADR 0031](0031-edit-history-as-file-format.md) §9) use the same `.unibrow-history` file format; this ADR governs **native** `.unibrow` and folder projects.

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes — modes visible; no auto folder switch; explicit pack/unpack |
| Never blocks save or work | Yes — view only + Save As; archive journal does not refuse flush |
| Local-first | Yes — standard zip and plain files; no cloud |
| Predictable and consistent | Yes — same three modes and commands everywhere native projects open |
| Simple durable mechanism | Yes — OPC zip, one history file, content hashes, atomic rename |
| Mod not core | Yes — graph and clip ops stay in plugins; core owns container and modes |

**Product fit:** A non-technical parent gets one `.unibrow` file and **Archive** mode without managing a project tree. A Vegas-style editor gets **Folder** mode and references. A recipient can **View only** without side effects. Everything remains inspectable with ordinary tools — **it just works** without hidden magic.

## Consequences

**Positive**

- One mental model: same paths in zip or folder; one history file for pins and ops.
- Deduped assets shrink mixed projects.
- Clear consent path builds trust; support can read `.unibrow-history` or unzip the archive.
- Journal + atomic replace matches sacred save and history append.

**Tradeoffs**

- Full zip rewrite on large archive save remains expensive; folder mode is the escape hatch.
- Users must learn three mode labels once (always visible).
- OPC + mixed compression adds implementation and test surface.
- Pack/unpack are explicit steps; no silent sync between duplicate paths.
- Pin header rewrites require careful atomic IO on `.unibrow-history`.

## References

- [architecture.md](../architecture.md) — project package sketch
- [unibrow-format-migration.md](../unibrow-format-migration.md) — `format_version` in manifest / `project.json`
- [ux.md](../design/ux.md) — project storage mode UX
- [status-bar.md](../design/status-bar.md) — slot model
- [ui-implementation-plan.md](../design/ui-implementation-plan.md) — PKG-* tickets
