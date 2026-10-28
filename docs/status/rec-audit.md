# REC-01 / REC-02 audit

Date: 2026-10-27. Base: `467bd3a`.

## REC-01 (crash recovery)

| Area | Status | Notes |
|------|--------|-------|
| Durable unsaved edit journal in app data dir | **missing** | No `edit-recovery` store; only opt-in full-buffer JSON under `crash-recovery/` |
| Append-only + fsync | **partial** | `package_save` journals folder saves in `.unibrow-history`; not used for loose-file / in-session edits |
| Clear journal on clean save / close | **missing** | Crash backups are dismissed manually; no journal lifecycle |
| Next-launch banner (non-modal) | **partial** | `recovery-bar-host` + `recoveryBar.ts`; auto-applies backup on launch via `crash_launch.rs` |
| Restore / Discard / Show changes + i18n | **partial** | Only View original + Discard; strings exist but no Show changes or Restore |
| Archive + folder + loose files | **partial** | Opt-in crash backup covers text/SVG/JS paths only when setting enabled |
| Torn tail tolerance | **missing** | N/A until framed journal exists |
| Rust tests (kill / torn / clean-save) | **missing** | |

### Existing pieces to extend

- `crash_recovery.rs`, `crash_launch.rs`, `recoveryBar.ts`, `RecoveryBarItemDto` in snapshot.
- `flush_document_autosave` / `sync_text_document` call sites for journaling hooks.
- `TEXT_SAVE` path in `commands.rs` for journal clear.
- `diffEditor.ts` + `file_compare` session state for Show changes.

## REC-02 (stable asset references)

| Area | Status | Notes |
|------|--------|-------|
| Explicit `asset_hash` on project graph | **missing** | `ProjectNode` only has `asset` (hash filename today, but GC does not treat it as explicit ref field) |
| Explicit `asset_hash` in history ops | **partial** | Test manually pushes `asset_hash` in snapshot payload; producers do not set it consistently |
| GC via explicit refs only | **missing** | `hashes_from_history_payload` scans all JSON strings matching sha256 shape |
| Migrate on load | **partial** | `migrate_assets_to_content_hashes` on save; no `asset_hash` backfill on open |
| Tests: keep referenced / drop orphan / decoy string | **partial** | PKG-08 tests history `asset_hash` but not decoy-string false positive |

### Existing pieces to extend

- `refs_collect.rs` (`referenced_asset_hashes_from_manifest`, `referenced_asset_hashes_from_history`, `gc_content_addressed_assets`).
- `add_asset_copy`, `collect_media_ref`, `MediaRefRecord.content_hash`.
- `pkg_audit.rs` PKG-08 tests.

## Planned work

1. `unibrowser-document-model::edit_recovery` — framed journal + tests.
2. Desktop `edit_recovery` host — always journal dirty edits; launch discovers pending; banner actions; clear on save.
3. `ProjectNode.asset_hash`, migration in `open_package`, narrow GC history walk to `asset_hash` only; REC-02 tests in `pkg_audit.rs` or `refs_collect` tests.
4. Capture script + `docs/screenshots/rec/*.png` with pixel gate.
