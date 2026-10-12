# ADR 0031: Edit history as the file format

- **Status:** Accepted
- **Date:** 2026-10-12
- **Amends:** prior draft paths (`history/log`, `history/manifest.lock`) — superseded by single **`.unibrow-history`** ([ADR 0032](0032-unibrow-container-archive-and-folder-modes.md))
- **Relates to:** [ADR 0015](0015-native-project-format.md), [ADR 0023](0023-saving-is-sacred.md), [ADR 0030](0030-core-as-skeleton.md)

## Context

Today, undo for text and SVG uses an in-memory per-path stack in the desktop host ([commands-undo-redo.md](../commands-undo-redo.md)); `.unibrow` timeline edits snapshot `project.json` on discrete actions. History does not survive copy to another machine, cannot be browsed as a timeline of edits, and is separate from the on-disk project shape. [ADR 0015](0015-native-project-format.md) left “operation log in SQLite vs JSONL” as an open question.

Bear’s direction: for native Unibrowser documents (especially `.unibrow`, and any plugin-owned **native** save format), the **undo stack is the file**. The durable representation is an **append-only operation log** plus **periodic snapshots**, not a single mutable JSON blob that only implicitly encodes the latest state.

Goals:

- Undo and redo work after moving the file to another computer (replay with pinned plugins).
- Users can **inspect history** and roll back to an earlier point without a separate “versions” product.
- Room for future **peer-to-peer or self-hosted** sync of ops later; **live co-editing is a 1.0 non-goal** and Unibrowser will **never run a central collaboration server**.
- Crash safety: the log is a **crash journal**; atomic append preserves work.
- Foreign formats (`.docx`, `.png` on disk) keep the foreign bytes as the authoritative file; history lives in a **sidecar** or inside a **`.unibrow` wrapper** when the user opts in.

## Decision

### 1. Logical model

Each **history-bearing document** has **one history file** (canonical name **`.unibrow-history`**) that combines plugin pins and the edit log. There is no separate lockfile on disk.

| Part | Role |
| --- | --- |
| **Pin header** | Leading section of `.unibrow-history` — plugin ids, **exact versions**, and highest `op_schema_version` seen per `op_type` (see §3). Rewritten when a new pin is needed; not append-only |
| **Append log** | Same file, after a fixed **log marker** — committed **operations** (see §2) in order |
| **Snapshots** | Full materialised checkpoints referenced from the log (blob in `assets/<hash>` or inline for small state); not a separate `history/snapshots/` tree |
| **Head pointer** | Session state: latest applied `seq` (undo = move head backward; redo = forward) |

The **current document state** is: latest snapshot at or before head, then apply ops `(snapshot_index+1 … head]`. Saving does not discard history unless the user runs **squash** (§6).

**Fixed paths** ([ADR 0032](0032-unibrow-container-archive-and-folder-modes.md)):

| Container | History file path |
| --- | --- |
| OPC `.unibrow` archive | Package part **`.unibrow-history`** at the archive root (beside `manifest.json`) |
| Folder-mode project | **`.unibrow-history`** in the project folder (hidden file; see ADR 0032) |
| Foreign file on disk | Sidecar **`<filename>.unibrow-history`** next to the authoritative file (e.g. `report.docx.unibrow-history`) |

Archive and folder mode use the **same file format** for `.unibrow-history`; only the outer container differs.

### 2. Operation schema

Each log entry is one JSON object per line (**JSONL**) or length-prefixed record in a binary segment (implementation choice; JSONL is the reference for v1).

Required fields:

| Field | Meaning |
| --- | --- |
| `op_id` | Globally unique id for this op (UUID v4 or ULID) — stable across export/sync |
| `seq` | Monotonic integer **per document**; assigned at append time; no gaps after commit |
| `parent_seq` | Previous seq after this op applies (forms a DAG spine; branching reserved for future merge) |
| `timestamp` | RFC 3339 UTC — informational; **replay order uses `seq` only** |
| `plugin_id` | Mod that defined and emitted the op |
| `plugin_version` | Exact semver string used when the op was created |
| `op_type` | Namespaced string (`base-text.insert`, `base-pdf.annotate`, …) |
| `op_schema_version` | Positive integer — semver for the **payload shape** of `op_type` |
| `payload` | Plugin-defined JSON (or binary ref into `assets/` for large blobs) |

**Ordering rules (CRDT/OT-ready, no commitment yet):**

- Total order on committed ops is **`seq`** ascending.
- `op_id` is immutable identity for a logical edit (future: idempotent apply, merge).
- Concurrent editors are out of scope for 1.0; `parent_seq` is linear today but reserved so a later ADR can attach alternate parents without renaming fields.
- Core **does not interpret** `payload`; it stores, indexes, and forwards to plugins for apply/revert.

Plugins implement:

- `apply(op, state) -> state`
- `invert(op, state) -> op'` optional for undo (or replay from earlier snapshot)
- `migrate_op(op, from_schema, to_schema) -> op` for breaking payload changes (§5)

### 3. Pin header (merged into `.unibrow-history`)

Plugin version pins live in the **header** at the start of `.unibrow-history`, not in a separate `manifest.lock`. The header is human-readable YAML or JSON (like `pnpm-lock.yaml`), bounded in size, and **rewritten atomically** whenever an appended op introduces a new `plugin_id`, **exact** `plugin_version`, `op_type`, or `op_schema_version` not already recorded.

Example header (followed by a log marker line, then JSONL ops):

```yaml
historyFormat: 1
plugins:
  base-text@1.4.2:
    op_types:
      insert: 3
      delete: 2
  base-pdf@0.9.0:
    op_types:
      annotate: 1
--- UNIBROW HISTORY LOG ---
```

Pins are **exact versions** (no carets). On append, if the op’s plugin tuple requires a header change, the host rewrites the header (write `.unibrow-history.part`, fsync, rename) then appends the op. Opening a document **reads the header first**, then replays the log after the marker.

**Snapshots in the log:** a committed checkpoint is a normal op with `op_type: unibrowser.history.snapshot` and `payload` pointing at snapshot bytes (typically `assets/<content-hash>` per host format). Older snapshot blobs remain in `assets/` until squash or clean removes unreferenced hashes.

### 4. Missing or incompatible plugins

On open:

1. Materialise **latest snapshot** (read-only baseline).
2. Attempt replay from the following op until the first failure.

| Condition | Behaviour |
| --- | --- |
| Plugin not installed | Stop replay; open **read-only** at last good state. Banner: **Install `plugin_id` version `Y`** (portal link) or **Continue read-only**. Never delete or hide user data. |
| Plugin installed but wrong version | Offer **Install matching version** or **Migrate** if a registered migration path exists (§5). If migration declines, same read-only last-good state. |
| `op_schema_version` unsupported | Same; plugin may register migrator to current schema. |

**Save is sacred:** user can always **Save As** export of last materialised state (may be read-only session). Autosave writes log + snapshot per [ADR 0023](0023-saving-is-sacred.md); validation on open may be strict while save stays permissive for in-session edits that have not yet been flushed to foreign paths.

### 5. Migration across breaking op schemas

Breaking changes to `payload` shape bump `op_schema_version` for that `op_type`. Plugins ship **migration functions** registered with the host:

- `migrate_op(op, target_schema) -> op | error`
- Optional batch `migrate_log(snapshot, ops) -> ops'` for squash

The host records `migrated_from` metadata in a new append entry when the user chooses **Migrate document** (creates auditable op `unibrowser.history.migrate`). Migrations never silently drop ops; failed migration leaves read-only + diagnostic.

### 6. Squash history (user action)

Command: **Squash history into snapshot** (palette + History panel).

- Writes a new full snapshot at current head.
- Truncates log before that point (user confirmation explains undo depth loss).
- Rewrites the pin header to match remaining ops (current plugin versions only if ops were dropped).
- Does not remove `assets/` blobs still referenced by the snapshot.

Autosquash **optional** setting: when log byte size or op count exceeds threshold, prompt or auto-snapshot in background (default: prompt only).

### 7. File size and compaction

| Strategy | When |
| --- | --- |
| **Snapshots** | Every N ops or M megabytes of log (defaults TBD; user-tunable) |
| **Blob externalisation** | Large `payload` bodies stored content-addressed under `assets/` with hash in op |
| **Squash** | User-initiated or threshold prompt |
| **Zip re-pack** | Save compacts zip central directory; does not dedupe assets without explicit “optimize project” |

Deleted content remains in the log until squash or **clean history** (§8). Compaction is offline/local only.

### 8. Privacy and sharing

- **Deleted text and annotations remain in history** until squash or clean. Sensitive edits are recoverable from the file.
- **Clean history before sharing** — wizard: squash to single snapshot, strip log, optional strip old snapshots; explain what recipients can no longer undo.
- **Export without history** — Save Copy / Export delivers foreign format final state only, or `.unibrow` without the `.unibrow-history` part (explicit checkbox, default off for “full project”, on for “delivery bundle”).
- Activity is local; no telemetry of ops.

### 9. Foreign formats (`.docx`, `.png`, …)

| Pattern | History location |
| --- | --- |
| User saves directly to `file.docx` | Default: sidecar file **`file.docx.unibrow-history`** (same single-file format as native projects) |
| User saves into `.unibrow` project | **`.unibrow-history`** inside archive or beside `project.json` in folder mode ([ADR 0032](0032-unibrow-container-archive-and-folder-modes.md)); embedded asset may keep native bytes in `assets/` with ops referencing handles |
| User opts out | No history sidecar; in-memory undo only for session (current behaviour), with status bar note |

Foreign file bytes always reflect **latest materialised state** on Save to that path; replay is required to reconstruct intermediate states. Plugins that cannot emit ops fall back to snapshot-only history (one op = full file replace) with size warning.

### 10. Crash safety

- **Op append:** stage tail in `.unibrow-history.part` or length-checksummed record buffer; fsync; append to log section after marker (or atomic replace of whole file when header and tail must commit together).
- **Snapshot:** write snapshot bytes to `assets/<hash>.part`, fsync, rename, then append `unibrowser.history.snapshot` op to `.unibrow-history`.
- **Header rewrite:** same atomic replace pattern as §3; never leave a pin header half-written.
- On open, detect trailing partial record after marker; truncate or recover last complete op; surface non-modal “Recovered N edits after crash”.
- Autosave slots may hold duplicate tail of log; merge by `seq` on recovery.

### 11. UI and commands (core)

Core provides:

- `unibrowser.document.undo` / `unibrowser.document.redo` — move head; delegate invert/replay to plugins
- **History panel** — list ops (time, plugin, summary label from plugin); click to preview; rollback to seq ([history-panel.md](../design/history-panel.md))
- Read-only mode chrome when replay stops

Plugin provides: human `summary` on append optional; rich preview hooks.

### 12. Collaboration posture (future)

This format does not implement sync. It **enables** later designs:

- **OT:** ops ordered by `seq`; transform at append boundary per plugin negotiation.
- **CRDT:** `op_id` + plugin-defined payload merges; `parent_seq` may become multi-parent.

1.0: single writer, single machine. No server. Peer sync, if ever added, is explicit file/op exchange user controls.

### 13. Relation to ADR 0015 open question

**Closed:** prefer **JSONL log + snapshot files** inside the bundle over SQLite for the canonical history (SQLite may still index search in `meta/` as optional derived data). `project.json` becomes snapshot-shaped state at a point in time, not the sole source of truth for undo.

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes — visible history, explicit squash/clean, read-only not delete |
| Never blocks save or work | Yes — save appends; open degradation is read-only with install path |
| Local-first | Yes — no server; ops on disk |
| Predictable and consistent | Yes — same undo model for native formats; foreign formats documented |
| Simple durable mechanism | Yes — one history file, pin header + append log, atomic rename |
| Mod not core | Yes — op semantics in plugins; core stores and sequences |

## Consequences

**Positive**

- Portable undo; support can inspect `.unibrow-history` (header + log) without reproducing UI steps.
- Crash journal reduces lost work anxiety.
- Clear path to sync without redesigning save format.

**Tradeoffs**

- Larger files; users must learn squash/clean before sharing secrets.
- Pin header rigidity: old projects need matching mods or migration investment.
- Implementation spans host, every editing mod, and `.unibrow` IO.

## References

- [commands-undo-redo.md](../commands-undo-redo.md)
- [unibrow-format-migration.md](../unibrow-format-migration.md)
- [history-panel.md](../design/history-panel.md)
- [ui-implementation-plan.md](../design/ui-implementation-plan.md) — HIST-* tickets
