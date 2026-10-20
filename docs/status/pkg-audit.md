# PKG-01..08 audit (2026-10-20)

Scope: `.unibrow` OPC package, folder vs archive storage modes, unpack/pack, journal save, refs/collect. Specs: `docs/design/ui-implementation-plan.md` Phase L, ADR 0031/0032, `docs/design/ux.md` storage section.

Baseline commit audited: `c391334`. PKG-01/PKG-02 implemented on branch through OPC writer, unpacked layout, legacy zip migration.

## Summary

| Ticket | Status | Evidence |
|--------|--------|----------|
| PKG-01 | done | `pkg01_exported_zip_has_opc_content_types_and_rels`, `pkg01_export_has_manifest_and_project_json`, `pkg01_png_in_zip_uses_store_not_deflate`, `pkg01_identical_asset_bytes_are_content_hash_deduped`, `pkg01_zip_has_unibrow_history_part`, `pkg01_zip_opens_in_python_zipfile`, `pkg01_archive_folder_archive_parts_lossless`, `pkg01_legacy_plain_zip_opens_and_upgrades_on_save`, `pkg01_zip_roundtrip_preserves_project_json` |
| PKG-02 | done | `pkg02_unpacked_bundle_has_manifest_and_project_json`, `pkg02_create_package_writes_unpacked_layout_not_plain_scaffold_only`, `pkg02_share_import_writes_unpacked_layout`, `pkg02_native_package_uses_root_history_not_per_file_sidecar`; `folder_project_uses_per_file_sidecar_not_package_root` (loose foreign files) |
| PKG-03 | missing | `pkg03_opening_zip_materialises_cache_on_disk`, `pkgStorageAudit.test.mjs` (no View only / Archive / Folder session mode) |
| PKG-04 | missing | `pkgStorageAudit.test.mjs` (no history path status copy); status bar has document editor modes only |
| PKG-05 | partial | `pkg05_pack_from_directory_does_not_touch_sibling_plain_folder`, `zip_export_and_materialize` (lib export/import); no UI unpack/pack (`pkgStorageAudit.test.mjs`) |
| PKG-06 | missing | `pkgStorageAudit.test.mjs` (no threshold settings keys) |
| PKG-07 | missing | no archive edit journal or atomic in-zip save pipeline in `unibrowser-project` |
| PKG-08 | partial | `pkg08_refs_directory_exists_but_no_collect_api`; assets are content-addressed on embed (`add_asset_copy`) but Collect API still absent |

## PKG-01: OPC archive layout and compression

**Works today**

- `.unibrow` exports as OPC zip: `[Content_Types].xml`, `_rels/.rels`, root `manifest.json`, `project.json`, `.unibrow-history`, content-addressed `assets/<sha256>` (`opc_bundle.rs`, `export_package_to_zip`).
- PNG and other compressed media use STORED; JSON, XML, and `.unibrow-history` use DEFLATE.
- Identical asset bytes dedupe to one `assets/` part; archive opens in stock unzip and Python `zipfile`.
- Legacy ADR 0015 plain zip (root `project.json` only) still opens; save/export upgrades to OPC (`pkg01_legacy_plain_zip_opens_and_upgrades_on_save`).
- Archive → materialize → re-export compares equal OPC part maps (`pkg01_archive_folder_archive_parts_lossless`).

## PKG-02: Folder mode on-disk IO

**Works today**

- Unpacked §2b layout: `manifest.json`, `project.json`, `.unibrow-history`, `assets/`, `refs/`, `derived/` via `create_package` / `write_manifest` / share import.
- `is_unpacked_native_package` detects root `manifest.json` + `project.json`.
- Native package edits append history to package-root `.unibrow-history` (`history_storage_path`); foreign loose files keep `<file>.unibrow-history` sidecars.
- Plain directories without `project.json` are not packages (`pkg02_plain_directory_without_project_json_is_not_a_package`).

**Gaps (later tickets)**

- Plain-folder §2a session (only `.unibrow-history`, no `project.json`) is not wired in the desktop shell; `create_package` still scaffolds a full §2b tree for explicit new projects.
- No `manifest.json`-only open path; both manifest and `project.json` are required for unpacked detection.

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
- Pack builds OPC with history; unpack is still `materialize_zip_package` to cache, not user-chosen §2b tree.

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
- Embed path uses content hashes; Collect and relink still unimplemented.

## Commands run

```bash
cargo test -p unibrowser-project --test pkg_audit
cargo test -p unibrowser-project
cargo test -p unibrowser-document-model
cd crates/unibrowser-desktop && node --experimental-strip-types --test tests/pkgStorageAudit.test.mjs tests/firstSaveDialog.test.mjs
```

`cargo test --workspace` requires `npm run build` in `crates/unibrowser-desktop` (Tauri frontend stamp) and `rustup target add wasm32-unknown-unknown` for `unibrowser-mod-host` wasm tests in this VM.
