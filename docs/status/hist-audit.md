# HIST-01..06 audit (2026-10-20)

Scope: `.unibrow-history` op log, undo/redo from disk head, History panel, folder vs archive layout, foreign sidecars, squash/clean. Specs: `docs/design/ui-implementation-plan.md` Phase K, ADR 0031/0032, `docs/design/history-panel.md`.

## Summary

| Ticket | Status | Evidence |
|--------|--------|----------|
| HIST-01 | done (markdown sidecar scope) | `foreign_markdown_sidecar_name_and_log_marker`, `append_increments_seq_and_survives_reopen`, `truncated_tail_line_is_ignored_on_open`, `folder_project_uses_per_file_sidecar_not_package_root`, `hist01_text_replace_replays_from_log` |
| HIST-02 | done (markdown host) | `undo_after_reopen_restores_prior_snapshot`, `redo_after_undo_from_history_head_restores_latest`, `hist02_host_undo_redo_matches_disk_head`, `hist02_history_list_reads_sidecar`, `hist02_invert_text_replace_roundtrip`, `hist02_jump_appends_snapshot_preserving_prior_ops`, `hist02_history_jump_returns_restored_markdown` |
| HIST-03 | missing | `hist03_pin_header_not_rewritten_for_new_plugin_version`; no missing-plugin banner in desktop UI |
| HIST-04 | done | `historyPanelBehavior.test.mjs`, `historyPanelEntries.test.mjs`, `doc-history-panel` / `doc-history-jump` capture; jump-to-seq appends snapshot; grouped typing rows; labels and timestamps |
| HIST-05 | missing | `hist05_docx_sidecar_path_is_nested_extension`; no export-without-history or docx save sidecar wiring |
| HIST-06 | done | `hist06_truncate_head_keeps_ops_up_to_seq`, `hist06_trim_before_keeps_from_seq_onward`, `hist06_squash_range_replaces_span_with_snapshot`, `historyCommands.test.mjs`; palette commands `unibrowser.history.trimBefore`, `unibrowser.history.squashSelection` with inline confirm |

## HIST-01: History file layout and atomic append

**Works today**

- Foreign markdown edits append JSONL ops to a sibling `*.md.unibrow-history` with pin header and `--- UNIBROW HISTORY LOG ---` marker (`history_file.rs`, `documentEditorHost` snapshot append on WYSIWYG commit).
- Append rewrites the whole file via `.part` + rename (atomic replace, not true append-only).
- Re-open lists the same ops (`append_increments_seq_and_survives_reopen`).
- Truncated final log line is dropped on read (`read_header_and_ops`).
- `base-text.text.replace` and `base-text.mark.toggle` replay through `apply_history_op`.

**Gaps (deferred)**

- No `.unibrow-history` inside `.unibrow` archives (`unibrowser-project` has no history part).
- Package-root `.unibrow-history` for folder projects (ADR 0032 §2a) not wired.
- No simulated-crash UI; recovery is silent tail drop only.

## HIST-02: Host history API and undo/redo head

**Works today**

- Tauri: `document_history_append`, `document_history_list`, `document_history_append_snapshot`, `document_history_jump_to_seq`, `document_history_trim_before`, `document_history_squash_range`.
- Markdown `document_undo` / `document_redo` pop/push disk log via `try_undo_markdown_history` / `try_redo_markdown_history` when path ends with `.md`.
- Core replay: `session_at_history_head`, snapshot ops, `invert_history_op` for text.replace / mark.toggle / block style / opaque.
- Jump to seq appends a snapshot at head without deleting later log entries.

**Gaps (deferred)**

- In-memory ProseMirror undo still used for some paths; disk log is snapshot-heavy for WYSIWYG commits.
- Web host keeps history in RAM for demo undo, not durable sidecars.

## HIST-03: Pin header and missing-plugin banner

**Gaps**

- Header is static `default_header()`; plugin version on ops does not rewrite pins (`hist03_pin_header_not_rewritten_for_new_plugin_version`).
- No open-time replay failure banner or read-only mode for unknown plugins.

## HIST-04: History panel UI

**Works today**

- Bottom dock tab lists grouped entries with seq, local time, `plugin_id`, and summary label (`historyPanel.ts`, `historyPanelEntries.ts`).
- Enter on a row shows inline confirm; confirm runs jump-to-seq (new snapshot, log preserved).
- Demo seeds multiple ops for capture (`demoState` + `doc-history-panel`, `doc-history-jump`).

**Gaps (deferred)**

- Clean-before-sharing wizard (HIST-06 scope overlap with ADR §8) not in panel v1.

## HIST-05: Foreign sidecar and export without history

**Works today**

- Sidecar naming helper: `report.docx` → `report.docx.unibrow-history`.

**Gaps**

- `document_save_markdown_as_docx` writes `.docx` only; no sidecar creation or preference.
- No "Omit edit history" on export/save copy.

## HIST-06: Squash and clean-before-sharing

**Works today**

- Library `truncate_head`, `trim_history_before`, `squash_op_range`.
- Commands **Trim history before here** and **Squash selection** in the command registry; `historyCommands.ts` uses `showInlineConfirm` before calling Tauri APIs.

**Gaps (deferred)**

- Full clean-before-sharing wizard, `migrate_op`, and asset GC per ADR §8.

## Screenshots

- `docs/screenshots/history/history-panel-v4-1280.png`
- `docs/screenshots/history/history-jump-v4-1280.png`

## How to re-run evidence

```bash
cargo test -p unibrowser-document-model history_
cargo test -p unibrowser-desktop hist_audit
cd crates/unibrowser-desktop && node --experimental-strip-types --test tests/historyPanelBehavior.test.mjs tests/historyPanelEntries.test.mjs tests/historyCommands.test.mjs
./scripts/capture-history-panel.sh
```

Full workspace: `cargo test --workspace` and project `scripts/check-workspace.sh` as usual.
