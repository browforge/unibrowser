# Browbridge repository layout (split plan)

This document describes how the monorepo maps to the future **browbridge** GitHub organisation. Nothing here creates remotes; it is preparation only.

## Target repositories

| Repository | Role |
| --- | --- |
| [browbridge/unibrowser](https://github.com/browbridge/unibrowser) | Desktop app (Tauri shell), bundled mods, mdBook docs, CI |
| [browbridge/registry](https://github.com/browbridge/registry) | Published mod index (`index.json` + checksums), generated from TOML entries |
| [browbridge/ubmod-example](https://github.com/browbridge/ubmod-example) | WASM mod template (from `examples/ub-mod-wasm`) |
| [browbridge/.github](https://github.com/browbridge/.github) | Org-wide community files (see `org/dot-github/`) |
| `browbridge/ubmod-<id>` | One repo per split-out mod (e.g. `ubmod-driveworks` for read-only `.driveprojx` / `.drivegroup` / `.drivespec`) |

## In-tree layout today

- `crates/` — engine, mod host, desktop shell
- `mods/` — bundled mods (must pass `scripts/check-mod-api-boundaries.sh`)
- `registry/mods/*.toml` — registry skeleton (`path` for in-tree mods)
- `scripts/build-registry-index.sh` — emits `registry/index.json` + `registry/checksums.sha256`
- `scripts/extract-mod.sh` — `git filter-repo` helper (dry-run by default)
- `org/dot-github/` — files to copy into `browbridge/.github`

## Split-out criteria

Extract a mod to its own repo when:

1. It has an independent release cadence or licence.
2. It is maintained by a different team or community.
3. Its binary size or third-party deps should not ship in the main app artefact.
4. It needs CI that does not run the full desktop matrix.

Keep mods bundled when they are required for first-run experience (text, web, PDF, project) or share tight coupling with engine snapshots.

## Esoteric / vertical mods

DriveWorks and similar formats ship as `browbridge/ubmod-<id>` (read-only handlers first). The registry entry points at the release artefact; Unibrowser may sideload or optional-download.

## Developer workflow

- Sign commits with DCO (`Signed-off-by`, see `CONTRIBUTING.md`); run `./scripts/check-dco.sh`.
- Before adding bundled mod deps: `./scripts/check-mod-api-boundaries.sh`.
- To rehearse a split: `./scripts/extract-mod.sh base-dxf` (dry-run), then `--execute` on a clone.
