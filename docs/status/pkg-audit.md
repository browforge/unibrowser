# PKG-01..08 audit (2026-10-20)

Scope: `.unibrow` OPC package, folder vs archive storage modes, unpack/pack, journal save, refs/collect. Specs: `docs/design/ui-implementation-plan.md` Phase L, ADR 0031/0032, `docs/design/ux.md` storage section.

Baseline commit audited: `da8cd06`.

## Summary

| Ticket | Status | Evidence |
|--------|--------|----------|
| PKG-01 | missing | `pkg01_exported_zip_lacks_opc_content_types`, `pkg01_export_has_project_json_not_manifest_json`, `pkg01_png_in_zip_uses_deflate_not_store`, `pkg01_identical_asset_bytes_are_not_content_hash_deduped`, `pkg01_zip_has_no_unibrow_history_part`, `pkg01_zip_roundtrip_preserves_project_json` (directory zip only) |
| PKG-02 | partial | `pkg02_unpacked_bundle_uses_project_json_at_root`, `pkg02_plain_directory_without_project_json_is_not_a_package`, `pkg02_create_package_scaffolds_full_tree_not_plain_folder`, `pkg02_share_import_writes_unpacked_layout`; `folder_project_uses_per_file_sidecar_not_package_root` (document-model) |
| PKG-03 | missing | `pkg03_opening_zip_materialises_cache_on_disk`, `pkgStorageAudit.test.mjs` (no View only / Archive / Folder session mode) |
| PKG-04 | missing | `pkgStorageAudit.test.mjs` (no history path status copy); status bar has document editor modes only |
| PKG-05 | partial | `pkg05_pack_from_directory_does_not_touch_sibling_plain_folder`, `zip_export_and_materialize` (lib export/import); no UI unpack/pack (`pkgStorageAudit.test.mjs`) |
| PKG-06 | missing | `pkgStorageAudit.test.mjs` (no threshold settings keys) |
| PKG-07 | missing | no archive edit journal or atomic in-zip save pipeline in `unibrowser-project` |
| PKG-08 | missing | `pkg08_refs_directory_exists_but_no_collect_api`; `add_asset_copy` copies by filename, not content hash |

## PKG-01: OPC archive layout and compression

**Works today**

- A directory bundle with `project.json` exports to a `.unibrow` zip and round-trips via `export_package_to_zip` / `materialize_zip_package` / `open_package` (`pkg01_zip_roundtrip_preserves_project_json`, `zip_export_and_materialize` in `lib.rs`).

**Gaps**

- Zip is a flat archive of project files, not OPC: no `[Content_Types].xml`, no `_rels/.rels`, no root `manifest.json` (ADR 0032 §1).
- All parts use DEFLATE, including PNG (`pkg01_png_in_zip_uses_deflate_not_store`).
- Assets keep original filenames under `assets/`; identical bytes produce two files (`pkg01_identical_asset_bytes_are_not_content_hash_deduped`).
- No `.unibrow-history` part inside the zip (`pkg01_zip_has_no_unibrow_history_part`, also noted in `hist-audit.md`).

## PKG-02: Folder mode on-disk IO

**Works today**

- Unpacked **ADR 0015** layout: `project.json`, `assets/`, `refs/`, `derived/` via `create_package` and `open_package` (`pkg02_unpacked_bundle_uses_project_json_at_root`).
- Arbitrary folders without `project.json` are not opened as packages (`pkg02_plain_directory_without_project_json_is_not_a_package`).
- Share zip import expands to a directory bundle (`pkg02_share_import_writes_unpacked_layout`).

**Gaps**

- ADR 0032 **plain folder** (§2a): no package scaffold on first edit; only `.unibrow-history` at project root. Today `create_package` writes full tree immediately (`pkg02_create_package_scaffolds_full_tree_not_plain_folder`).
- Package-root `.unibrow-history` for native folder projects not wired (`folder_project_uses_per_file_sidecar_not_package_root`).
- No `manifest.json` detection for unpacked §2b (field does not exist in codebase).

## PKG-03: Storage mode state (view / archive / folder)

**Gaps**

- No session flag or backend routing for view-only vs archive vs folder.
- Opening a `.unibrow` zip always materialises to a cache directory (`pkg03_opening_zip_materialises_cache_on_disk`), so read path writes disk before any edit.
- First-save dialog offers archive vs folder for **new** projects only; not the ADR mode prompt on open (`firstSaveDialog.test.mjs`).

## PKG-04: Status bar mode indicator

**Gaps**

- No i18n or status contributions for `View only`, `Archive`, `Folder`, or “History will be saved in …” (`pkgStorageAudit.test.mjs`).
- Document editor Visual/Source/Split/Read chips are unrelated (`documentEditorModes.ts`).

## PKG-05: Unpack to folder / Pack as .unibrow

**Works today**

- Library can zip a directory bundle and import a share zip (`export_package_to_zip`, `import_share_package_zip`).
- Exporting a package does not modify unrelated plain folders (`pkg05_pack_from_directory_does_not_touch_sibling_plain_folder`).

**Gaps**

- No File menu or palette commands for unpack/pack (`pkgStorageAudit.test.mjs`).
- Pack does not build OPC or embed history; unpack is `materialize_zip_package` to cache, not user-chosen §2b tree.

## PKG-06: Large-project folder hint

**Gaps**

- No settings schema or status warning for size/asset threshold (`pkgStorageAudit.test.mjs`).

## PKG-07: Archive edit journal and atomic save

**Gaps**

- No sidecar journal beside `.unibrow`, no fold-into-zip on save, no crash recovery banner for archives.
- Project autosave copies `project.json` to `derived/project-autosave.json` only for directory bundles.

## PKG-08: Referenced media and Collect into project

**Gaps**

- `refs/` directory is created empty; no graph refs or Collect command (`pkg08_refs_directory_exists_but_no_collect_api`).
- Embedding uses `fs::copy` to `assets/<filename>`, not content-addressed store.

## Commands run

```bash
cargo test -p unibrowser-project --test pkg_audit
cargo test --workspace
cd crates/unibrowser-desktop && node --experimental-strip-types --test tests/pkgStorageAudit.test.mjs tests/firstSaveDialog.test.mjs
```
