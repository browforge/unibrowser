# `.unibrow` format migration

`project.json` carries a `format_version` field. The host accepts **v1 and v2**; saving always writes **v2**.

## Version 2 (current)

- Optional `description` string on the manifest (notes for collaborators).
- Stricter validation on open: non-empty title, unique node ids, timeline clips must reference existing nodes.

## Version 1

- Same core fields (`title`, `nodes`, `timeline`). Missing `description` is treated as `null`.
- Open v1 projects as usual; the next save upgrades `format_version` to 2.

## Unsupported versions

If `format_version` is missing, below 1, or above 2, open fails with a short message pointing here. Fix the file manually or re-export from a newer Unibrowser build.
