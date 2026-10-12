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

The **unpacked on-disk layout** (§2b) uses the same relative paths as the archive interior. A **plain user folder** (§2a) does not.

### 2. Folder mode: two on-disk shapes

**Folder mode** (§3) means the session is rooted at a directory path, not a `.unibrow` file. On disk there are only two allowed shapes.

#### 2a. Plain user folder (default for “open my project folder”)

The user opens an **ordinary directory** of their own files (`MyShow/`, a Vegas tree, a download folder). Unibrowser must **not** treat this as permission to scaffold a Unibrowser project on disk.

| Rule | Behaviour |
| --- | --- |
| **Open** | Never creates `manifest.json`, `project.json`, `assets/`, `refs/`, `derived/`, or any other Unibrowser package path |
| **First edit** | The **only** file Unibrowser may add is **`.unibrow-history`** (hidden), and **only** after the first mutating edit ([ADR 0031](0031-edit-history-as-file-format.md)) |
| **User files** | Existing media and documents stay where the user put them; the graph and refs point at real paths |
| **In-memory / log** | Project and undo state live in session memory and in `.unibrow-history`; not in a forced `project.json` on disk |

This is the Vegas-style case: one history sidecar, no takeover of the folder layout.

#### 2b. Unpacked package folder (explicit unpack or collect only)

`manifest.json`, `project.json`, `assets/`, `refs/`, and `derived/` appear on disk **only** when:

1. The user runs **Unpack to folder** from a `.unibrow` archive (writes the full interior beside user content), or
2. The user runs **Collect into project** and the operation materialises embedded assets and package metadata on disk (same path layout as the archive interior).

| Path (when present) | Role |
| --- | --- |
| `manifest.json` | Package manifest (format version, title, …) |
| `project.json` | Materialised project graph |
| **`.unibrow-history`** | Pin header + append log |
| `assets/`, `refs/`, `derived/` | Same as archive |

There is **no** hidden `.unibrow/` subtree. An unpacked folder is visibly a mix of the user’s files and Unibrowser package files they asked for.

**Open detection:**

| What the user opens | Treatment |
| --- | --- |
| `MyShow.unibrow` (file) | **Archive** mode (§3) unless converted |
| `MyShow/` with root `manifest.json` (Unibrowser `format_version`) | **Folder** mode, **unpacked** layout (§2b) |
| Any other folder | **Folder** mode, **plain** layout (§2a) — file browser plus optional native editing with at most `.unibrow-history` |

### 3. Three storage modes (always visible)

Every open native project session has exactly one **storage mode**. The UI must show it at all times (status bar; see [ux.md](../design/ux.md)).

| Mode | User label | Writes |
| --- | --- | --- |
| **1. View only** | **View only** | **Nothing** to disk — read archive or folder; in-memory display only; no `.unibrow-history`, no journal |
| **2. Work in archive** | **Archive** | Edits the **`.unibrow` file in place** without unpacking the whole project to a visible tree (Word-style `.docx` editing) |
| **3. Folder mode** | **Folder** | Work rooted at a directory: user files unchanged; **only** **`.unibrow-history`** may be added (after first edit). **`manifest.json`**, **`project.json`**, and **`assets/`** are written only after **Unpack to folder** or **Collect into project** (§2b) |

**View only** is the default when the user opens a `.unibrow` from an attachment, a read-only volume, or explicitly chooses “Open for viewing”. Save, autosave, and history append are disabled until the user picks an editing mode or **Save As** into a writable location.

**Work in archive** is the default for a writable `.unibrow` file when the user expects a single portable file.

**Folder mode** applies when the user opens a directory for native project work, or after **Unpack to folder**.

**Conversion is explicit only** — never silent:

| Command | Effect |
| --- | --- |
| **Unpack to folder** | User picks destination; writes **unpacked** layout (§2b): `manifest.json`, `project.json`, `assets/`, **`.unibrow-history`**, etc.; archive path recorded for optional “Pack back” |
| **Pack as .unibrow** | User picks output `.unibrow` path; builds OPC zip including **`manifest.json` inside the archive only**. **Never** writes `manifest.json`, `project.json`, or `assets/` into a **plain** source folder (§2a). Session graph + `.unibrow-history` + on-disk user files (and unpacked parts if §2b) are serialised into the zip |
| **Collect into project** | User-initiated; may create §2b paths on disk when embedding linked media into `assets/` and materialising graph state |

Switching modes mid-session may require save and a short confirmation when history or refs would change shape.

### 4. Consent and no surprises

Aligned with [principles.md](../principles.md): trust the user, predictable behaviour, local files they can inspect.

| Rule | Behaviour |
| --- | --- |
| **Open never creates files** | Opening a `.unibrow`, a **plain folder**, or a foreign file does **not** create `.unibrow-history`, journals, `manifest.json`, or other package paths |
| **First edit creates storage** | Archive: journal / in-zip history. **Plain folder:** create **only** **`.unibrow-history`**. **Unpacked folder:** append history and update on-disk `project.json` / `assets/` as the edit requires. Foreign files: sidecar per [ADR 0031](0031-edit-history-as-file-format.md) §9 |
| **Plain status** | Before and after first edit, status bar states where history will be saved, e.g. **History will be saved in `/path/to/file.unibrow`** (archive: inside the package) or **History will be saved in `/path/to/MyShow/.unibrow-history`** |
| **Large project hint** | When estimated unpacked size, embedded asset count, or largest single asset exceeds a **configurable threshold** (default TBD; e.g. hundreds of MB or many large clips), show a **non-blocking** hint: folder mode may be faster for this project — **Unpack to folder** link. **Never** auto-switch mode |
| **Threshold setting** | Settings → Projects → **Suggest folder mode when project exceeds** (size and/or asset count); off disables hints |

Save remains sacred ([ADR 0023](0023-saving-is-sacred.md)): view only does not trap edits — **Save As** or mode change always offers a writable path.

### 5. Work-in-archive: journal, atomic save, recovery

Editing a `.unibrow` zip without full unpack on every keystroke uses a **write journal** outside the zip:

1. While editing, new ops and blob writes go to a **temporary journal** beside the file (`<name>.unibrow~journal/`) or in the app cache keyed by file identity (user preference: “Keep recovery journal next to project”).
2. On **Save** / autosave flush, core **folds** journal changes into a new zip (including updated **`.unibrow-history`**) and **atomically replaces** the `.unibrow` file (write `*.part`, fsync, rename), same spirit as [ADR 0031](0031-edit-history-as-file-format.md) §10.
3. **Crash recovery** on next open: replay journal into the archive or offer restore; non-modal **Recovered N edits after crash** (no modal blocking Save).

**Tradeoff (documented honestly):** very large archives still pay a **full repack** on save unless the user works in **folder mode** on a **plain** folder, where only **`.unibrow-history`** grows on disk and media stays in the user’s existing paths. Unpacked `assets/` or **Collect into project** reintroduce on-disk blobs by explicit choice. The size hint (§4) steers without forcing.

### 6. Interior consistency and tooling

- **Validators** (CLI and open path) check OPC structure, required parts, manifest `format_version`, and `.unibrow-history` header/log consistency.
- **Hand repair:** unpacked folders (§2b) allow fixing `project.json` in an editor; plain folders have no `project.json` until unpack/collect; archive mode allows unzip → fix → re-zip.
- **Export without history** and **clean before sharing** from [ADR 0031](0031-edit-history-as-file-format.md) apply to both shapes.

### 7. Embedded vs referenced large media

| Pattern | When |
| --- | --- |
| **Embed** | Default under size threshold; stored once under `assets/<hash>` |
| **Reference** | User chooses **Link** or asset above embed threshold; `refs/` entry with path + last-seen hash |
| **Collect into project** | Explicit command copies referenced bytes into on-disk `assets/` (creating §2b layout if needed), updates graph handles; required before a portable single-file archive is safe when refs pointed outside the folder |

Missing reference: banner with **Relink** / **Collect**; read-only if file missing and user declines — never delete graph nodes silently.

### 8. Relation to prior ADRs

| ADR | Update |
| --- | --- |
| **0015** | **Closed:** OPC zip is the canonical **single-file** shape; **plain folder + `.unibrow-history`** is the default power-user shape; **unpacked** interior matches archive only after explicit unpack/collect |
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

**Product fit:** A non-technical parent gets one `.unibrow` file and **Archive** mode. A Vegas-style editor opens their existing folder and sees **at most one** new hidden history file after they edit — never a surprise `manifest.json`. Unpack and Collect are deliberate. A recipient can **View only** without side effects.

## Consequences

**Positive**

- One mental model: full package paths live in the zip or after explicit unpack/collect; plain folders stay the user’s tree plus optional `.unibrow-history`.
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
