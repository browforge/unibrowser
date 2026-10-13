# ADR 0035: One document model — WYSIWYG keystone and format serialisers

- **Status:** Accepted
- **Date:** 2026-10-13
- **Relates to:** [ADR 0028](0028-platform-content-kinds-and-plugin-scope.md), [ADR 0030](0030-core-as-skeleton.md), [ADR 0031](0031-edit-history-as-file-format.md), [ADR 0034](0034-compact-text-toolbar-and-semantic-styles.md)
- **Research:** [markdown-editing-modes.md](../research/markdown-editing-modes.md)

## Context

Markdown and Word-style documents today tend to fork into different editors: a code surface for `.md`, a separate rich canvas for `.docx`, and preview panes that can disagree with what gets saved. Users think in terms of **writing a document**, not fighting format wars.

Bear’s direction:

1. Offer **three editing modes** like Obsidian, Typora, Mark Text, Zettlr, and Notion: **plain source**, optional **split preview**, and **WYSIWYG** as the keystone default for new users.
2. **One shared rich-text content model** backs every mode and every host format. Blocks (paragraph, heading, list, quote, code, table, image) and marks (bold, italic, link, …) are the truth. Whether the file is `.md` or `.docx` is only **how it is serialised** on save, plus native vs emulated semantics per [ADR 0028](0028-platform-content-kinds-and-plugin-scope.md) and [ADR 0034](0034-compact-text-toolbar-and-semantic-styles.md).
3. **Never silently rewrite** the user’s Markdown style. Unedited regions must round-trip **byte-identical**. Unknown constructs become **opaque blocks**, not deletions.

Research summary: [markdown-editing-modes.md](../research/markdown-editing-modes.md).

## Decision

### 1. User-facing modes (per open file)

| Mode | User intent | Surface | Editable |
| --- | --- | --- | --- |
| **WYSIWYG** | Write like a document | Rich canvas on shared model | Yes (default for new profiles) |
| **Source** | See and edit exact text | Serialised text for active format plugin (Markdown, OOXML fragment, plain) | Yes |
| **Split** | Source + reassurance | Source pane + live preview of same model | Yes in source; preview follows |
| **Reading** | Review without accidents | Typography projection of model | No (toggle out to edit) |

Rules:

- Mode is **per tab**, persisted in session layout; **default mode** is a user preference (WYSIWYG for new users; power users may default to Source).
- Switching mode **does not change document meaning** and must not run a lossy format converter (contrast Toast UI dual-engine issues in research).
- **Reading** is not a separate Markdown engine with different rules; it renders the same model as WYSIWYG with editing chrome removed.

UX detail: [document-editor-modes.md](../design/document-editor-modes.md).

### 2. Canonical document model (logical)

Core defines a **versioned schema id** `unibrowser.document.model` (JSON schema published with mod API). Plugins map host files ↔ model; core does not embed Markdown or OOXML semantics ([ADR 0030](0030-core-as-skeleton.md)).

**Block types** (minimum v1):

| Block id | Role |
| --- | --- |
| `paragraph` | Body text |
| `heading` | `level` 1–6 |
| `blockquote` | Quoted block |
| `code_block` | Fenced or indented (metadata carries original fence style when known) |
| `bullet_list` / `ordered_list` / `task_list` | List containers |
| `list_item` | Item with optional task checked state |
| `table` | Rows/cells (colspan metadata when native host supports it) |
| `image` | Resource ref + alt/title |
| `horizontal_rule` | Thematic break |
| `opaque` | Unknown or raw host slice (see §4) |

**Marks** (inline, attach to text): `bold`, `italic`, `strikethrough`, `code`, `link`, `underline`, `subscript`, `superscript`, `text_color` (where host allows), with capability/emulation per ADR 0034.

**Document metadata:** title, optional front matter map (when host supports), `host_format`, `serializer_plugin_id`.

The model is the object **undo**, **proofing**, **AI diff**, and **command targets** use. Content-kind nodes in [ADR 0028](0028-platform-content-kinds-and-plugin-scope.md) expose `unibrowser.content.rich_text` subtrees backed by this model.

### 3. Serialisers as plugins (ADR 0030 skeleton)

Per [ADR 0030](0030-core-as-skeleton.md), core owns:

- Model schema registry and validation hooks
- Mode chrome and command routing
- Save pipeline orchestration ([ADR 0023](0023-saving-is-sacred.md))
- History append ([ADR 0031](0031-edit-history-as-file-format.md))

**Format plugins** (e.g. `base-text` Markdown, `base-docx`) implement:

| Hook | Responsibility |
| --- | --- |
| `parse(bytes) → { model, source_map }` | Build model; record **per-block source spans** in the original file |
| `serialize(model, source_map, options) → bytes` | Write file; apply round-trip rules (§4) |
| `capability_matrix` | ADR 0034 semantic support / emulation per block and mark |
| `opaque_policy` | Which foreign syntax becomes `opaque` vs parsed |

`source_map` entries: `{ block_id, byte_start, byte_end, digest }` for top-level blocks (and nested where needed for lists/tables). Core stores `source_map` in session and in history snapshot payloads when the host format is text-linear (Markdown).

No plugin may register a second independent WYSIWYG stack for the same capability; they plug serialisers into the shared editor host.

### 4. Round-trip rule (normative)

When saving or exporting to a **linear text** format (chiefly Markdown):

1. **Unchanged blocks:** If the model block’s semantic content and children equal the parse snapshot **and** the user did not edit inside its span, emit **`source_map` bytes verbatim** for that span.
2. **Edited or new blocks:** Serialize through the plugin’s **canonical** writer for that block type (may normalize cosmetics **only** for that block).
3. **Deleted blocks:** Omit span; adjacent untouched blocks still copy verbatim.
4. **Unknown / unsupported:** Preserve as **`opaque`** block holding raw text (and optional `language` hint). WYSIWYG shows a labelled raw card; Source shows exact bytes; Reading may hide or grey per product policy (default: show as preformatted with label).
5. **No silent rewrite:** Global reformat on save, “normalize document”, or mode switch is **opt-in** (explicit command with preview diff).

**Front matter:** Parsed into `metadata`; serialiser writes back using original delimiter style when untouched.

**Emulation:** When WYSIWYG applies a mark the host cannot persist natively, ADR 0034 emulation markers apply; serializer emits agreed escape (e.g. `<u>`) and records emulation in op payload.

### 5. Mode switching mechanics

All modes bind to **one in-memory model** + optional **source buffer view**:

- **WYSIWYG:** ProseMirror-compatible view (implementation in `base-text` / shared editor mod); transactions update model; `source_map` invalidates per touched block.
- **Source:** Text buffer is a **projection** of `serialize(preview=true)` or the concatenation of mapped spans; on commit, plugin **re-parse** merges into model (conflicts surface inline, not silent overwrite).
- **Split:** Same as Source with synchronized scroll and selection mapping; preview pane is read-only render of model.
- **Reading:** Same render as WYSIWYG without input; mode toggle returns to last edit mode.

Switching **WYSIWYG → Source** must not run remark stringify over the whole file if untouched blocks exist (use span copy). Switching **Source → WYSIWYG** parses dirty buffer; opaque regions survive.

### 6. Edit history (ADR 0031)

History stores **model ops**, not raw keystrokes in Monaco.

| Op style | Example `op_type` | Payload |
| --- | --- | --- |
| Structural | `base-text.block.set_style` | `block_id`, `style`, `emulation_note?` |
| Text | `base-text.text.replace` | `block_id`, `from`, `to`, `text`, `marks[]` |
| Mark | `base-text.mark.toggle` | `block_id`, `range`, `mark` |
| Opaque | `base-text.opaque.replace` | `block_id`, `raw` |
| Snapshot | `unibrowser.history.snapshot` | Model + `source_map` blob ref |

Core appends to `.unibrow-history` unchanged ([ADR 0031](0031-edit-history-as-file-format.md)). Plugins implement `apply` / `invert` on model + map. **Mode changes are not ops** (UI state only). Saving flushes model through serializer; autosave appends ops already committed in the session.

For foreign files with sidecar history (`report.md.unibrow-history`), the same op types apply; authoritative bytes remain the `.md` file unless user opts into wrapper format.

### 7. Toolbar and heading picker (ADR 0034)

[ADR 0034](0034-compact-text-toolbar-and-semantic-styles.md) semantic controls target the **document model**, not a specific view:

- `unibrowser.semantic.block_style` mutates block type in the model; serializer chooses `#` vs named style.
- Inline toggles mutate marks on the model selection.
- **Identical** in WYSIWYG, Source (when selection maps to ranges), and Split; disabled states come from capability resolver for the **host format**, not from mode.
- Reading mode hides the compact toolbar or shows it disabled with “Switch to edit”.

Command Manager strip (History, Find, Proofing, Export) unchanged.

### 8. Native vs emulated hosts

| Host | Model authority | Serialiser notes |
| --- | --- | --- |
| `.md` | Model + `source_map` | Round-trip rule §4 |
| `.docx` | Model | OOXML via `base-docx`; no `source_map` byte rule (binary format) |
| `.unibrow` rich text nodes | Model in package | History in `.unibrow-history` per [ADR 0032](0032-unibrow-container-archive-and-folder-modes.md) |
| Plain `.txt` | Single `paragraph` or opaque only | Source mode only; WYSIWYG allowed but no heading semantics (ADR 0034) |

### 9. Non-goals (1.0)

- Real-time collaborative editing (unchanged from ADR 0031).
- Guaranteed cosmetic preservation **inside** a block the user edited (only semantic preservation required there).
- Pandoc-complete Markdown; plugins declare supported flavour (GFM baseline).

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Opaque blocks and byte-stable untouched regions |
| Never blocks save | Serializer errors become Save As / export with banner, not data deletion |
| Local-first | Model and map live on disk in file + sidecar |
| Predictable across formats | Same semantics; serialiser differs |
| Mod not core | Markdown/DOCX writers in `base-*`; core owns schema and modes |

## Consequences

**Positive**

- One learning curve for “document editing” across Markdown and Word.
- WYSIWYG default matches Typora/Notion expectations; developers keep Source.
- History and AI operate on stable structure.

**Negative**

- `source_map` memory and merge logic add complexity.
- Source mode re-parse must be fast and test-heavy.
- Existing Monaco-only text path migrates behind `base-text` provider ([ADR 0030](0030-core-as-skeleton.md)).

## References

- [markdown-editing-modes.md](../research/markdown-editing-modes.md)
- [document-editor-modes.md](../design/document-editor-modes.md)
- [text-toolbar.md](../design/text-toolbar.md)
- [ui-implementation-plan.md](../design/ui-implementation-plan.md) (DOC-01–DOC-10)
