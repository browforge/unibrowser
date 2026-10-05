# ADR 0015: Native project format (`.unibrow`)

- **Status:** Accepted (direction; format not implemented)
- **Date:** 2026-10-05

## Context

Unibrowser’s direction is to **mix arbitrary formats in one workspace** — half-finished PDF, a timeline, MIDI, notes, diagrams — without forcing everything into a single interchange file on save ([ADR 0011](0011-lazy-conversion-native-until-encode.md), [vision.md](../vision.md)).

Most NLEs and DAWs treat the **project file** as an app-specific edit graph (Vegas, Premiere, Resolve) that **does not** aim to be a universal container for unrelated document types. That is fine for a single-domain editor; it is **insufficient** for Unibrowser’s “drop literally anything on an infinite canvas / project” goal.

Bear’s analogy:

| Role | Illustration / video analogue | Unibrowser |
|------|------------------------------|------------|
| **Editable native project** | Adobe Illustrator `.ai` (layers, objects, non-destructive) | **`.unibrow`** — anything-goes project package |
| **Flattened delivery** | JPEG (pixels, lossy workflow end state) | **MP4, PNG, PDF export**, etc. — **export**, not the living project |

**Ongoing work is not “a Vegas project.”** It is a **first-class Unibrowser project** that **references or embeds** source assets in their **native or lazy-native** form until the user **exports** to a delivery format.

## Decision

### Goals

1. **Non-destructive by default** — importing a PDF, MIDI file, or DOCX into a project does **not** permanently flatten it unless the user chooses export or an explicit “bake” step ([ADR 0011](0011-lazy-conversion-native-until-encode.md)).
2. **Anything on the canvas** — spatial and temporal placement (infinite canvas, boards, timelines) with heterogeneous clip types in one project graph.
3. **Portable project package** — one user-visible file or bundle the user can copy, back up, and reopen; optional external references for huge media.
4. **Mod-extensible** — new clip/layer types register schema + serializers via mods; the host owns the **project shell** and **asset index**, not every format’s internals.
5. **Export is separate** — “Save project” ≠ “Export MP4”; export runs the conversion graph at full fidelity.

### File extension (decided)

Bear confirmed **2026-10-05**: the standard extension is **`.unibrow`** only (not `.uni`). Longer-than-three-letter extensions are acceptable on modern **Windows, Linux, and macOS**; the branded name avoids generic collisions (e.g. “uni”) and matches the product.

- **Directory bundle:** `MyProject.unibrow/` (package folder).
- **Single-file save:** `MyProject.unibrow` as a ZIP (or OPC) container with the same interior layout — one extension for both shapes; disambiguate by file type (directory vs archive) at open time.
- **MIME / UTI:** register a vendor type when shipping (e.g. `application/vnd.unibrowser.project+json` or zip-based subtype) — details TBD; extension is fixed.

### Non-goals (v1 format design)

- Replacing ZIP, PDF, or Office as interchange standards for other apps.
- Storing decrypted DRM content or violating upstream licences inside the bundle (licence metadata required per asset).
- Mandating cloud sync or accounts.

### Container design sketch (not final)

Evaluate three shapes; the product may ship **one primary** with a documented migration path.

| Shape | Pros | Cons | Likely use in Unibrowser |
|-------|------|------|---------------------------|
| **Directory bundle** (e.g. `MyShow.unibrow/` as a folder) | Easy debugging; partial copy; git-friendly text parts | macOS “folder looks like file” UX; user must not break inner layout | Strong candidate for **dev** and **power users** |
| **ZIP (OPC-style) single file** | Familiar (Office, VSIX); one attachment to email | Whole-file rewrite on save; large media inside zip | Strong candidate for **grandma “one file”** UX |
| **SQLite inside bundle** | Fast asset index, queries, undo metadata, thumbnails | Binary blob; harder hand-edit | **Index + metadata** inside directory or zip, not necessarily sole container |

**Working hypothesis (subject to spike):**

```
MyProject.unibrow/          # directory bundle; or MyProject.unibrow (ZIP) — same interior
  project.json              # version, schema, canvas/timeline graph, mod API version
  assets/                   # embedded copies (content-addressed or uuid names)
  refs/                     # sidecar pointers to external file:// paths (optional)
  derived/                  # preview proxies; regenerable; provenance in project.json
  meta/                     # thumbnails, search index (optional)
```

- **`project.json`** — graph of **nodes** (clips, notes, artboards, timelines) and **edges** (placement, z-order, time range). Each node has `type` (mod-defined), `native_ref` (handle into `assets/` or `refs/`), and `engines` used ([architecture.md](../architecture.md)).
- **Single-file variant** — same tree, ZIP container with `mimetype` or `META-INF/container.xml` (TBD) for OS association.

Host validates **project format version** and **minimum `unibrowser_api`** like mod manifests ([ADR 0006](0006-mod-manifest-and-versioning.md)).

### Embedded vs referenced assets

| Mode | When | Behaviour |
|------|------|-----------|
| **Embed** | Default for small/medium files user dropped into project | Byte copy (or mod-specific canonical blob) under `assets/`; survives move of `.unibrow` |
| **Reference** | User explicitly links folder of footage, or “link not copy” | `refs/` entry: absolute or project-relative path + last-seen hash; **missing file** UI in shell |
| **Derived** | Preview proxies ([ADR 0011](0011-lazy-conversion-native-until-encode.md)) | Under `derived/`; safe to delete and rebuild; `derived_from` + `purpose: preview \| export_staging` |

**Relink / collect assets** — power-user commands (FOSS path): gather references into bundle before archive.

### Relation to lazy conversion ([ADR 0011](0011-lazy-conversion-native-until-encode.md))

| ADR 0011 rule | Project format |
|---------------|----------------|
| Native-first model | Project graph stores **native handles**, not flattened frames |
| Preview vs export | **Preview** artifacts live in `derived/`; **export** writes **outside** the project to user-chosen paths (MP4, PDF, …) |
| Conversion at boundaries | Export job reads project graph → schedules **ConversionEdge** jobs → does not rewrite source assets in `assets/` |
| No silent flattening | Opening a project never replaces an embedded PDF with PNG unless user bakes |

### Project vs tab vs export

| Concept | Meaning |
|---------|---------|
| **Tab** | One `BrowserLocation` + backend session in the shell ([ADR 0009](0009-browser-as-unified-shell.md)) |
| **Project** | Persistent graph spanning many assets; may open as one tab (“project surface”) or spawn child tabs |
| **Export** | Deterministic output file(s) for delivery; may discard editability by design |

A PDF opened **only** as a tab is still a **file**; **Save As project** copies or references it into `.unibrow` when the user starts a mixed canvas.

### Infinite canvas / “anything goes”

- **Spatial** — artboards, freeform placement (diagrams, sticky notes, images, PDF pages as objects).
- **Temporal** — timeline lanes referencing the same `assets/` pool (video + audio + MIDI + PDF clips per [ADR 0011](0011-lazy-conversion-native-until-encode.md)).
- **Mods** declare node types (`PdfClip`, `MidiClip`, `MarkdownNote`, …) and serialization; host provides **layout**, **selection**, **undo scope** (open question).

### Licensing inside the bundle

- Each embedded asset records **source licence** and **origin** (user file, mod template, export from another app).
- AGPL/GPL tools ([ADR 0007](0007-licensing.md)) produce **derived** blobs via **T3**; project stores **pointers** to those artifacts, not statically linked code.

## Consequences

- Roadmap needs a **project mod / host project service** phase before “real” mixed-format timelines ship.
- Mod API will need **project node registration** and **serialize/deserialize** hooks (future ADR or extension to 0006).
- ADR 0011 open question on **global vs per-project cache** → default **per-project `derived/`** with optional global cache override.
- File association and Mod Portal can advertise “creates `.unibrow` projects”.

## Open questions

1. **Single file vs bundle default** — ZIP `*.unibrow` for everyday save vs directory `*.unibrow/`; “Reveal package contents” for support?
2. **Schema versioning** — `project.json` `format_version` vs semver; forward-compatible readers?
3. **Undo / collaboration** — operation log in SQLite vs event-sourced JSONL in bundle?
4. **Encryption** — optional project password (out of scope v1)?
5. **Which mod owns the project surface** — host built-in `base-project` vs community timeline mod?
6. **Git / diff** — text-first `project.json` + avoid binary noise in `assets/` for LFS workflows?

**Closed (extension):** **`.unibrow`** only — see **File extension (decided)** above.

## Related

- [ADR 0011](0011-lazy-conversion-native-until-encode.md) — lazy conversion
- [ADR 0009](0009-browser-as-unified-shell.md) — tabs and locations
- [architecture.md](../architecture.md) — engines and multi-engine mods
- [roadmap.md](../roadmap.md) — project format phase
