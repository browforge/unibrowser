# HIST-01..06 audit (2026-10-20)

Scope: `.unibrow-history` op log, undo/redo from disk head, History panel, folder vs archive layout, foreign sidecars, squash/clean. Specs: `docs/design/ui-implementation-plan.md` Phase K, ADR 0031/0032, `docs/design/history-panel.md`.

## Summary

| Ticket | Status | Evidence |
|--------|--------|----------|
| HIST-01 | partial | `foreign_markdown_sidecar_name_and_log_marker`, `append_increments_seq_and_survives_reopen`, `truncated_tail_line_is_ignored_on_open`, `folder_project_uses_per_file_sidecar_not_package_root` |
| HIST-02 | partial | `undo_after_reopen_restores_prior_snapshot`, `redo_after_undo_from_history_head_restores_latest`, `hist02_host_undo_redo_matches_disk_head`, `hist02_history_list_reads_sidecar` |
| HIST-03 | missing | `hist03_pin_header_not_rewritten_for_new_plugin_version`; no missing-plugin banner in desktop UI |
| HIST-04 | partial | `historyPanelBehavior.test.mjs`; capture demo `doc-history-panel` (Playwright). List only; no rollback/squash/jump |
| HIST-05 | missing | `hist05_docx_sidecar_path_is_nested_extension`; no export-without-history or docx save sidecar wiring |
| HIST-06 | partial | `hist06_truncate_head_keeps_ops_up_to_seq`; no user-facing squash/clean commands |

## HIST-01: History file layout and atomic append

**Works today**

- Foreign markdown edits append JSONL ops to a sibling `*.md.unibrow-history` with pin header and `--- UNIBROW HISTORY LOG ---` marker (`history_file.rs`, `documentEditorHost` snapshot append on WYSIWYG commit).
- Append rewrites the whole file via `.part` + rename (atomic replace, not true append-only).
- Re-open lists the same ops (`append_increments_seq_and_survives_reopen`).
- Truncated final log line is dropped on read (small fix in `read_header_and_ops`).

**Gaps**

- No `.unibrow-history` inside `.unibrow` archives (`unibrowser-project` has no history part).
- Folder projects use per-file sidecars, not package-root `.unibrow-history` (ADR 0032 §2a).
- No simulated-crash UI; recovery is silent tail drop only.

## HIST-02: Host history API and undo/redo head

**Works today**

- Tauri: `document_history_append`, `document_history_list`, `document_history_append_snapshot`.
- Markdown `document_undo` / `document_redo` pop/push disk log via `try_undo_markdown_history` / `try_redo_markdown_history` when path ends with `.md`.
- Core replay: `session_at_history_head`, snapshot ops.

**Gaps**

- `invert_history_op` unimplemented; non-snapshot ops cannot replay (`apply_history_op` errors on text.replace).
- In-memory ProseMirror undo still used for some paths; disk log is snapshot-heavy.
- Web host keeps history in RAM for demo undo, not durable sidecars.

## HIST-03: Pin header and missing-plugin banner

**Gaps**

- Header is static `default_header()`; plugin version on ops does not rewrite pins (`hist03_pin_header_not_rewritten_for_new_plugin_version`).
- No open-time replay failure banner or read-only mode for unknown plugins.

## HIST-04: History panel UI

**Works today**

- Bottom dock tab renders seq/time/plugin/op_type from `listDocumentHistory` (`historyPanel.ts`).
- Demo seeds ops for capture (`demoState` + `doc-history-panel`).

**Gaps**

- No rollback to seq, squash/clean links, keyboard jump, or inline confirm (`historyPanelBehavior.test.mjs`).

## HIST-05: Foreign sidecar and export without history

**Works today**

- Sidecar naming helper: `report.docx` → `report.docx.unibrow-history`.

**Gaps**

- `document_save_markdown_as_docx` writes `.docx` only; no sidecar creation or preference.
- No "Omit edit history" on export/save copy.

## HIST-06: Squash and clean-before-sharing

**Works today**

- Library `truncate_head` for tests.

**Gaps**

- No palette/panel commands, user confirm, `migrate_op`, or clean wizard.

## How to re-run evidence

```bash
cargo test -p unibrowser-document-model history_
cargo test -p unibrowser-desktop hist_audit
cd crates/unibrowser-desktop && node --experimental-strip-types --test tests/historyPanelBehavior.test.mjs
```

Full workspace: `cargo test --workspace` and project `scripts/check-workspace.sh` as usual.
